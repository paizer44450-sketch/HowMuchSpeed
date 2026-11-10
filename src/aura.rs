//! Aura : le visuel d'un buff (épines, foudre, flammes…) sur le personnage, permanent et
//! sans aucun effet de jeu.
//!
//! Mesures F11 du 07/11/2026 : chaque buff pose un ou plusieurs effets dont le visuel a un
//! visuel continu sur le corps. On ne recopie que les visuels : chaque ligne d'aura est une
//! copie de l'effet neutre 18553 (aucune statistique), permanente, sans catégorie, qui
//! pointe sur le visuel du buff (ligne du jeu, non modifiée). Remise après chaque
//! chargement (mort comprise), comme les effets d'arme.
//!
//! Lignes : 7230 et 7240 (« Area Scaling - (Unused) », recopiées depuis l'effet neutre :
//! ces lignes ne sont pas vides, elles multiplient PV, attaque, défenses et résistances).

use eldenring::cs::{ChrInsExt, SoloParamRepository, SpEffectParam, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Effet de Radagon sans aucune statistique : modèle des lignes.
const EFFET_NEUTRE: u32 = 18553;
/// Lignes d'effet réservées à l'aura (une par visuel, deux au plus).
const LIGNES: [u32; 2] = [7230, 7240];

/// Les auras proposées : (texte du menu, visuels du buff, nom).
pub const AURAS: [(i32, &[i32], &str); 14] = [
    (69000321, &[21490000, 21490001], "Corps d'épines"),
    // Camouflage seulement (pas de visuel continu) : le visuel de fin 523412 joue quand on
    // l'enlève.
    (69000322, &[1467000], "Corps indiscernable"),
    (69000323, &[21691000], "Éclair draconique de Fortissax"),
    (69000324, &[21692000], "Électrocharge"),
    (69000325, &[1697100], "Bénédiction électrifiante du dragon"),
    (69000326, &[1643100], "Bénédiction de l'Arbre-Monde"),
    (69000327, &[1646000], "Renforcement magique"),
    (69000328, &[1645000], "Renforcement ardent"),
    (69000329, &[1647000], "Renforcement électrique"),
    (69000330, &[1648000], "Renforcement divin"),
    (69000331, &[1606000], "Flamme, protège-moi"),
    (69000332, &[1605000], "Flamme, renforce-moi"),
    (69000333, &[1627000], "Flamme noire protectrice"),
    (69000334, &[1676000], "Décret de causalité"),
];

pub struct Aura {
    /// Aura choisie (rang dans AURAS), ou None.
    choisie: Option<usize>,
}

impl Aura {
    /// Choix actuel (lu par reglages.rs pour la sauvegarde).
    pub fn choisie(&self) -> Option<usize> {
        self.choisie
    }

    pub fn new() -> Self {
        Self { choisie: None }
    }

    /// Remplit les lignes de l'aura : copies de l'effet neutre, permanentes, sans
    /// catégorie, chacune avec un visuel du buff.
    fn preparer(rang: usize) -> bool {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(neutre) = parametres.get::<SpEffectParam>(EFFET_NEUTRE).cloned() else {
            noter(&format!("Aura : effet neutre {EFFET_NEUTRE} introuvable."));
            return false;
        };
        let (_, visuels, _) = AURAS[rang];
        for (ligne_libre, &visuel) in LIGNES.iter().zip(visuels.iter()) {
            let Some(ligne) = parametres.get_mut::<SpEffectParam>(*ligne_libre) else {
                noter(&format!("Aura : ligne d'effet {ligne_libre} introuvable."));
                return false;
            };
            *ligne = neutre.clone();
            ligne.set_effect_endurance(-1.0);
            ligne.set_sp_category(0);
            ligne.set_vfx_id(visuel);
        }
        true
    }

    /// Nombre de lignes utilisées par l'aura choisie.
    fn lignes_utilisees(&self) -> usize {
        self.choisie.map(|r| AURAS[r].1.len().min(LIGNES.len())).unwrap_or(0)
    }

    /// Choisit l'aura (None = aucune) et l'applique tout de suite.
    pub fn choisir(&mut self, rang: Option<usize>) {
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        // On enlève d'abord l'ancienne aura, avant de réécrire les lignes.
        for ligne in LIGNES {
            joueur.chr_ins.remove_speffect(ligne as i32);
        }
        self.choisie = None;
        let Some(r) = rang else {
            noter("Aura : aucune.");
            return;
        };
        if !Self::preparer(r) {
            return;
        }
        self.choisie = Some(r);
        for &ligne in LIGNES.iter().take(self.lignes_utilisees()) {
            joueur.apply_speffect(ligne as i32, true);
        }
        noter(&format!("Aura : {}.", AURAS[r].2));
    }

    /// À appeler à chaque image : remet l'aura si elle a disparu (mort, chargement).
    pub fn mettre_a_jour(&mut self) {
        if self.choisie.is_none() {
            return;
        }
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        for &ligne in LIGNES.iter().take(self.lignes_utilisees()) {
            let ligne = ligne as i32;
            if !joueur.chr_ins.special_effect.entries().any(|e| e.param_id == ligne) {
                joueur.apply_speffect(ligne, true);
            }
        }
    }
}
