//! Mannequin d'entraînement de la Table ronde : un ennemi immobile (sans intelligence,
//! Think -1) posé dans la carte m11_10_00_00, entité MANNEQUIN, pour tester les impacts et
//! les armes fantômes sans sortir. Les coups sont autorisés dans toute la Table ronde depuis
//! la 0.63.1 (région « zone de non-combat (exclusion) », entité 11102600, agrandie).
//!
//! Le mod lui rend tous ses PV dès qu'il descend sous la moitié : il ne meurt jamais, et les
//! coups restent détectés par l'Impact (qui compare les PV d'une image à l'autre).

use eldenring::cs::WorldChrMan;
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Numéro d'entité du mannequin dans la carte (absent ailleurs dans la carte).
pub const MANNEQUIN: u32 = 11105951;

pub struct Mannequin {
    vu: bool,
}

impl Mannequin {
    pub fn new() -> Self {
        Self { vu: false }
    }

    /// À appeler à chaque image, après l'Impact.
    pub fn mettre_a_jour(&mut self) {
        let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
            return;
        };
        for liste in monde.chr_sets.iter().flatten() {
            for personnage in liste.characters() {
                if personnage.event_entity_id != MANNEQUIN {
                    continue;
                }
                if !self.vu {
                    self.vu = true;
                    let p = personnage.modules.physics.position;
                    noter(&format!(
                        "Mannequin trouvé (modèle c{:04}) en ({:.2}, {:.2}, {:.2}), PV {}/{}.",
                        personnage.character_id, p.0, p.1, p.2, personnage.modules.data.hp, personnage.modules.data.max_hp
                    ));
                }
                let data = &mut personnage.modules.data;
                if data.hp < data.max_hp / 2 {
                    data.hp = data.max_hp;
                }
                return;
            }
        }
    }
}
