//! Vitesse globale du jeu, de 100 % à 300 % par paliers de 10 %.
//! Le jeu la range dans son gestionnaire d'images (CSFlipper, champ « game_speed »).

use eldenring::cs::CSFlipper;
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

pub const NORMALE: u32 = 100;
pub const MAX: u32 = 300;
pub const PALIER: u32 = 10;

pub struct Vitesse {
    /// La vitesse choisie par le joueur, en pourcentage.
    pub choisie: u32,
    /// Vrai quand le mod est en train d'imposer une vitesse au jeu.
    imposee: bool,
}

impl Vitesse {
    pub fn new() -> Self {
        Self { choisie: NORMALE, imposee: false }
    }

    pub fn accelerer(&mut self) {
        if self.choisie < MAX {
            self.choisie += PALIER;
            noter(&format!("Vitesse choisie : {} %", self.choisie));
        }
    }

    pub fn ralentir(&mut self) {
        if self.choisie > NORMALE {
            self.choisie -= PALIER;
            noter(&format!("Vitesse choisie : {} %", self.choisie));
        }
    }

    /// Choisit directement une vitesse (menu de Melina), bornée de 100 % à 300 %.
    pub fn definir(&mut self, pourcentage: u32) {
        self.choisie = pourcentage.clamp(NORMALE, MAX);
        noter(&format!("Vitesse choisie (menu) : {} %", self.choisie));
    }

    pub fn remettre_normale(&mut self) {
        if self.choisie != NORMALE {
            self.choisie = NORMALE;
            noter("Vitesse choisie : 100 %");
        }
    }

    /// À appeler à chaque image. « autorisee » = faux dans les endroits où on veut
    /// le jeu à 100 % (Table ronde : discussions avec les PNJ, qui peuvent planter en accéléré).
    pub fn appliquer(&mut self, autorisee: bool) {
        let Ok(flipper) = (unsafe { CSFlipper::instance_mut() }) else {
            return;
        };

        if autorisee && self.choisie != NORMALE {
            // Maintenue à chaque image, car le jeu peut la changer lui-même (cinématiques).
            flipper.game_speed = self.choisie as f32 / 100.0;
            self.imposee = true;
        } else if self.imposee {
            // On rend la main au jeu une seule fois, à 100 %.
            flipper.game_speed = 1.0;
            self.imposee = false;
        }
    }
}
