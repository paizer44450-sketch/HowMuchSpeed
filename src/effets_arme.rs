//! Effets d'arme : le visuel d'une graisse (feu, magie…) sur l'arme de la main droite ou
//! de la main gauche, sans ses dégâts et sans limite de temps.
//!
//! Mesure (outil F11, 07/11/2026) : la graisse magique applique l'effet 3170 (visuel 64,
//! info d'état 64, catégorie 162 = buff d'arme main droite, 85 de dégâts magiques, 60 s).
//! Les graisses ont une ligne « main droite » (catégorie 162) et une ligne « main gauche »
//! (catégorie 163). Pour chaque main, on recopie en mémoire la ligne de la graisse choisie
//! dans une ligne d'effet inutilisée du jeu (« Area Scaling - (Unused) » : 7210 pour la
//! droite, 7220 pour la gauche), on enlève les dégâts (puissance d'attaque à 0, pas d'effet
//! au coup) et on la rend permanente (durée -1). L'effet est remis après chaque chargement
//! (mort comprise). Marche aussi sur les armes doubles (griffes…).
//!
//! Le jeu ne dessine qu'une lueur d'enchantement par arme (essai « Effet 1 / Effet 2 »,
//! 0.37.0 : le dernier effet posé remplace l'autre à l'écran). Pour combiner, chaque main
//! a donc un « Effet » (la lueur) et une « Traînée » (la couleur de la trace des coups,
//! `trace_sfx_id_offset_type` du visuel) prise sur un autre effet : les deux sont réunis
//! dans un seul visuel. Une traînée seule (sans effet) garde la trace sans la lueur.
//!
//! Le jeu ne dessine qu'une lueur de chaque côté (type de visuel 1 droite, 2 gauche) pour
//! tout le personnage : un « Effet 2 » (deuxième lueur sur la même arme, 0.39.x) obligeait
//! à jongler entre les mains, retiré le 07/11/2026. En revanche, arme tenue à deux mains,
//! le jeu pose de lui-même la lueur de gauche sur l'arme tenue : les deux effets se
//! combinent. Au retour à une main, on repose les effets pour que chacun revienne sur
//! son arme (sinon la lueur de gauche restait sur l'arme de droite).

use eldenring::cs::{
    ChrInsExt, EquipParamWeapon, SoloParamRepository, SpEffectParam, SpEffectVfxParam, WorldChrMan,
};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Les effets proposés : (texte du menu, graisse main droite, graisse main gauche, effet
/// visuel continu à mettre à la place de celui d'origine (0 = garder), nom).
/// Types d'arme (EquipParamWeapon, wepType) des boucliers : petit (65), moyen (67) et
/// pavois (69).
const TYPES_BOUCLIERS: [u16; 3] = [65, 67, 69];

/// Les boucliers ne prennent pas tous les buffs d'arme dans le jeu (case « isEnhance »
/// éteinte : pas de graisse possible), donc pas nos effets visuels non plus. On l'allume en
/// mémoire pour eux, au chargement de la partie. Effet de bord : les vraies graisses
/// s'utilisent aussi sur ces boucliers.
///
/// Essais du 08/11/2026 : ni les bâtons (57) et sceaux (61), ni les boucliers n'affichent
/// d'effet, même avec la case allumée (leurs modèles n'ont pas les points d'attache de la
/// lueur). La fonction n'est plus appelée ; gardée pour mémoire.
#[allow(dead_code)]
pub fn autoriser_les_boucliers() {
    let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
        return;
    };
    let mut nombre = 0;
    for (_, arme) in parametres.rows_mut::<EquipParamWeapon>() {
        if TYPES_BOUCLIERS.contains(&arme.wep_type()) && !arme.is_enhance() {
            arme.set_is_enhance(true);
            nombre += 1;
        }
    }
    noter(&format!("Effets d'arme : {nombre} boucliers acceptent maintenant les effets."));
}

pub const EFFETS: [(i32, u32, u32, i32, &str); 20] = [
    (69000067, 3160, 3162, 0, "Feu"),
    (69000068, 3170, 3172, 0, "Magie"),
    (69000069, 3165, 3167, 0, "Foudre"),
    (69000070, 3185, 3187, 0, "Sacré"),
    (69000071, 3140, 3144, 0, "Givre"),
    (69000072, 3190, 3194, 0, "Sang"),
    (69000073, 3175, 3179, 0, "Poison"),
    (69000074, 3310, 3314, 0, "Pourriture"),
    (69000077, 3150, 3154, 0, "Sommeil"),
    (69000078, 3260, 3262, 0, "Plaie draconique"),
    // Graisses du DLC (mesurées au F11 : même fonctionnement, un seul effet par graisse).
    // Eau bénite, graisse dorée et communion draconique retirées : même visuel qu'un
    // effet déjà proposé (vu en jeu le 07/11/2026).
    (69000080, 102310, 102314, 0, "Sommeil éternel"),
    (69000081, 20501400, 20501402, 0, "Flamme de Messmer"),
    (69000082, 20501410, 20501412, 0, "Foudre draconique"),
    (69000083, 20501420, 20501422, 0, "Magie royale"),
    // Sorts d'enchantement (mesurés au F11, 07/11/2026). Ils n'existent que pour la main
    // droite (0 = pas de version gauche) : pour la gauche, on adapte la version droite
    // (catégorie 163, arme de gauche).
    (69000086, 1466000, 0, 0, "Lame invisible"),
    (69000087, 1626000, 0, 0, "Lame de flamme noire"),
    (69000088, 1632000, 0, 0, "Lame de feu sanguin"),
    // Foudre draconique de Vyke (mesure F11 du 08/11/2026) : effet 1697000, visuel
    // 1697000 (foudre rouge le long de la lame, visuel continu 303151, côté droit).
    (69000508, 1697000, 0, 0, "Foudre draconique de Vyke"),
    // Cendres de guerre (mesures F11 du 07/11/2026) : buffs d'arme avec un visuel
    // (catégorie 162, pas de version gauche).
    (69000090, 1791, 0, 0, "Flamboiement mortel"),
    (69000091, 1821, 0, 0, "Lame rocheuse"),
    // Retirés le 07/11/2026 : Clair de lune (1721, visuel 8610 : lame fantôme entière
    // faite pour l'Épée de clair de lune, posée au pommeau des autres armes) et flamme de
    // l'Épée nocturne (visuel résident 490040 de l'arme 9500000 : ne marche pas).
    // Feu de la dague de Melina (effet 18699, visuel continu 600901 attaché au point 220
    // de son modèle) : abandonné le 07/11/2026 après 3 essais (mal placé, visible
    // seulement sur les dagues, ou rien du tout) : effet fait pour son modèle, pas pour
    // une arme.
];

/// Visuel « main gauche » qui sert de modèle (graisse magique, main gauche) : on lui
/// prend le côté de l'arme (type d'effet, points d'attache sur la lame).
const VISUEL_MODELE_GAUCHE: u32 = 5115;

/// Recopie le visuel de l'effet de base dans la ligne de visuel `libre` et le retouche :
/// type d'effet (côté droit 1 / gauche 2) pris sur le visuel `type_de`, points d'attache
/// sur la lame pris sur le visuel `points_de`, visuel continu remplacé si `visuel_continu`
/// n'est pas 0, traînée remplacée si `trainee`, lueur enlevée si `sans_lueur`. Renvoie le
/// numéro du visuel prêt.
#[allow(clippy::too_many_arguments)]
fn preparer_visuel(
    parametres: &mut SoloParamRepository,
    libre: u32,
    visuel_origine: i32,
    type_de: Option<u32>,
    points_de: Option<u32>,
    visuel_continu: i32,
    trainee: Option<u8>,
    sans_lueur: bool,
) -> Option<i32> {
    let modele_type = match type_de {
        Some(v) => Some(parametres.get::<SpEffectVfxParam>(v)?.clone()),
        None => None,
    };
    let modele_points = match points_de {
        Some(v) => Some(parametres.get::<SpEffectVfxParam>(v)?.clone()),
        None => None,
    };
    let origine = parametres.get::<SpEffectVfxParam>(visuel_origine as u32)?.clone();
    let cible = parametres.get_mut::<SpEffectVfxParam>(libre)?;
    *cible = origine;
    // Pas de teinte sur l'arme (« soul param » de l'enchantement : 1 magie, 10 sacré,
    // 6 givre, 17 flamme noire… ; 0 chez le feu, la foudre, le sang) : le joueur la trouve
    // moche (07/11/2026).
    cible.set_soul_param_id_for_wep_enchant(0);
    if visuel_continu != 0 {
        cible.set_midst_sfx_id(visuel_continu);
        cible.set_midst_se_id(-1); // pas le son de la flamme noire
    }
    if let Some(t) = trainee {
        cible.set_trace_sfx_id_offset_type(t);
    }
    if sans_lueur {
        cible.set_midst_sfx_id(-1);
        cible.set_midst_se_id(-1);
    }
    if let Some(m) = modele_type {
        cible.set_effect_type(m.effect_type());
        cible.set_force_deceased_type(m.force_deceased_type());
    }
    if let Some(m) = modele_points {
        cible.set_enchant_start_dmy_id_0(m.enchant_start_dmy_id_0());
        cible.set_enchant_end_dmy_id_0(m.enchant_end_dmy_id_0());
        cible.set_enchant_start_dmy_id_1(m.enchant_start_dmy_id_1());
        cible.set_enchant_end_dmy_id_1(m.enchant_end_dmy_id_1());
        cible.set_enchant_start_dmy_id_2(m.enchant_start_dmy_id_2());
        cible.set_enchant_end_dmy_id_2(m.enchant_end_dmy_id_2());
        cible.set_enchant_start_dmy_id_3(m.enchant_start_dmy_id_3());
        cible.set_enchant_end_dmy_id_3(m.enchant_end_dmy_id_3());
        cible.set_enchant_start_dmy_id_4(m.enchant_start_dmy_id_4());
        cible.set_enchant_end_dmy_id_4(m.enchant_end_dmy_id_4());
        cible.set_enchant_start_dmy_id_5(m.enchant_start_dmy_id_5());
        cible.set_enchant_end_dmy_id_5(m.enchant_end_dmy_id_5());
        cible.set_enchant_start_dmy_id_6(m.enchant_start_dmy_id_6());
        cible.set_enchant_end_dmy_id_6(m.enchant_end_dmy_id_6());
        cible.set_enchant_start_dmy_id_7(m.enchant_start_dmy_id_7());
        cible.set_enchant_end_dmy_id_7(m.enchant_end_dmy_id_7());
    }
    Some(libre as i32)
}

/// Rend une ligne d'effet recopiée purement visuelle : permanente, sans dégâts, sans effet
/// au coup ni bonus contre les faiblesses.
fn rendre_visuelle(ligne: &mut eldenring::param::SP_EFFECT_PARAM_ST) {
    ligne.set_effect_endurance(-1.0);
    ligne.set_physics_attack_power(0);
    ligne.set_magic_attack_power(0);
    ligne.set_fire_attack_power(0);
    ligne.set_thunder_attack_power(0);
    ligne.set_dark_attack_power(0);
    ligne.set_atk_occurrence_sp_effect_id(-1);
    ligne.set_cycle_occurrence_sp_effect_id(-1);
    ligne.set_weak_dmg_rate_a(1.0);
    ligne.set_weak_dmg_rate_b(1.0);
    ligne.set_weak_dmg_rate_c(1.0);
    ligne.set_weak_dmg_rate_d(1.0);
    ligne.set_weak_dmg_rate_e(1.0);
    ligne.set_weak_dmg_rate_f(1.0);
}

/// Les deux mains.
#[derive(Clone, Copy, PartialEq)]
pub enum Main {
    Droite,
    Gauche,
}

impl Main {
    fn rang(self) -> usize {
        match self {
            Main::Droite => 0,
            Main::Gauche => 1,
        }
    }

    /// Ligne d'effet inutilisée du jeu (« Area Scaling - (Unused) ») réservée à cette main.
    fn ligne_libre(self) -> u32 {
        [7210, 7220][self.rang()]
    }

    /// Ligne de visuel inutilisée du jeu (présente une seule fois dans regulation.bin,
    /// référencée par aucun effet) pour le visuel retouché de cette main.
    fn visuel_libre(self) -> u32 {
        [20008511, 1753000][self.rang()]
    }

    fn nom(self) -> &'static str {
        ["main droite", "main gauche"][self.rang()]
    }
}

/// Ce qu'on choisit pour une main : la lueur (effet) ou la couleur de la trace des coups.
#[derive(Clone, Copy, PartialEq)]
pub enum Genre {
    Effet,
    Trainee,
}

/// Un emplacement du menu : une main et son effet ou sa traînée.
#[derive(Clone, Copy, PartialEq)]
pub struct Emplacement {
    pub main: Main,
    pub genre: Genre,
}

#[derive(Clone, Copy)]
struct Choix {
    /// Rang dans EFFETS de l'effet (lueur), ou None.
    effet: Option<usize>,
    /// Rang dans EFFETS de l'effet dont on prend la traînée, ou None (celle de l'effet).
    trainee: Option<usize>,
}

impl Choix {
    fn vide(self) -> bool {
        self.effet.is_none() && self.trainee.is_none()
    }
}

/// Effets de posture « arme tenue à deux mains » (mesure F11 du 07/11/2026 : 100621 en
/// prenant l'arme de droite à deux mains ; 100620 est toujours là). 100622 et 100623 sont
/// pris aussi au cas où (arme de gauche à deux mains).
const POSTURES_DEUX_MAINS: [i32; 3] = [100621, 100622, 100623];

pub struct EffetsArme {
    /// Choix de chaque main (droite, gauche).
    choix: [Choix; 2],
    /// Arme tenue à deux mains à l'image précédente.
    deux_mains: bool,
}

/// Ligne d'effet (graisse ou sort) d'un rang d'EFFETS pour une main, et s'il faut adapter
/// son visuel à la main gauche (sorts : pas de version gauche).
fn ligne_de(main: Main, rang: usize) -> (u32, bool) {
    let (_, droite, gauche, _, _) = EFFETS[rang];
    let adapter_a_gauche = main == Main::Gauche && gauche == 0;
    let graisse = if main == Main::Droite || adapter_a_gauche { droite } else { gauche };
    (graisse, adapter_a_gauche)
}

impl EffetsArme {
    /// Choix actuel d'une main : (effet, traînée), rangs dans EFFETS (lu par reglages.rs).
    pub fn choix_main(&self, main: Main) -> (Option<usize>, Option<usize>) {
        let c = self.choix[main.rang()];
        (c.effet, c.trainee)
    }

    pub fn new() -> Self {
        Self { choix: [Choix { effet: None, trainee: None }; 2], deux_mains: false }
    }

    /// Prépare la ligne libre de la main : l'effet (ou, sans effet, celui de la traînée)
    /// recopié sans dégâts et permanent, avec un visuel qui réunit lueur et traînée.
    fn preparer_la_ligne(main: Main, choix: Choix) -> bool {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(base) = choix.effet.or(choix.trainee) else {
            return false;
        };
        let (_, _, _, visuel_continu, nom) = EFFETS[base];
        let (graisse, adapter_a_gauche) = ligne_de(main, base);
        let Some(source) = parametres.get::<SpEffectParam>(graisse).cloned() else {
            noter(&format!("Effets d'arme : ligne de la graisse {graisse} ({nom}) introuvable."));
            return false;
        };
        // Couleur de la traînée : celle du visuel de l'effet choisi pour la traînée.
        let trainee = match choix.trainee {
            Some(r) => {
                let (ligne_t, _) = ligne_de(main, r);
                parametres
                    .get::<SpEffectParam>(ligne_t)
                    .map(|e| e.vfx_id())
                    .and_then(|v| parametres.get::<SpEffectVfxParam>(v as u32))
                    .map(|v| v.trace_sfx_id_offset_type())
            }
            None => None,
        };
        let modele_gauche = adapter_a_gauche.then_some(VISUEL_MODELE_GAUCHE);
        let Some(visuel) = preparer_visuel(
            parametres,
            main.visuel_libre(),
            source.vfx_id(),
            modele_gauche,
            modele_gauche,
            visuel_continu,
            trainee,
            choix.effet.is_none(),
        ) else {
            noter(&format!("Effets d'arme : visuel de {nom} impossible à préparer."));
            return false;
        };
        let libre = main.ligne_libre();
        let Some(ligne) = parametres.get_mut::<SpEffectParam>(libre) else {
            noter(&format!("Effets d'arme : ligne libre {libre} introuvable."));
            return false;
        };
        *ligne = source;
        rendre_visuelle(ligne);
        if adapter_a_gauche {
            // Mêmes réglages que les graisses de la main gauche.
            ligne.set_sp_category(163);
            ligne.set_wep_param_change(2);
        }
        ligne.set_vfx_id(visuel);
        true
    }

    /// Choisit l'effet ou la traînée d'une main (None = aucun) et applique tout de suite.
    pub fn choisir(&mut self, emplacement: Emplacement, rang: Option<usize>) {
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        let main = emplacement.main;
        let mut choix = self.choix[main.rang()];
        match emplacement.genre {
            Genre::Effet => choix.effet = rang,
            Genre::Trainee => choix.trainee = rang,
        }
        let ligne = main.ligne_libre() as i32;
        // On enlève d'abord l'ancien effet, avant de réécrire la ligne.
        joueur.chr_ins.remove_speffect(ligne);
        self.choix[main.rang()] = Choix { effet: None, trainee: None };
        let nom_de = |r: Option<usize>| r.map(|r| EFFETS[r].4).unwrap_or("aucun");
        if choix.vide() {
            noter(&format!("Effets d'arme ({}) : aucun.", main.nom()));
            return;
        }
        if !Self::preparer_la_ligne(main, choix) {
            return;
        }
        joueur.apply_speffect(ligne, true);
        self.choix[main.rang()] = choix;
        noter(&format!(
            "Effets d'arme ({}) : effet {}, traînée {}.",
            main.nom(),
            nom_de(choix.effet),
            nom_de(choix.trainee)
        ));
    }

    /// À appeler à chaque image : remet les effets choisis s'ils ont disparu (mort,
    /// chargement), et les repose au retour à une main.
    pub fn mettre_a_jour(&mut self) {
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        let deux_mains = joueur
            .chr_ins
            .special_effect
            .entries()
            .any(|e| POSTURES_DEUX_MAINS.contains(&e.param_id));
        let retour_a_une_main = self.deux_mains && !deux_mains;
        self.deux_mains = deux_mains;
        for main in [Main::Droite, Main::Gauche] {
            if self.choix[main.rang()].vide() {
                continue;
            }
            let ligne = main.ligne_libre() as i32;
            if retour_a_une_main {
                // Enlevé ici, remis à l'image suivante : la lueur se rattache alors à
                // l'arme de sa main.
                joueur.chr_ins.remove_speffect(ligne);
                continue;
            }
            if !joueur.chr_ins.special_effect.entries().any(|e| e.param_id == ligne) {
                joueur.apply_speffect(ligne, true);
            }
        }
    }
}
