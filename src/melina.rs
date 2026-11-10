//! Melina, debout près de la table de la Table ronde.
//!
//! Elle est placée dans le fichier de la carte du mod (Smithbox) :
//!   modengine\mod\map\mapstudio\m11_10_00_00.msb.dcx
//!   personnage « c2180_9000 », modèle c2180, PNJ 21801034, entité 11105950, sans IA.
//! Le jeu la charge et l'affiche lui-même, comme n'importe quel PNJ.
//! Ses groupes d'affichage et de dessin (DisplayGroups / DrawGroups) sont tous allumés :
//! à zéro, le jeu ne la dessinait jamais (carte v7).
//! La DLL se contente de la retrouver (par son numéro d'entité) pour savoir si le joueur
//! est assez près pour lui parler.

use std::ptr::NonNull;

use std::sync::atomic::{AtomicBool, Ordering};

use eldenring::cs::{
    ChrActivateConditionParam, ChrIns, NpcParam, SoloParamRepository, WorldChrMan,
};
use eldenring::position::HavokPosition;
use eldenring::rotation::Quaternion;
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Numéro d'entité de Melina dans la carte de la Table ronde.
const ENTITE_MELINA: u32 = 11105950;

/// Ligne des données de PNJ (NpcParam) utilisée par Melina dans la carte :
/// la Melina qu'on invoque contre Morgott.
const PNJ_MELINA: u32 = 21801034;

/// Vrai une fois les données de PNJ de Melina réglées.
static DONNEES_PNJ_REGLEES: AtomicBool = AtomicBool::new(false);

/// Lève les conditions d'apparition de Melina dans ses données de PNJ.
///
/// Le journal (F10) montrait qu'elle était chargée mais jamais activée ni dessinée :
/// la vérification « météo et heure » échouait. Ces conditions viennent de sa ligne de
/// données de PNJ (chrActivateConditionParamId) : on la met à 0, « toujours ».
/// Les données du jeu ne sont prêtes qu'un peu après le démarrage : on réessaie à chaque
/// image jusqu'à ce que ce soit fait.
pub fn regler_donnees_pnj() {
    if DONNEES_PNJ_REGLEES.load(Ordering::Relaxed) {
        return;
    }
    let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
        return;
    };
    let Some(ligne) = parametres.get_mut::<NpcParam>(PNJ_MELINA) else {
        return;
    };
    noter(&format!(
        "Données de PNJ {PNJ_MELINA} : conditions d'apparition {}, activation désactivée (ouvert {}, ancien {}), distance de désactivation {}. Conditions mises à 0 (toujours).",
        ligne.chr_activate_condition_param_id(),
        ligne.disable_activate_open_xb1(),
        ligne.disable_activate_legacy_xb1(),
        ligne.deactivate_dist()
    ));
    ligne.set_chr_activate_condition_param_id(0);
    ligne.set_disable_activate_open_xb1(false);
    ligne.set_disable_activate_legacy_xb1(false);

    // La ligne 0 des conditions d'apparition (« toujours ») : on note ce qu'elle contient.
    match parametres.get::<ChrActivateConditionParam>(0) {
        Some(c) => noter(&format!(
            "Conditions d'apparition 0 : soleil {}, ciel clair {}, nuageux léger {}, nuageux {}, pluie {}, forte pluie {}, orage {}, neige {}, brouillard {} ; de {}h{:02} à {}h{:02}.",
            c.weather_sunny(), c.weather_clear_sky(), c.weather_weak_cloudy(), c.weather_cloudy(),
            c.weather_rain(), c.weather_heavy_rain(), c.weather_storm(), c.weather_snow(),
            c.weather_fog(), c.time_start_hour(), c.time_start_min(), c.time_end_hour(),
            c.time_end_min()
        )),
        None => noter("Conditions d'apparition 0 : ligne absente des données du jeu."),
    }
    DONNEES_PNJ_REGLEES.store(true, Ordering::Relaxed);
}

/// Angle autour de l'axe vertical (en degrés) d'une orientation du jeu (quaternion x, y, z, w).
pub fn angle_vertical(q: Quaternion) -> f32 {
    let (x, y, z, w) = (q.0, q.1, q.2, q.3);
    let lacet = (2.0 * (w * y + x * z)).atan2(1.0 - 2.0 * (x * x + y * y));
    lacet.to_degrees()
}

/// Distance à laquelle l'invite « Parler » apparaît (en mètres).
const DISTANCE_POUR_PARLER: f32 = 2.5;

pub struct Melina {
    /// Melina une fois retrouvée (None tant qu'on ne l'a pas trouvée).
    personnage: Option<NonNull<ChrIns>>,
}

impl Melina {
    pub fn new() -> Self {
        Self { personnage: None }
    }

    /// À appeler à chaque écran de chargement : Melina disparaît avec l'ancienne carte.
    pub fn oublier(&mut self) {
        self.personnage = None;
    }

    /// Le personnage de Melina, s'il a été retrouvé.
    pub fn personnage(&self) -> Option<&mut ChrIns> {
        self.personnage.map(|mut p| unsafe { p.as_mut() })
    }

    /// Note dans le journal où est Melina (touche F10).
    pub fn noter_etat(&self) {
        match self.personnage() {
            Some(melina) => {
                let p = melina.modules.physics.position;
                noter(&format!(
                    "Melina : position ({:.2}, {:.2}, {:.2}), orientation {:.1}°, carte {}, transparence base {:.2} (évolution {:.2}), teinte {:.2}, images clés {:.2}, camouflage {:.2}.",
                    p.0, p.1, p.2, angle_vertical(melina.modules.physics.orientation), melina.block_id,
                    melina.base_transparency, melina.base_transparency_modifier,
                    melina.tint_alpha_multiplier, melina.opacity_keyframes_multiplier,
                    melina.camouflage_transparency
                ));
                noter(&format!(
                    "Melina, affichage : {:?} ; {:?} ; {:?} ; chargement {:?}.",
                    melina.chr_flags1c4, melina.chr_flags1c5, melina.chr_flags1c8,
                    melina.load_state
                ));
                // Effets actifs sur Melina (pour retrouver celui du feu de sa dague).
                let effets = crate::suivi::effets_de(melina);
                noter(&format!("Melina : {} effet(s) actif(s) :", effets.len()));
                for (_, description) in effets {
                    noter(&format!("  - {description}"));
                }
            }
            None => noter("Melina : pas encore retrouvée dans la carte."),
        }
    }

    /// Cherche Melina parmi les personnages chargés (tant qu'elle n'est pas trouvée),
    /// puis la garde bien visible à chaque image : ses données de PNJ la font apparaître
    /// en fondu depuis une transparence totale (comme aux grâces du jeu normal).
    pub fn mettre_a_jour(&mut self) {
        if let Some(melina) = self.personnage() {
            melina.base_transparency = 1.0;
            melina.base_transparency_modifier = 1.0;
            melina.tint_alpha_multiplier = 1.0;
            // Essai : on valide de force sa condition « météo et heure », la seule qui
            // échoue d'après le journal (F10), pour que le jeu l'active et la dessine.
            melina.load_state.set_weather_and_time_of_day(true);
            // Insensible aux coups : depuis la 0.63.1, on peut frapper à la Table ronde
            // (zone de combat élargie dans la carte) ; Melina ne doit jamais être touchée.
            melina.debug_flags.set_disabled_hit(true);
            return;
        }
        let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
            return;
        };
        for liste in monde.chr_sets.iter().flatten() {
            for personnage in liste.characters() {
                if personnage.event_entity_id == ENTITE_MELINA {
                    self.personnage = Some(NonNull::from(&mut *personnage));
                    let p = personnage.modules.physics.position;
                    noter(&format!(
                        "Melina retrouvée dans la carte, en ({:.2}, {:.2}, {:.2}) ; transparence base {:.2} (évolution {:.2}), teinte {:.2}, images clés {:.2}, camouflage {:.2}.",
                        p.0, p.1, p.2,
                        personnage.base_transparency, personnage.base_transparency_modifier,
                        personnage.tint_alpha_multiplier, personnage.opacity_keyframes_multiplier,
                        personnage.camouflage_transparency
                    ));
                    return;
                }
            }
        }
    }

    /// Vrai si le joueur est assez près de Melina pour lui parler.
    pub fn joueur_assez_pres(&self, joueur: HavokPosition) -> bool {
        let Some(melina) = self.personnage() else {
            return false;
        };
        let p = melina.modules.physics.position;
        let (dx, dy, dz) = (p.0 - joueur.0, p.1 - joueur.1, p.2 - joueur.2);
        (dx * dx + dy * dy + dz * dz).sqrt() <= DISTANCE_POUR_PARLER
    }
}
