//! Débloque toutes les grâces et toutes les parties de la carte du monde.
//!
//! Fait une fois par session, au chargement de la partie. Les changements sont
//! enregistrés dans la sauvegarde (ce sont des drapeaux d'évènement du jeu).
//!
//! - Grâces : drapeau « grâce allumée » de chaque ligne de BonfireWarpParam (eventflag_id).
//!   On ne touche PAS au drapeau « cleared » (qui dépend de la mort d'un boss).
//! - Carte : drapeau d'ouverture de chaque morceau (WorldMapPieceParam, open_event_flag_id),
//!   plus le drapeau 82001 qui permet la vue des souterrains. Pas le drapeau d'obtention
//!   (acquisition_event_flag_id) : il affiche une fenêtre « fragment obtenu » à chaque fois.

use eldenring::cs::{
    BonfireWarpParam, CSEventFlagMan, ItemLotParam_map, SoloParamRepository, WorldChrMan,
    WorldMapPieceParam,
};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Lot d'objets « récompense du tableau Résurrection » (une larme larvaire). La DLL le rend
/// répétable (plus de drapeau « déjà obtenu ») : Melina s'en sert pour donner une larme
/// larvaire avant la renaissance, qui devient ainsi gratuite.
pub const LOT_LARME_LARVAIRE: u32 = 80312;
/// Larme larvaire (EquipParamGoods).
pub const LARME_LARVAIRE: i32 = 8185;

/// Règle le lot de la larme larvaire (une larme, toujours donnée, répétable).
fn preparer_lot_larme_larvaire() {
    let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
        return;
    };
    let Some(lot) = parametres.get_mut::<ItemLotParam_map>(LOT_LARME_LARVAIRE) else {
        noter("Renaissance : lot de la larme larvaire introuvable.");
        return;
    };
    lot.set_lot_item_id01(LARME_LARVAIRE);
    lot.set_lot_item_category01(1); // objet (goods)
    lot.set_lot_item_num01(1);
    lot.set_lot_item_base_point01(1000);
    lot.set_get_item_flag_id(0);
    lot.set_get_item_flag_id01(0);
    noter("Renaissance : lot de la larme larvaire prêt (gratuit et répétable).");
}

/// Emplacements de talisman débloqués en plus du premier (3 = les 4 emplacements, comme
/// avec les 3 sacoches de talisman du jeu).
const EMPLACEMENTS_DE_TALISMAN_EN_PLUS: u8 = 3;

/// Débloque tous les emplacements de talisman du joueur. Renvoie faux si le personnage
/// n'est pas encore chargé.
pub fn debloquer_les_talismans() -> bool {
    let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
        return false;
    };
    let Some(joueur) = monde.main_player.as_ref() else {
        return false;
    };
    let donnees = unsafe { &mut *joueur.player_game_data.as_ptr() };
    if donnees.unlocked_talisman_slots < EMPLACEMENTS_DE_TALISMAN_EN_PLUS {
        noter(&format!(
            "Talismans : {} emplacement(s) en plus débloqué(s), passage à {EMPLACEMENTS_DE_TALISMAN_EN_PLUS}.",
            donnees.unlocked_talisman_slots
        ));
        donnees.unlocked_talisman_slots = EMPLACEMENTS_DE_TALISMAN_EN_PLUS;
    }
    true
}

/// Drapeau qui permettrait d'afficher la vue des souterrains sur la carte (non vérifié).
const DRAPEAU_SOUTERRAINS_VISITES: u32 = 82001;


/// Drapeaux « journal de fabrication obtenu » du jeu de base (59 journaux). Ce sont les
/// drapeaux de déblocage des recettes (eventFlag_forRelease de ShopLineupParam_Recipe,
/// sauf ceux du DLC, 68510 et plus) : allumés, toutes les recettes sont proposées.
const DRAPEAUX_JOURNAUX_DE_FABRICATION: [u32; 59] = [
    67000, 67010, 67020, 67030, 67050, 67060, 67070, 67080,
    67090, 67100, 67110, 67120, 67130, 67200, 67210, 67220,
    67230, 67250, 67260, 67270, 67280, 67290, 67300, 67310,
    67400, 67410, 67420, 67430, 67440, 67450, 67460, 67470,
    67480, 67600, 67610, 67630, 67640, 67650, 67800, 67830,
    67840, 67850, 67860, 67870, 67880, 67890, 67900, 67910,
    67920, 68000, 68010, 68020, 68030, 68200, 68210, 68220,
    68230, 68400, 68410,
];

/// Les mêmes drapeaux pour les 45 journaux du DLC (68510 et plus).
const DRAPEAUX_JOURNAUX_DE_FABRICATION_DLC: [u32; 45] = [
    68510, 68520, 68530, 68540, 68550, 68560, 68570, 68580,
    68590, 68600, 68610, 68620, 68630, 68640, 68650, 68660,
    68670, 68680, 68690, 68700, 68710, 68720, 68730, 68740,
    68750, 68760, 68770, 68780, 68790, 68800, 68810, 68820,
    68830, 68840, 68850, 68860, 68870, 68880, 68890, 68900,
    68910, 68920, 68930, 68940, 68950,
];

/// Allume toutes les grâces et ouvre toute la carte. Renvoie faux si les données du jeu
/// ne sont pas encore prêtes (il faudra réessayer).
pub fn tout_debloquer() -> bool {
    let Ok(parametres) = (unsafe { SoloParamRepository::instance() }) else {
        return false;
    };
    let Ok(drapeaux) = (unsafe { CSEventFlagMan::instance_mut() }) else {
        return false;
    };

    let graces: Vec<u32> = parametres
        .rows::<BonfireWarpParam>()
        .map(|(_, ligne)| ligne.eventflag_id())
        .filter(|&d| d != 0)
        .collect();
    let mut morceaux: Vec<u32> = Vec::new();
    for (id, ligne) in parametres.rows::<WorldMapPieceParam>() {
        // Lignes 100 à 104 : souterrains (Ainsel, lac de la Putréfaction, Mohgwyn,
        // Siofra, Racines profondes). Notées dans le journal pour vérifier leurs drapeaux.
        if (100..200).contains(&id) {
            noter(&format!(
                "Carte souterraine {id} : drapeau d'ouverture {}, d'obtention {}.",
                ligne.open_event_flag_id(),
                ligne.acquisition_event_flag_id()
            ));
        }
        // Seulement le drapeau d'ouverture : le drapeau d'obtention déclenche la fenêtre
        // « fragment de carte obtenu », et le jeu l'éteint après l'avoir affichée (15 fenêtres
        // à chaque lancement). Les souterrains s'affichent grâce au drapeau 82001 plus bas.
        let d = ligne.open_event_flag_id();
        if d != 0 {
            morceaux.push(d);
        }
    }

    // On n'allume que ce qui est encore éteint (rien à refaire d'un lancement à l'autre).
    let mut allumes = 0;
    for &d in graces.iter().chain(morceaux.iter()) {
        if !drapeaux.virtual_memory_flag.get_flag(d) {
            drapeaux.virtual_memory_flag.set_flag(d, true);
            allumes += 1;
        }
    }
    preparer_lot_larme_larvaire();
    // Essai : drapeau « souterrains déjà visités » (82001, d'après la communauté des
    // moddeurs, non vérifié) : sans lui, la carte ne propose pas la vue des souterrains.
    drapeaux.virtual_memory_flag.set_flag(DRAPEAU_SOUTERRAINS_VISITES, true);
    for &d in DRAPEAUX_JOURNAUX_DE_FABRICATION.iter().chain(DRAPEAUX_JOURNAUX_DE_FABRICATION_DLC.iter()) {
        drapeaux.virtual_memory_flag.set_flag(d, true);
    }
    noter(&format!(
        "Déblocage : {} grâces et {} morceaux de carte vérifiés, {allumes} drapeau(x) allumé(s) cette fois.",
        graces.len(),
        morceaux.len()
    ));
    true
}

/// Drapeaux « boss vaincu » de la carte de chaque boss du menu (XXXX0800 ; 0850 pour
/// Margit). Éteints, le boss réapparaît au prochain chargement de sa zone. On ne touche
/// PAS aux drapeaux généraux « boss vaincu » (9100 et suivants) : ils pilotent l'avancée
/// du monde (capitale en cendres, etc.).
const DRAPEAUX_BOSSES_VAINCUS: [(u32, &str); 21] = [
    (10000850, "Margit"),
    (10000800, "Godrick"),
    (14000800, "Rennala"),
    (1252380800, "Radahn"),
    (11000800, "Morgott"),
    // Géant de feu : c'est le drapeau du grand carré (1252520800) qui le fait revenir
    // (vérifié le 07/11/2026) ; celui du petit carré est éteint aussi par sécurité.
    (1052520800, "Géant de feu"),
    (1252520800, "Géant de feu (grand carré)"),
    (13000800, "Maliketh"),
    (11050800, "Godfrey (Hoarah Loux)"),
    (12050800, "Mohg"),
    (16000800, "Rykard"),
    (15000800, "Malenia"),
    (19000800, "Radagon et la Bête d'Elden"),
    (20000800, "Bête dansante"),
    (2048440800, "Rellana"),
    (2054390800, "Bayle"),
    (21010800, "Messmer"),
    (2050480800, "Avatar de l'arbre occulte"),
    (2049480800, "Gaius"),
    (2044450800, "Romina"),
    (20010800, "Radahn Consort"),
];

/// Fait revivre tous les bosses du menu déjà vaincus. Renvoie le nombre de bosses
/// ranimés, ou None si les drapeaux ne sont pas accessibles.
pub fn faire_revivre_les_bosses() -> Option<usize> {
    let drapeaux = unsafe { CSEventFlagMan::instance_mut() }.ok()?;
    let mut ranimes = Vec::new();
    for &(d, nom) in DRAPEAUX_BOSSES_VAINCUS.iter() {
        if drapeaux.virtual_memory_flag.get_flag(d) {
            drapeaux.virtual_memory_flag.set_flag(d, false);
            ranimes.push(nom);
        }
    }
    noter(&format!("Bosses ranimés ({}) : {:?}.", ranimes.len(), ranimes));
    Some(ranimes.len())
}
