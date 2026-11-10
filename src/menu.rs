//! Le menu de Melina, avec le vrai menu de dialogue du jeu (manette et clavier d'office).
//!
//!   Menu principal : Bosses / Vitesse du jeu / Fioles / Niveau / Marchands / Effets d'arme /
//!                    Partir
//!   Effets d'arme  : Main droite / Main gauche, puis l'effet (visuel d'une graisse ou d'un
//!                    sort, permanent, sans dégâts ; voir effets_arme.rs)
//!   Bosses         : Jeu de base / DLC / Faire revivre les bosses (vaincus) / Retour
//!   Jeu de base    : Margit, Godrick, Rennala, Radahn, Morgott, Géant de feu, Maliketh, Godfrey, Mohg, Rykard, Malenia, Radagon (téléportation vers la grâce du combat)
//!   DLC            : Bête dansante, Rellana, Bayle, Messmer, Avatar de l'arbre occulte,
//!                    Gaius, Romina, Radahn Consort (grâce juste devant chaque combat)
//!   Vitesse        : 100 % à 300 %, par paliers de 10 %
//!   Fioles         : le menu « Répartir les fioles » du jeu, 14 fioles au total
//!   Niveau         : Monter de niveau (écran du jeu, gratuit) / Renaissance (gratuite :
//!                    Melina donne la larme larvaire demandée) / Améliorations (graines
//!                    dorées, larmes sacrées, fragments, cendres vénérées) / Retour
//!   Marchands      : Équipement / Talismans / Sorts / Matériaux / Cendres d'esprit /
//!                    Retour (vrais magasins,
//!                    gratuits, jeu de base puis DLC dans chacun, voir armes.rs)
//!   Équipement     : Armes / Armures / Forgeron (comme Hewg) / Pierres de forge (99 de
//!                    chaque, gratuites) / Cendres de guerre / Munitions / Retour
//!   Sorts          : Sorcelleries / Incantations / Retour
//!   Cendres d'esprit : Niveau +0 / Niveau +10 / Muguets / Améliorer les cendres
//!                    (menu du jeu, comme Roderika, gratuit) / Retour
//!
//! Les textes sont dans le fichier de textes du mod (Smithbox), numéros 69000000 et suivants.
//! La liste complète est dans outils\textes_menu.txt.

use eldenring::{
    cs::{FieldInsHandle, MenuType, TalkScript, WorldChrMan},
    ez_state::EzStateValue,
};
use fromsoftware_shared::FromStatic;

use crate::armes::Rayon;
use crate::journal::noter;
use crate::vitesse::{self, Vitesse};

// --- Commandes du système de dialogue du jeu (TalkESD) ---
const AFFICHER_LISTE: i32 = 10; // ShowShopMessage
const AJOUTER_LIGNE: i32 = 19; // AddTalkListData
const VIDER_LISTE: i32 = 20; // ClearTalkListData
const RESULTAT_LISTE: i32 = 23; // GetTalkListEntryResult
const DIALOGUE_OUVERT: i32 = 58; // CheckSpecificPersonGenericDialogIsOpen
const MENU_OUVERT: i32 = 59; // CheckSpecificPersonMenuIsOpen
const OUVRIR_REPARTITION_FIOLES: i32 = 105; // OpenEstusAllotMenu
const OUVRIR_RENAISSANCE: i32 = 113; // ReallocateAttributes (menu de renaissance)
const DONNER_LOT: i32 = 104; // AwardItemLot
const OUVRIR_MONTEE_DE_NIVEAU: i32 = 31; // OpenSoul
const OUVRIR_MAGASIN: i32 = 22; // OpenRegularShop (première ligne, dernière ligne)
const OUVRIR_FORGERON: i32 = 24; // OpenEnhanceShop (0 = amélioration complète)
const OUVRIR_REINCARNATRICE: i32 = 136; // c1_136 : amélioration des cendres d'esprit
const LIER_DRAPEAU_DE_MENU: i32 = 49; // CombineMenuFlagAndEventFlag (drapeau de menu, drapeau)
const NOTER_JOURNAL_DE_JEU: i32 = 141; // RecordPlayLog

const A_L_OBJET: i32 = 16; // DoesPlayerHaveItem (catégorie, objet)
/// Catégorie « objet » pour DoesPlayerHaveItem.
const CATEGORIE_OBJET: i32 = 3;

// --- Textes du menu (fichier de textes du mod) ---
const TEXTE_BOSSES: i32 = 69000000;
const TEXTE_VITESSE: i32 = 69000001;
const TEXTE_FIOLES: i32 = 69000002;
const TEXTE_RENAISSANCE: i32 = 69000006;
const TEXTE_JEU_DE_BASE: i32 = 69000031;
const TEXTE_DLC: i32 = 69000032;
const TEXTE_NIVEAU: i32 = 69000033;
const TEXTE_MONTER_DE_NIVEAU: i32 = 69000034;
const TEXTE_MARCHANDS: i32 = 69000035;
const TEXTE_FORGERON: i32 = 69000036;
const TEXTE_ARMES: i32 = 69000040;
const TEXTE_ARMURES: i32 = 69000045;
const TEXTE_TALISMANS: i32 = 69000050;
const TEXTE_MATERIAUX: i32 = 69000051;
const TEXTE_EQUIPEMENT: i32 = 69000052;
const TEXTE_SORTS: i32 = 69000053;
const TEXTE_CENDRES: i32 = 69000056;
const TEXTE_AMELIORATIONS: i32 = 69000057;
const TEXTE_ESPRITS: i32 = 69000058;
const TEXTE_FAIRE_REVIVRE: i32 = 69000063;
const TEXTE_MUNITIONS: i32 = 69000064;
const TEXTE_EFFETS_ARME: i32 = 69000065;
const TEXTE_AUCUN: i32 = 69000066;
const TEXTE_MAIN_DROITE: i32 = 69000075;
const TEXTE_MAIN_GAUCHE: i32 = 69000076;
const TEXTE_EFFET: i32 = 69000095;
const TEXTE_TRAINEE: i32 = 69000096;
const TEXTE_ROULADE: i32 = 69000097;
const TEXTE_ROULADE_NORMALE: i32 = 69000098;
const TEXTE_ROULADE_RADAGON: i32 = 69000099;
const TEXTE_ROULADE_LIMIER: i32 = 69000310;
const TEXTE_ROULADE_COUTEAU_NOIR: i32 = 69000311;
const TEXTE_ROULADE_REVENANT: i32 = 69000312;
const TEXTE_ROULADE_CHEVALIER_MORT: i32 = 69000313;
const TEXTE_ROULADE_TENEBRES: i32 = 69000314;
const TEXTE_ROULADE_THOPS: i32 = 69000315;
const TEXTE_ROULADE_PLEINE_LUNE: i32 = 69000316;
const TEXTE_ROULADE_GRAVITATIONNEL: i32 = 69000317;
const TEXTE_ROULADE_MAGMA: i32 = 69000340;
/// Impact (voir impact.rs) : l'onglet.
const TEXTE_IMPACT: i32 = 69000526;
/// Impact de la main gauche : l'onglet.
const TEXTE_IMPACT_GAUCHE: i32 = 69000558;
/// « Réinitialiser tous les effets » (voir reglages.rs).
const TEXTE_REINITIALISER: i32 = 69000580;
/// Arme fantôme (voir arme_fantome.rs) : l'onglet.
const TEXTE_ARME_FANTOME: i32 = 69000541;
const TEXTE_ARME_FANTOME_GAUCHE: i32 = 69000543;
/// Caméra (voir camera.rs) : l'onglet, puis 69000600 + recul (69000605 = « +5 % »,
/// 69000700 = « +100 % »).
const TEXTE_CAMERA: i32 = 69000600;
const TEXTE_ROULADE_HALOS: i32 = 69000341;
const TEXTE_ROULADE_FLAMME_NOIRE: i32 = 69000342;
const TEXTE_ROULADE_SERPENT: i32 = 69000343;
const TEXTE_ROULADE_GURRANQ: i32 = 69000344;
const TEXTE_ROULADE_DISQUE: i32 = 69000345;
const TEXTE_ROULADE_FOUDROIEMENT: i32 = 69000346;
const TEXTE_AURA: i32 = 69000320;
const TEXTE_COURSE: i32 = 69000350;
const TEXTE_COURSE_ESSAIS: i32 = 69000452;
/// Galerie des sorts (voir galerie.rs) : l'onglet et ses lignes.
const TEXTE_GALERIE: i32 = 69000510;
const TEXTE_GALERIE_REPRENDRE: i32 = 69000511;
const TEXTE_GALERIE_PRECEDENT: i32 = 69000512;
const TEXTE_GALERIE_INCANTATIONS: i32 = 69000513;
const TEXTE_GALERIE_EXTENSION: i32 = 69000514;
const TEXTE_GALERIE_DEBUT: i32 = 69000515;
const TEXTE_GALERIE_ARRETER: i32 = 69000516;
const TEXTE_AMELIORER_ESPRITS: i32 = 69000062;
/// Ligne « Améliorer les cendres » du sous-menu « Cendres d'esprit » (après les magasins).
const LIGNE_AMELIORER_ESPRITS: i32 = 50;
const TEXTE_PIERRES_DE_FORGE: i32 = 69000037;
const TEXTE_RETOUR: i32 = 69000005;
/// Texte « Partir », déjà présent dans le jeu.
const TEXTE_PARTIR: i32 = 20000009;
/// Les vitesses : texte 69000000 + pourcentage (69000100 = « 100 % », 69000300 = « 300 % »).
const TEXTE_VITESSE_BASE: i32 = 69000000;

/// Les bosses : (texte du nom, grâce où l'on est téléporté).
/// Les grâces viennent de la liste des grâces du jeu (BonfireWarpParam). Elles servent à
/// reconnaître le boss ; le départ et le point d'arrivée (devant la brume) sont réglés
/// dans arrivees.rs :
///   Margit  : grâce « Margit, le Déchu » (dans l'arène)          10001951
///   Godrick : « Cellule isolée », juste avant la brume            10001957
///   Rennala : « Salle de débat », juste avant la brume            14001951
///   Radahn  : grâce « Radahn » (dans l'arène des dunes)           1052381950
///   Morgott : « Trône d'Elden » (dans l'arène)                    11001950
///   Géant de feu : grâce « Géant de feu » (dans l'arène)          1053521950
///   Maliketh : grâce « Maliketh » (dans l'arène)                  13001950
///   Godfrey : « Trône d'Elden » de Leyndell en cendres (arène)    11051950
///   Mohg : « Cocon de l'Empyréen » (dans l'arène)                 12051950
///   Rykard : « Allée de l'Audience » (Audience Pathway, ligne 160005), juste devant le
///            combat                                               16001955
///   Malenia : « Racines de l'arbre sacré » (Haligtree Roots, ligne 150004), juste
///             devant le combat                                     15001954
///   Radagon / Bête d'Elden : « Marika brisée » (dans l'arène)     19001950
/// Si une grâce ne convient pas (trop loin, combat qui ne démarre pas), on la change ici.
/// Pour certains bosses, la DLL part d'une autre grâce puis place le joueur devant la
/// brume : voir arrivees.rs (reconnus par la grâce indiquée ici).
const BOSSES: [(i32, u32); 12] = [
    (69000010, 10001951),   // Margit
    (69000011, 10001957),   // Godrick
    (69000012, 14001951),   // Rennala
    (69000013, 1052381950), // Radahn
    (69000014, 11001950),   // Morgott
    (69000015, 1053521950), // Géant de feu
    (69000016, 13001950),   // Maliketh
    (69000017, 11051950),   // Godfrey
    (69000018, 12051950),   // Mohg
    (69000030, 16001955),   // Rykard
    (69000029, 15001954),   // Malenia
    (69000019, 19001950),   // Radagon / Bête d'Elden
];

/// Les bosses du DLC (Shadow of the Erdtree) : (texte du nom, grâce où l'on est téléporté).
///   Bête dansante : grâce « Devant la scène » (ligne 200003), juste devant le combat.
///   Rellana : grâce « Chambre du seigneur du château » (ligne 61484401), juste devant le
///             combat. Le numéro ci-dessous sert à la reconnaître ; le vrai numéro de grâce
///             est lu dans les données du jeu (voir arrivees.rs).
///   Les suivants de même (grâce lue dans les données du jeu, voir arrivees.rs).
const BOSSES_DLC: [(i32, u32); 8] = [
    (69000020, 20001953),   // Bête dansante
    (69000021, 2048441951), // Rellana
    (69000022, 2054391950), // Bayle
    (69000023, 21011954), // Messmer
    (69000024, 2050481951), // Avatar de l'arbre occulte
    (69000025, 2049481951), // Gaius
    (69000026, 2044461950), // Romina
    // Metyr : retiré pour le moment (accès par la quête de Ymir, à revoir plus tard).
    (69000027, 20011956), // Radahn Consort
];

/// Numéro de ligne « Retour » dans les sous-menus.
const LIGNE_RETOUR: i32 = 99;

/// Nombre total de fioles (soin + mana).
pub const FIOLES_TOTAL: u8 = 14;

/// Rang (à partir de 0) de la ligne choisie si c'est une ligne de la liste (1 à nombre),
/// sinon None (Retour, ou menu fermé).
fn choisi(choix: i32, nombre: usize) -> Option<usize> {
    if choix >= 1 && (choix as usize) <= nombre {
        Some(choix as usize - 1)
    } else {
        None
    }
}

/// Où en est le menu.
#[derive(Clone, Copy, PartialEq)]
enum Ecran {
    Ferme,
    /// À l'ouverture du menu : Melina donne les objets de départ qui manquent au joueur
    /// (kit de confection, fioles, pots, pierres de mémoire... voir OBJETS_DE_DEPART).
    DonnerObjetsDeDepart,
    Principal,
    /// Sous-menu « Bosses » : Jeu de base / DLC.
    ChoixBosses,
    Bosses,
    BossesDlc,
    Vitesse,
    Fioles,
    /// Sous-menu « Niveau » : Monter de niveau / Renaissance.
    Niveau,
    MonteeDeNiveau,
    /// Sous-menu « Cendres d'esprit » : Niveau +0 / Niveau +10 / Muguets / Améliorer.
    Esprits,
    /// Le menu d'amélioration des cendres d'esprit du jeu (comme chez Roderika), gratuit.
    Reincarnatrice,
    Renaissance,
    /// Sous-menu « Marchands » : Pierres de forge.
    Marchands,
    /// Sous-menu « Marchands > Équipement » : Armes / Armures / Forgeron / Pierres de forge.
    Equipement,
    /// Effets d'arme : choix de la main.
    EffetsArme,
    /// Aura : aucune / visuel d'un buff sur le personnage.
    Aura,
    /// Style de roulade : normale / téléportation de Radagon / pas de chevalier-limier /
    /// esquive du Couteau noir / téléportation du Revenant royal / du Chevalier de la mort.
    Roulade,
    /// Traînée de course : aucune / un style (voir course.rs).
    Course,
    /// Traînées à l'essai : chaque sort mesuré, en entier puis visuel par visuel.
    CourseEssais,
    /// Galerie des sorts : reprendre, sort précédent, aller à un groupe, arrêter.
    Galerie,
    /// Impact : aucun / un visuel joué sur l'ennemi touché (voir impact.rs).
    Impact,
    /// Impact de la main gauche.
    ImpactGauche,
    /// Arme fantôme : aucune / une arme de sort tenue pendant les attaques (voir arme_fantome.rs).
    ArmeFantome,
    /// Arme fantôme de la main gauche.
    ArmeFantomeGauche,
    /// Recul de la caméra : normale / +5 % à +100 %.
    Camera,
    /// Effets d'arme d'une main : Effet (la lueur) / Traînée (couleur de la trace des coups).
    EffetsArmeMain(crate::effets_arme::Main),
    /// Choix de l'effet ou de la traînée d'une main (visuel permanent, sans dégâts).
    EffetsArmeChoix(crate::effets_arme::Emplacement),
    /// Melina donne les pierres de forge, une sorte par une.
    DonnerPierres,
    /// Forgeron : comme Hewg (script t213001110 de la Table ronde).
    Forgeron,
    /// Armes (jeu de base, voir armes.rs) : la liste des magasins, puis le magasin choisi.
    ListeMagasins(Rayon),
    Magasin(Rayon, usize),
}

/// Images entre deux pierres données (le temps que le jeu traite le lot d'objets).
const IMAGES_ENTRE_DEUX_PIERRES: u32 = 10;

/// Objets de départ (EquipParamGoods, nombre voulu) : Melina complète ce qui manque à
/// chaque ouverture de son menu, donc une seule fois pour un nouveau personnage.
const OBJETS_DE_DEPART: [(u32, u8, &str); 14] = [
    (8500, 1, "Kit de confection"),
    (9500, 20, "Pot fêlé"),
    (9501, 10, "Pot rituel"),
    (2009500, 10, "Gros pot fêlé (DLC)"),
    (9510, 10, "Fiole de parfumeur"),
    (10030, 8, "Pierre de mémoire"),
    (8590, 1, "Couteau à aiguiser"),
    (8970, 1, "Lame d'affûtage en fer"),
    (8971, 1, "Lame d'affûtage chauffée à blanc"),
    (8972, 1, "Lame d'affûtage sanctifiée"),
    (8973, 1, "Lame d'affûtage en pierre d'éclat"),
    (8974, 1, "Lame d'affûtage noire"),
    (130, 1, "Sifflet du destrier spectral"),
    (8158, 1, "Clochette d'appel spirituel"),
];

pub struct MenuMelina {
    /// Effets d'arme choisis dans « Effets d'arme » du menu principal (voir effets_arme.rs).
    pub effets_arme: crate::effets_arme::EffetsArme,
    /// Style de roulade choisi dans « Effets d'arme > Roulade » (voir roulade.rs).
    pub roulade: crate::roulade::Roulade,
    /// Aura choisie dans « Effets d'arme > Aura » (voir aura.rs).
    pub aura: crate::aura::Aura,
    /// Traînée de course choisie dans « Effets d'arme > Course » (voir course.rs).
    pub course: crate::course::Course,
    /// Galerie des sorts, dans « Effets d'arme > Galerie des sorts » (voir galerie.rs).
    pub galerie: crate::galerie::Galerie,
    /// Impact choisi dans « Effets d'arme > Impact » (voir impact.rs).
    pub impact: crate::impact::Impact,
    /// Impact de la main gauche (« Effets d'arme > Impact main gauche »).
    pub impact_gauche: crate::impact::Impact,
    /// Arme fantôme choisie dans « Effets d'arme > Arme fantôme ».
    pub arme_fantome: crate::arme_fantome::ArmeFantome,
    pub arme_fantome_gauche: crate::arme_fantome::ArmeFantome,
    /// Recul de la caméra choisi dans « Caméra » du menu principal.
    pub camera: crate::camera::Camera,
    script: Box<TalkScript>,
    ecran: Ecran,
    /// Vrai quand la liste de l'écran actuel est affichée et attend un choix.
    liste_affichee: bool,
    /// Grâce choisie dans la liste des bosses (la téléportation se fait une fois le menu fermé).
    grace_choisie: Option<u32>,
    /// Renaissance : images restantes à attendre la larme larvaire donnée par Melina
    /// (None : la larme est là, le menu de renaissance est ouvert).
    attente_larme: Option<u32>,
    /// Runes du joueur avant la montée de niveau ou la forge (gratuites : on lui donne le
    /// maximum pendant l'écran, puis on lui rend celles-ci).
    runes_avant: Option<u32>,
    /// Écran du jeu (montée de niveau, forgeron, magasin) : vrai une fois qu'on l'a vu
    /// ouvert. Le forgeron et le magasin mettent quelques images à s'ouvrir : avant, le jeu
    /// les dit fermés, et on réaffichait la liste de Melina par-dessus (écran vide).
    ecran_du_jeu_vu: bool,
    /// Images restantes à attendre l'ouverture de l'écran du jeu.
    attente_ouverture: u32,
    /// Pierres de forge : prochaine pierre à donner (rang dans PIERRES_DE_FORGE), et
    /// images à attendre avant de la donner.
    pierre_suivante: usize,
    attente_pierre: u32,
    /// Fioles de départ à donner (vrai tant que Melina ne les a pas données, voir
    /// `donner_les_objets_de_depart`).
    fioles_a_donner: bool,
}

/// Images à attendre au plus l'ouverture d'un écran du jeu (environ 5 s à 60 images/s).
const ATTENTE_OUVERTURE_MAX: u32 = 300;

impl MenuMelina {
    pub fn new() -> Self {
        Self {
            script: Box::new(TalkScript::new(
                eldenring::cs::BlockId::none(),
                1000,
                FieldInsHandle {
                    block_id: eldenring::cs::BlockId::none(),
                    selector: eldenring::cs::FieldInsSelector(0),
                },
            )),
            ecran: Ecran::Ferme,
            liste_affichee: false,
            grace_choisie: None,
            attente_larme: None,
            runes_avant: None,
            ecran_du_jeu_vu: false,
            attente_ouverture: 0,
            pierre_suivante: 0,
            fioles_a_donner: false,
            attente_pierre: 0,
            effets_arme: crate::effets_arme::EffetsArme::new(),
            roulade: crate::roulade::Roulade::new(),
            aura: crate::aura::Aura::new(),
            course: crate::course::Course::new(),
            galerie: crate::galerie::Galerie::new(),
            impact: crate::impact::Impact::new(crate::impact::Main::Droite),
            impact_gauche: crate::impact::Impact::new(crate::impact::Main::Gauche),
            arme_fantome: crate::arme_fantome::ArmeFantome::new(crate::arme_fantome::Main::Droite),
            arme_fantome_gauche: crate::arme_fantome::ArmeFantome::new(crate::arme_fantome::Main::Gauche),
            camera: crate::camera::Camera::new(),
        }
    }

    /// Ferme le menu sans rien faire (écran de chargement).
    pub fn fermer(&mut self) {
        self.rendre_les_runes();
        self.ecran = Ecran::Ferme;
    }

    /// Rend au joueur ses runes d'avant la montée de niveau ou la forge.
    fn rendre_les_runes(&mut self) {
        if let Some(avant) = self.runes_avant.take() {
            crate::marchands::rendre_les_runes(avant);
        }
    }

    pub fn est_ouvert(&self) -> bool {
        self.ecran != Ecran::Ferme
    }

    /// Ouvre le menu principal (quand le joueur parle à Melina).
    pub fn ouvrir(&mut self, melina: FieldInsHandle) {
        self.script.npc_talk.base.field_ins_handle = melina;
        // D'abord les objets de départ (seulement ceux qui manquent), puis le menu.
        self.aller_a(Ecran::DonnerObjetsDeDepart);
        noter("Menu de Melina ouvert.");
    }

    /// À appeler à chaque image tant que le menu est ouvert.
    /// Renvoie la grâce du boss choisi, une fois le menu fermé (sinon None).
    pub fn mettre_a_jour(&mut self, vitesse: &mut Vitesse) -> Option<u32> {
        if self.ecran == Ecran::Ferme {
            return None;
        }
        if let Err(erreur) = self.etape(vitesse) {
            noter(&format!("Menu de Melina : erreur {erreur:?}, menu fermé."));
            self.ecran = Ecran::Ferme;
        }
        if self.ecran == Ecran::Ferme {
            return self.grace_choisie.take();
        }
        None
    }

    fn aller_a(&mut self, ecran: Ecran) {
        self.ecran = ecran;
        self.liste_affichee = false;
        self.ecran_du_jeu_vu = false;
        self.attente_ouverture = ATTENTE_OUVERTURE_MAX;
    }

    fn etape(&mut self, vitesse: &mut Vitesse) -> Result<(), eldenring::cs::EzStateInvokeError> {
        // 1) Afficher la liste de l'écran, une fois.
        if !self.liste_affichee {
            self.liste_affichee = true;
            match self.ecran {
                Ecran::Principal => self.afficher(&[
                    (1, TEXTE_BOSSES),
                    (2, TEXTE_VITESSE),
                    (3, TEXTE_FIOLES),
                    (7, TEXTE_NIVEAU),
                    (8, TEXTE_MARCHANDS),
                    (9, TEXTE_EFFETS_ARME),
                    (10, TEXTE_CAMERA),
                    (4, TEXTE_PARTIR),
                ])?,
                Ecran::ChoixBosses => self.afficher(&[
                    (1, TEXTE_JEU_DE_BASE),
                    (2, TEXTE_DLC),
                    (3, TEXTE_FAIRE_REVIVRE),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::Niveau => self.afficher(&[
                    (1, TEXTE_MONTER_DE_NIVEAU),
                    (2, TEXTE_RENAISSANCE),
                    (3, TEXTE_AMELIORATIONS),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::Marchands => self.afficher(&[
                    (1, TEXTE_EQUIPEMENT),
                    (2, TEXTE_TALISMANS),
                    (3, TEXTE_SORTS),
                    (4, TEXTE_MATERIAUX),
                    (5, TEXTE_ESPRITS),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::Equipement => self.afficher(&[
                    (1, TEXTE_ARMES),
                    (2, TEXTE_ARMURES),
                    (3, TEXTE_FORGERON),
                    (4, TEXTE_PIERRES_DE_FORGE),
                    (5, TEXTE_CENDRES),
                    (6, TEXTE_MUNITIONS),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::EffetsArme => {
                    let mut lignes = vec![
                        (1, TEXTE_MAIN_DROITE),
                        (2, TEXTE_MAIN_GAUCHE),
                        (3, TEXTE_ROULADE),
                        (4, TEXTE_AURA),
                        (5, TEXTE_COURSE),
                    ];
                    // Onglet des essais seulement quand il y a des essais en cours.
                    if !crate::course::ESSAIS.is_empty() {
                        lignes.push((6, TEXTE_COURSE_ESSAIS));
                    }
                    lignes.push((7, TEXTE_GALERIE));
                    lignes.push((8, TEXTE_IMPACT));
                    // Impact de la main gauche juste sous l'impact (numéro 11 gardé).
                    lignes.push((11, TEXTE_IMPACT_GAUCHE));
                    lignes.push((9, TEXTE_ARME_FANTOME));
                    lignes.push((10, TEXTE_ARME_FANTOME_GAUCHE));
                    lignes.push((12, TEXTE_REINITIALISER));
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Course => {
                    // Numéros de ligne : 1 = aucune, puis 100 + rang du style (pour ne
                    // jamais tomber sur la ligne Retour, 99).
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, style) in crate::course::STYLES.iter().enumerate() {
                        lignes.push((100 + i as i32, style.texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Camera => {
                    let mut lignes = vec![(1, TEXTE_ROULADE_NORMALE)];
                    let mut recul = crate::camera::PALIER;
                    while recul <= crate::camera::RECUL_MAX {
                        lignes.push((100 + recul as i32, TEXTE_CAMERA + recul as i32));
                        recul += crate::camera::PALIER;
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::ArmeFantome | Ecran::ArmeFantomeGauche => {
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, f) in crate::arme_fantome::FANTOMES.iter().enumerate() {
                        lignes.push((100 + i as i32, f.texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Impact | Ecran::ImpactGauche => {
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, &(texte, _, _)) in crate::impact::IMPACTS.iter().enumerate() {
                        lignes.push((100 + i as i32, texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Galerie => self.afficher(&[
                    (1, TEXTE_GALERIE_REPRENDRE),
                    (2, TEXTE_GALERIE_PRECEDENT),
                    (3, TEXTE_GALERIE_INCANTATIONS),
                    (4, TEXTE_GALERIE_EXTENSION),
                    (5, TEXTE_GALERIE_DEBUT),
                    (6, TEXTE_GALERIE_ARRETER),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::CourseEssais => {
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, style) in crate::course::ESSAIS.iter().enumerate() {
                        lignes.push((100 + i as i32, style.texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Aura => {
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, &(texte, _, _)) in crate::aura::AURAS.iter().enumerate() {
                        lignes.push((i as i32 + 2, texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Roulade => self.afficher(&[
                    (1, TEXTE_ROULADE_NORMALE),
                    (2, TEXTE_ROULADE_RADAGON),
                    (3, TEXTE_ROULADE_LIMIER),
                    (4, TEXTE_ROULADE_COUTEAU_NOIR),
                    (5, TEXTE_ROULADE_REVENANT),
                    (6, TEXTE_ROULADE_CHEVALIER_MORT),
                    (7, TEXTE_ROULADE_TENEBRES),
                    (8, TEXTE_ROULADE_THOPS),
                    (9, TEXTE_ROULADE_PLEINE_LUNE),
                    (10, TEXTE_ROULADE_GRAVITATIONNEL),
                    (11, TEXTE_ROULADE_MAGMA),
                    (12, TEXTE_ROULADE_HALOS),
                    (13, TEXTE_ROULADE_FLAMME_NOIRE),
                    (14, TEXTE_ROULADE_SERPENT),
                    (15, TEXTE_ROULADE_GURRANQ),
                    (16, TEXTE_ROULADE_DISQUE),
                    (17, TEXTE_ROULADE_FOUDROIEMENT),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::EffetsArmeMain(_) => self.afficher(&[
                    (1, TEXTE_EFFET),
                    (2, TEXTE_TRAINEE),
                    (LIGNE_RETOUR, TEXTE_RETOUR),
                ])?,
                Ecran::EffetsArmeChoix(_) => {
                    let mut lignes = vec![(1, TEXTE_AUCUN)];
                    for (i, &(texte, _, _, _, _)) in crate::effets_arme::EFFETS.iter().enumerate() {
                        lignes.push((i as i32 + 2, texte));
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Forgeron => {
                    // Exactement ce que fait Hewg avant d'ouvrir sa forge : il lie le
                    // drapeau de menu 6001 aux drapeaux 232 à 235, note le journal de jeu
                    // (9), puis ouvre la forge. Sans ces liens, la forge restait vide.
                    self.runes_avant = crate::marchands::donner_les_runes();
                    for drapeau in 232..=235 {
                        self.script.event((
                            LIER_DRAPEAU_DE_MENU,
                            [EzStateValue::Int32(6001), EzStateValue::Int32(drapeau)],
                        ))?;
                    }
                    self.script.event((NOTER_JOURNAL_DE_JEU, [EzStateValue::Int32(9)]))?;
                    self.script.event((OUVRIR_FORGERON, [EzStateValue::Int32(0)]))?;
                    noter("Forgeron ouvert (comme Hewg).");
                }
                Ecran::ListeMagasins(rayon) => {
                    let mut lignes: Vec<(i32, i32)> = rayon
                        .magasins()
                        .iter()
                        .enumerate()
                        .map(|(i, magasin)| (i as i32 + 1, magasin.texte))
                        .collect();
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Magasin(rayon, rang) => {
                    // Le jeu cache une arme dont le prix d'achat est sous son prix de revente :
                    // les lignes ont donc un vrai prix (revente + 100), et l'achat est rendu
                    // gratuit comme la montée de niveau (runes au maximum, rendues à la fin).
                    self.runes_avant = crate::marchands::donner_les_runes();
                    let magasin = &rayon.magasins()[rang];
                    self.script.event((
                        OUVRIR_MAGASIN,
                        [
                            EzStateValue::Int32(magasin.premiere_ligne),
                            EzStateValue::Int32(magasin.derniere_ligne),
                        ],
                    ))?;
                    noter(&format!("Magasin {} ouvert.", magasin.premiere_ligne));
                }
                Ecran::Esprits => {
                    let mut lignes: Vec<(i32, i32)> = Rayon::Esprits
                        .magasins()
                        .iter()
                        .enumerate()
                        .map(|(i, magasin)| (i as i32 + 1, magasin.texte))
                        .collect();
                    lignes.push((LIGNE_AMELIORER_ESPRITS, TEXTE_AMELIORER_ESPRITS));
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Reincarnatrice => {
                    // Gratuit comme la forge : runes au maximum, rendues à la fermeture.
                    self.runes_avant = crate::marchands::donner_les_runes();
                    noter("Amélioration des cendres d'esprit ouverte.");
                    self.script.event(OUVRIR_REINCARNATRICE)?;
                }
                Ecran::MonteeDeNiveau => {
                    self.runes_avant = crate::marchands::donner_les_runes();
                    noter("Montée de niveau ouverte (gratuite).");
                    self.script.event(OUVRIR_MONTEE_DE_NIVEAU)?;
                }
                Ecran::DonnerObjetsDeDepart => {
                    self.pierre_suivante = 0;
                    self.attente_pierre = 0;
                    self.fioles_a_donner = !fioles_obtenues();
                }
                Ecran::DonnerPierres => {
                    noter("Pierres de forge : Melina les donne.");
                    self.pierre_suivante = 0;
                    self.attente_pierre = 0;
                }
                Ecran::Bosses => {
                    let mut lignes: Vec<(i32, i32)> = BOSSES
                        .iter()
                        .enumerate()
                        .map(|(i, &(texte, _))| (i as i32 + 1, texte))
                        .collect();
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::BossesDlc => {
                    let mut lignes: Vec<(i32, i32)> = BOSSES_DLC
                        .iter()
                        .enumerate()
                        .map(|(i, &(texte, _))| (i as i32 + 1, texte))
                        .collect();
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Vitesse => {
                    let mut lignes = Vec::new();
                    let mut pourcentage = vitesse::NORMALE;
                    while pourcentage <= vitesse::MAX {
                        lignes.push((pourcentage as i32, TEXTE_VITESSE_BASE + pourcentage as i32));
                        pourcentage += vitesse::PALIER;
                    }
                    lignes.push((LIGNE_RETOUR, TEXTE_RETOUR));
                    self.afficher(&lignes)?;
                }
                Ecran::Fioles => {
                    regler_total_des_fioles();
                    self.script.event(OUVRIR_REPARTITION_FIOLES)?;
                }
                Ecran::Renaissance => {
                    // Renaissance gratuite : Melina donne une larme larvaire si le joueur
                    // n'en a pas, puis ouvre le menu de renaissance du jeu.
                    if self.a_une_larme_larvaire()? {
                        self.ouvrir_renaissance()?;
                    } else {
                        noter("Renaissance : Melina donne une larme larvaire.");
                        self.script.event((
                            DONNER_LOT,
                            [EzStateValue::Int32(crate::deblocage::LOT_LARME_LARVAIRE as i32)],
                        ))?;
                        self.attente_larme = Some(600);
                    }
                }
                Ecran::Ferme => {}
            }
            return Ok(());
        }

        // 2) Le menu des fioles : on attend qu'il se ferme, puis retour au menu principal.
        if self.ecran == Ecran::Fioles {
            if !self.menu_encore_ouvert(MenuType::EstusAllotMenu)? {
                self.aller_a(Ecran::Principal);
            }
            return Ok(());
        }

        // 2 bis) Le menu de renaissance : on attend la larme (et la fin de son message),
        // puis on ouvre le menu ; ensuite on attend qu'il se ferme.
        if self.ecran == Ecran::Renaissance
            && let Some(images) = self.attente_larme
        {
            let dialogue: i32 = self.script.env((DIALOGUE_OUVERT, [EzStateValue::Int32(0)]))?.into();
            if dialogue == 0 && self.a_une_larme_larvaire()? {
                self.attente_larme = None;
                self.ouvrir_renaissance()?;
            } else if images == 0 {
                noter("Renaissance : la larme larvaire n'est pas arrivée, retour au menu.");
                self.attente_larme = None;
                self.aller_a(Ecran::Niveau);
            } else {
                self.attente_larme = Some(images - 1);
            }
            return Ok(());
        }
        if self.ecran == Ecran::Renaissance {
            if !self.menu_encore_ouvert(MenuType::ReallocateAttributes)? {
                self.aller_a(Ecran::Niveau);
            }
            return Ok(());
        }

        // 2 bis) Les objets de départ : un objet manquant toutes les quelques images, puis
        // on attend la fin des messages « objet obtenu » et on affiche le menu principal.
        if self.ecran == Ecran::DonnerObjetsDeDepart {
            self.donner_les_objets_de_depart()?;
            return Ok(());
        }

        // 2 bis) Les pierres de forge : une sorte toutes les quelques images, puis on
        // vérifie que le joueur les a bien toutes et on revient au sous-menu.
        if self.ecran == Ecran::DonnerPierres {
            self.donner_les_pierres()?;
            return Ok(());
        }

        // 2 ter) Montée de niveau : on attend que l'écran du jeu se
        // ferme, on rend les runes, puis retour au sous-menu.
        let ecran_du_jeu = match self.ecran {
            Ecran::MonteeDeNiveau => Some((MenuType::Soul, Ecran::Niveau)),
            Ecran::Forgeron => Some((MenuType::EnhanceShop, Ecran::Equipement)),
            // Un rayon d'un seul magasin (Talismans) revient directement au sous-menu.
            Ecran::Magasin(rayon, _) if rayon.magasins().len() == 1 => {
                Some((MenuType::RegularShop, sous_menu_du_rayon(rayon)))
            }
            Ecran::Reincarnatrice => Some((MenuType::UnkEvent136, Ecran::Esprits)),
            Ecran::Magasin(Rayon::Esprits, _) => Some((MenuType::RegularShop, Ecran::Esprits)),
            Ecran::Magasin(rayon, _) => Some((MenuType::RegularShop, Ecran::ListeMagasins(rayon))),
            _ => None,
        };
        if let Some((menu, retour)) = ecran_du_jeu {
            if self.menu_encore_ouvert(menu)? {
                if !self.ecran_du_jeu_vu {
                    self.ecran_du_jeu_vu = true;
                    noter("Écran du jeu ouvert.");
                }
            } else if !self.ecran_du_jeu_vu && self.attente_ouverture > 0 {
                // Pas encore ouvert : on attend encore un peu.
                self.attente_ouverture -= 1;
            } else {
                if !self.ecran_du_jeu_vu {
                    noter("Écran du jeu jamais vu ouvert, retour au menu.");
                }
                self.rendre_les_runes();
                self.aller_a(retour);
            }
            return Ok(());
        }

        // 3) Une liste : on attend le choix du joueur.
        if self.menu_encore_ouvert(MenuType::TalkList)? {
            return Ok(());
        }
        let choix: i32 = self.script.env(RESULTAT_LISTE)?.into();

        match self.ecran {
            Ecran::Principal => match choix {
                1 => self.aller_a(Ecran::ChoixBosses),
                2 => self.aller_a(Ecran::Vitesse),
                3 => self.aller_a(Ecran::Fioles),
                7 => self.aller_a(Ecran::Niveau),
                8 => self.aller_a(Ecran::Marchands),
                9 => self.aller_a(Ecran::EffetsArme),
                10 => self.aller_a(Ecran::Camera),
                _ => {
                    // « Partir » ou retour arrière : on ferme.
                    self.ecran = Ecran::Ferme;
                    noter("Menu de Melina fermé.");
                }
            },
            Ecran::ListeMagasins(rayon) => match choisi(choix, rayon.magasins().len()) {
                Some(rang) => self.aller_a(Ecran::Magasin(rayon, rang)),
                None => self.aller_a(sous_menu_du_rayon(rayon)),
            },
            Ecran::EffetsArme => match choix {
                1 => self.aller_a(Ecran::EffetsArmeMain(crate::effets_arme::Main::Droite)),
                2 => self.aller_a(Ecran::EffetsArmeMain(crate::effets_arme::Main::Gauche)),
                3 => self.aller_a(Ecran::Roulade),
                4 => self.aller_a(Ecran::Aura),
                5 => self.aller_a(Ecran::Course),
                6 => self.aller_a(Ecran::CourseEssais),
                7 => self.aller_a(Ecran::Galerie),
                8 => self.aller_a(Ecran::Impact),
                9 => self.aller_a(Ecran::ArmeFantome),
                10 => self.aller_a(Ecran::ArmeFantomeGauche),
                11 => self.aller_a(Ecran::ImpactGauche),
                12 => {
                    crate::reglages::reinitialiser(self);
                    self.aller_a(Ecran::EffetsArme);
                }
                _ => self.aller_a(Ecran::Principal),
            },
            Ecran::Course => {
                let nombre = crate::course::STYLES.len() as i32;
                if choix == 1 {
                    self.course.choisir(None);
                } else if (100..100 + nombre).contains(&choix) {
                    self.course.choisir(Some((choix - 100) as usize));
                } else if choix != LIGNE_RETOUR {
                    noter(&format!("Course : ligne inattendue {choix}."));
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::Camera => {
                if choix == 1 {
                    self.camera.choisir(0);
                } else if (101..=100 + crate::camera::RECUL_MAX as i32).contains(&choix) {
                    self.camera.choisir((choix - 100) as u32);
                }
                self.aller_a(Ecran::Principal);
            }
            Ecran::ArmeFantome | Ecran::ArmeFantomeGauche => {
                let nombre = crate::arme_fantome::FANTOMES.len() as i32;
                let fantome = if self.ecran == Ecran::ArmeFantome {
                    &mut self.arme_fantome
                } else {
                    &mut self.arme_fantome_gauche
                };
                if choix == 1 {
                    fantome.choisir(None);
                } else if (100..100 + nombre).contains(&choix) {
                    fantome.choisir(Some((choix - 100) as usize));
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::Impact | Ecran::ImpactGauche => {
                let nombre = crate::impact::IMPACTS.len() as i32;
                let impact = if self.ecran == Ecran::Impact {
                    &mut self.impact
                } else {
                    &mut self.impact_gauche
                };
                if choix == 1 {
                    impact.choisir(None);
                } else if (100..100 + nombre).contains(&choix) {
                    impact.choisir(Some((choix - 100) as usize));
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::Galerie => {
                use crate::galerie::{DEBUT_EXTENSION, DEBUT_INCANTATIONS};
                match choix {
                    1 => self.galerie.reprendre(),
                    2 => self.galerie.precedent(),
                    3 => self.galerie.aller_au_sort(DEBUT_INCANTATIONS),
                    4 => self.galerie.aller_au_sort(DEBUT_EXTENSION),
                    5 => self.galerie.aller_au_sort(0),
                    6 => self.galerie.arreter(),
                    _ => {}
                }
                if (1..=5).contains(&choix) {
                    // On ferme le menu pour lancer les sorts tout de suite.
                    self.ecran = Ecran::Ferme;
                    noter("Menu de Melina fermé (galerie des sorts).");
                } else {
                    self.aller_a(Ecran::EffetsArme);
                }
            }
            Ecran::CourseEssais => {
                // Les essais sont rangés après les styles (voir course::style).
                let premier = crate::course::STYLES.len();
                let nombre = crate::course::ESSAIS.len() as i32;
                if choix == 1 {
                    self.course.choisir(None);
                } else if (100..100 + nombre).contains(&choix) {
                    self.course.choisir(Some(premier + (choix - 100) as usize));
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::Aura => {
                let n = crate::aura::AURAS.len() as i32;
                if choix == 1 {
                    self.aura.choisir(None);
                } else if (2..=n + 1).contains(&choix) {
                    self.aura.choisir(Some((choix - 2) as usize));
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::Roulade => {
                match choix {
                    1 => self.roulade.choisir(crate::roulade::Style::Normale),
                    2 => self.roulade.choisir(crate::roulade::Style::Radagon),
                    3 => self.roulade.choisir(crate::roulade::Style::Limier),
                    4 => self.roulade.choisir(crate::roulade::Style::CouteauNoir),
                    5 => self.roulade.choisir(crate::roulade::Style::Revenant),
                    6 => self.roulade.choisir(crate::roulade::Style::ChevalierMort),
                    7 => self.roulade.choisir(crate::roulade::Style::TenebresEternelles),
                    8 => self.roulade.choisir(crate::roulade::Style::ChampThops),
                    9 => self.roulade.choisir(crate::roulade::Style::PleineLune),
                    10 => self.roulade.choisir(crate::roulade::Style::ProjectileGravitationnel),
                    11 => self.roulade.choisir(crate::roulade::Style::TirMagma),
                    12 => self.roulade.choisir(crate::roulade::Style::HalosRadagon),
                    13 => self.roulade.choisir(crate::roulade::Style::FlammeNoire),
                    14 => self.roulade.choisir(crate::roulade::Style::SerpentFeu),
                    15 => self.roulade.choisir(crate::roulade::Style::PierreGurranq),
                    16 => self.roulade.choisir(crate::roulade::Style::DisqueLumiere),
                    17 => self.roulade.choisir(crate::roulade::Style::Foudroiement),
                    _ => {}
                }
                self.aller_a(Ecran::EffetsArme);
            }
            Ecran::EffetsArmeMain(main) => match choix {
                1 | 2 => self.aller_a(Ecran::EffetsArmeChoix(crate::effets_arme::Emplacement {
                    main,
                    genre: if choix == 1 {
                        crate::effets_arme::Genre::Effet
                    } else {
                        crate::effets_arme::Genre::Trainee
                    },
                })),
                _ => self.aller_a(Ecran::EffetsArme),
            },
            Ecran::EffetsArmeChoix(emplacement) => {
                let n = crate::effets_arme::EFFETS.len() as i32;
                if choix == 1 {
                    self.effets_arme.choisir(emplacement, None);
                } else if (2..=n + 1).contains(&choix) {
                    self.effets_arme.choisir(emplacement, Some((choix - 2) as usize));
                }
                self.aller_a(Ecran::EffetsArmeMain(emplacement.main));
            }
            Ecran::Esprits => match choix {
                LIGNE_AMELIORER_ESPRITS => self.aller_a(Ecran::Reincarnatrice),
                _ => match choisi(choix, Rayon::Esprits.magasins().len()) {
                    Some(rang) => self.aller_a(Ecran::Magasin(Rayon::Esprits, rang)),
                    None => self.aller_a(Ecran::Marchands),
                },
            },
            Ecran::ChoixBosses => match choix {
                1 => self.aller_a(Ecran::Bosses),
                2 => self.aller_a(Ecran::BossesDlc),
                3 => {
                    // Les bosses réapparaissent au prochain chargement de leur zone (la
                    // téléportation depuis le menu en fait un). On revient au sous-menu.
                    crate::deblocage::faire_revivre_les_bosses();
                    self.aller_a(Ecran::ChoixBosses);
                }
                _ => self.aller_a(Ecran::Principal),
            },
            Ecran::Niveau => match choix {
                1 => self.aller_a(Ecran::MonteeDeNiveau),
                2 => self.aller_a(Ecran::Renaissance),
                3 => self.aller_a(Ecran::Magasin(Rayon::Ameliorations, 0)),
                _ => self.aller_a(Ecran::Principal),
            },
            Ecran::Marchands => match choix {
                1 => self.aller_a(Ecran::Equipement),
                2 => self.aller_a(Ecran::Magasin(Rayon::Talismans, 0)),
                3 => self.aller_a(Ecran::ListeMagasins(Rayon::Sorts)),
                4 => self.aller_a(Ecran::Magasin(Rayon::Materiaux, 0)),
                5 => self.aller_a(Ecran::Esprits),
                _ => self.aller_a(Ecran::Principal),
            },
            Ecran::Equipement => match choix {
                1 => self.aller_a(Ecran::ListeMagasins(Rayon::Armes)),
                2 => self.aller_a(Ecran::ListeMagasins(Rayon::Armures)),
                3 => self.aller_a(Ecran::Forgeron),
                4 => self.aller_a(Ecran::DonnerPierres),
                5 => self.aller_a(Ecran::Magasin(Rayon::Cendres, 0)),
                6 => self.aller_a(Ecran::Magasin(Rayon::Munitions, 0)),
                _ => self.aller_a(Ecran::Marchands),
            },
            Ecran::Vitesse => {
                if (vitesse::NORMALE as i32..=vitesse::MAX as i32).contains(&choix) {
                    vitesse.definir(choix as u32);
                }
                self.aller_a(Ecran::Principal);
            }
            Ecran::Bosses => {
                if choix >= 1 && (choix as usize) <= BOSSES.len() {
                    // Boss choisi : on ferme le menu, la téléportation suit.
                    let (_, grace) = BOSSES[choix as usize - 1];
                    self.grace_choisie = Some(grace);
                    self.ecran = Ecran::Ferme;
                    noter(&format!("Boss n° {choix} choisi (grâce {grace})."));
                } else {
                    self.aller_a(Ecran::ChoixBosses);
                }
            }
            Ecran::BossesDlc => {
                if choix >= 1 && (choix as usize) <= BOSSES_DLC.len() {
                    let (_, grace) = BOSSES_DLC[choix as usize - 1];
                    self.grace_choisie = Some(grace);
                    self.ecran = Ecran::Ferme;
                    noter(&format!("Boss du DLC n° {choix} choisi (grâce {grace})."));
                } else {
                    self.aller_a(Ecran::ChoixBosses);
                }
            }
            _ => self.aller_a(Ecran::Principal),
        }
        Ok(())
    }

    /// Affiche une liste de lignes (numéro de ligne, numéro de texte).
    fn afficher(&mut self, lignes: &[(i32, i32)]) -> Result<(), eldenring::cs::EzStateInvokeError> {
        self.script.event(VIDER_LISTE)?;
        for &(ligne, texte) in lignes {
            self.script.event((
                AJOUTER_LIGNE,
                [EzStateValue::Int32(ligne), EzStateValue::Int32(texte), EzStateValue::Int32(-1)],
            ))?;
        }
        self.script.event((AFFICHER_LISTE, [EzStateValue::Int32(1)]))?;
        Ok(())
    }

    /// Donne la pierre suivante (une image sur IMAGES_ENTRE_DEUX_PIERRES). Une fois toutes
    /// données, vérifie que le joueur les a et revient au sous-menu « Marchands ».
    fn donner_les_pierres(&mut self) -> Result<(), eldenring::cs::EzStateInvokeError> {
        use crate::marchands::{LOT_PIERRES, PIERRES_DE_FORGE, preparer_lot_pierre};
        if self.attente_pierre > 0 {
            self.attente_pierre -= 1;
            return Ok(());
        }
        if let Some(&pierre) = PIERRES_DE_FORGE.get(self.pierre_suivante) {
            if preparer_lot_pierre(pierre) {
                self.script.event((DONNER_LOT, [EzStateValue::Int32(LOT_PIERRES as i32)]))?;
            }
            self.pierre_suivante += 1;
            self.attente_pierre = IMAGES_ENTRE_DEUX_PIERRES;
            return Ok(());
        }
        // Toutes données : on attend la fin des messages « objet obtenu », puis on vérifie.
        let dialogue: i32 = self.script.env((DIALOGUE_OUVERT, [EzStateValue::Int32(0)]))?.into();
        if dialogue != 0 {
            return Ok(());
        }
        let mut manquantes = Vec::new();
        for &pierre in PIERRES_DE_FORGE.iter() {
            if !self.a_l_objet(pierre)? {
                manquantes.push(pierre);
            }
        }
        if manquantes.is_empty() {
            noter("Pierres de forge : toutes données.");
        } else {
            noter(&format!("Pierres de forge : il manque {manquantes:?}."));
        }
        self.aller_a(Ecran::Equipement);
        Ok(())
    }

    /// Donne le prochain objet de départ qui manque (la différence avec le nombre voulu).
    /// Quand il n'en manque plus, revient au menu principal.
    fn donner_les_objets_de_depart(&mut self) -> Result<(), eldenring::cs::EzStateInvokeError> {
        use crate::marchands::{LOT_OBJET, LOT_PIERRES, nombre_d_objets, preparer_lot};
        if self.attente_pierre > 0 {
            self.attente_pierre -= 1;
            return Ok(());
        }
        // Fioles de départ : sur une partie neuve, le joueur part à la Table ronde sans passer
        // par l'évènement du début du jeu qui les donne. Sans elles (drapeau 60000 éteint), le
        // jeu refuse de répartir ou d'améliorer les fioles. Melina donne le même lot que le jeu
        // (2000 : 3 fioles de soin, suivi du lot 2001 : 1 fiole de mana), qui allume 60000.
        if self.fioles_a_donner {
            self.fioles_a_donner = false;
            noter("Fioles de départ : Melina donne les fioles (lot 2000).");
            self.script.event((DONNER_LOT, [EzStateValue::Int32(LOT_FIOLES_DE_DEPART)]))?;
            self.attente_pierre = IMAGES_ENTRE_DEUX_PIERRES;
            return Ok(());
        }
        while let Some(&(objet, voulu, nom)) = OBJETS_DE_DEPART.get(self.pierre_suivante) {
            self.pierre_suivante += 1;
            let Some(possede) = nombre_d_objets(objet) else {
                continue;
            };
            if possede >= voulu as u32 {
                continue;
            }
            let manque = voulu - possede as u8;
            if preparer_lot(objet as i32, LOT_OBJET, manque) {
                noter(&format!("Objets de départ : {nom}, {possede} sur {voulu}, Melina en donne {manque}."));
                self.script.event((DONNER_LOT, [EzStateValue::Int32(LOT_PIERRES as i32)]))?;
                self.attente_pierre = IMAGES_ENTRE_DEUX_PIERRES;
                return Ok(());
            }
        }
        // Tout est donné : on attend la fin des messages « objet obtenu ».
        let dialogue: i32 = self.script.env((DIALOGUE_OUVERT, [EzStateValue::Int32(0)]))?.into();
        if dialogue == 0 {
            self.aller_a(Ecran::Principal);
        }
        Ok(())
    }

    /// Vrai si le joueur a au moins un exemplaire de cet objet (EquipParamGoods).
    fn a_l_objet(&mut self, objet: i32) -> Result<bool, eldenring::cs::EzStateInvokeError> {
        let a: i32 = self
            .script
            .env((A_L_OBJET, [EzStateValue::Int32(CATEGORIE_OBJET), EzStateValue::Int32(objet)]))?
            .into();
        Ok(a == 1)
    }

    /// Vrai si le joueur a au moins une larme larvaire.
    fn a_une_larme_larvaire(&mut self) -> Result<bool, eldenring::cs::EzStateInvokeError> {
        let a: i32 = self
            .script
            .env((
                A_L_OBJET,
                [EzStateValue::Int32(CATEGORIE_OBJET), EzStateValue::Int32(crate::deblocage::LARME_LARVAIRE)],
            ))?
            .into();
        Ok(a == 1)
    }

    fn ouvrir_renaissance(&mut self) -> Result<(), eldenring::cs::EzStateInvokeError> {
        noter("Menu de renaissance ouvert.");
        self.script.event(OUVRIR_RENAISSANCE)?;
        Ok(())
    }

    /// Vrai tant que le menu donné est ouvert (et qu'aucune boîte de dialogue ne le recouvre).
    fn menu_encore_ouvert(&mut self, menu: MenuType) -> Result<bool, eldenring::cs::EzStateInvokeError> {
        let ouvert: i32 = self
            .script
            .env((MENU_OUVERT, [EzStateValue::Int32(menu as i32), EzStateValue::Int32(0)]))?
            .into();
        let dialogue: i32 = self.script.env((DIALOGUE_OUVERT, [EzStateValue::Int32(0)]))?.into();
        Ok(ouvert == 1 && dialogue == 0)
    }
}

/// Le sous-menu où revenir en quittant un rayon : « DLC » ou « Marchands ».
fn sous_menu_du_rayon(rayon: Rayon) -> Ecran {
    if rayon == Rayon::Ameliorations {
        Ecran::Niveau
    } else if rayon.dans_equipement() {
        Ecran::Equipement
    } else {
        Ecran::Marchands
    }
}

/// Met le total des fioles (soin + mana) à 14, en gardant le nombre de fioles de mana,
/// pour que le menu « Répartir les fioles » du jeu propose bien 14 fioles.
/// Lot d'objets du jeu qui donne les fioles du début de partie (ItemLotParam_map 2000 :
/// 3 Fioles de larmes écarlates, et la ligne 2001 qui suit : 1 Fiole de larmes azurées).
/// Il allume le drapeau 60000 (« fioles obtenues »).
const LOT_FIOLES_DE_DEPART: i32 = 2000;
/// Drapeau « Fioles de larmes écarlates / azurées obtenues » (mesure F11 du 08/11/2026 :
/// allumé par l'évènement du début du jeu ; noms : Smithbox, EventFlags.json).
const DRAPEAU_FIOLES_OBTENUES: u32 = 60000;

/// Vrai si le joueur a déjà reçu les fioles du début de partie (ou si on ne peut pas lire
/// les drapeaux : dans le doute, on ne donne rien).
fn fioles_obtenues() -> bool {
    match unsafe { eldenring::cs::CSEventFlagMan::instance() } {
        Ok(drapeaux) => drapeaux.virtual_memory_flag.get_flag(DRAPEAU_FIOLES_OBTENUES),
        Err(_) => true,
    }
}

fn regler_total_des_fioles() {
    let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
        return;
    };
    let Some(joueur) = monde.main_player.as_ref() else {
        return;
    };
    let donnees = unsafe { &mut *joueur.player_game_data.as_ptr() };
    let mana = donnees.max_fp_flask.min(FIOLES_TOTAL);
    donnees.max_fp_flask = mana;
    donnees.max_hp_flask = FIOLES_TOTAL - mana;
    noter(&format!(
        "Fioles : total réglé à {FIOLES_TOTAL} ({} soin, {} mana) avant la répartition.",
        donnees.max_hp_flask, donnees.max_fp_flask
    ));
}
