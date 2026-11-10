//! Recul de la caméra : la distance de la caméra au personnage, allongée de 5 à 100 % (par
//! crans de 5 %), pour voir en entier les gros effets.
//!
//! La distance vient des lignes de LockCamParam (champ camDistTarget, une ligne par
//! situation de caméra). On les multiplie toutes en mémoire, d'après leur valeur d'origine
//! gardée au premier changement. Rien n'est écrit dans regulation.bin : au prochain
//! lancement du jeu, la caméra est normale.

use eldenring::cs::{LockCamParam, SoloParamRepository};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Recul maximal et palier, en pourcentage.
pub const RECUL_MAX: u32 = 100;
pub const PALIER: u32 = 5;

pub struct Camera {
    /// Recul choisi, en pourcentage (0 = normale).
    pub recul: u32,
    /// Distances d'origine de chaque ligne de LockCamParam.
    origines: Vec<(u32, f32)>,
}

impl Camera {
    pub fn new() -> Self {
        Self { recul: 0, origines: Vec::new() }
    }

    pub fn choisir(&mut self, recul: u32) {
        self.recul = recul.min(RECUL_MAX);
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            noter("Caméra : données du jeu introuvables.");
            return;
        };
        if self.origines.is_empty() {
            self.origines = parametres
                .rows_mut::<LockCamParam>()
                .map(|(id, ligne)| (id, ligne.cam_dist_target()))
                .collect();
        }
        let facteur = 1.0 + self.recul as f32 / 100.0;
        let mut nombre = 0;
        for (id, ligne) in parametres.rows_mut::<LockCamParam>() {
            if let Some(&(_, origine)) = self.origines.iter().find(|(i, _)| *i == id) {
                ligne.set_cam_dist_target(origine * facteur);
                nombre += 1;
            }
        }
        noter(&format!("Caméra : recul de {} % ({nombre} lignes de caméra).", self.recul));
    }
}
