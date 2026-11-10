//! Fondu au noir pendant les téléportations du mod.
//!
//! Le jeu dessine ses fondus avec des « plaques » (CSFade, 9 plaques). Au départ d'une
//! téléportation, on noircit une plaque ; on l'éclaircit une fois arrivé, après le
//! placement devant la brume s'il y en a un (le décor qui se charge et le déplacement du
//! joueur restent ainsi cachés).
//! Les plaques sont notées une fois dans le journal (nom, couleur) pour choisir la bonne si
//! la plaque 0 ne s'affiche pas.

use std::sync::atomic::{AtomicBool, Ordering};

use eldenring::cs::{CSFD4FadePlate, CSFade};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Plaque utilisée pour le fondu.
const PLAQUE: usize = 0;
/// Durée du passage au noir et du retour à l'image (secondes).
const DUREE_NOIRCIR: f32 = 0.3;
const DUREE_ECLAIRCIR: f32 = 1.0;

/// Vrai une fois les plaques notées dans le journal.
static PLAQUES_NOTEES: AtomicBool = AtomicBool::new(false);

fn plaque() -> Option<&'static mut CSFD4FadePlate> {
    let fondu = unsafe { CSFade::instance_mut() }.ok()?;
    if !PLAQUES_NOTEES.swap(true, Ordering::Relaxed) {
        for (i, p) in fondu.fade_plates.iter().enumerate() {
            let fin = p.title.iter().position(|&c| c == 0).unwrap_or(p.title.len());
            let couleur: [f32; 4] = (&p.current_color).into();
            noter(&format!(
                "Fondu : plaque {i} « {} », couleur actuelle {:?}.",
                String::from_utf16_lossy(&p.title[..fin]),
                couleur
            ));
        }
    }
    Some(&mut *fondu.fade_plates[PLAQUE])
}

/// Met la plaque en noir pur (seule l'opacité change ensuite).
fn mettre_en_noir(p: &mut CSFD4FadePlate) {
    p.start_color.r = 0.0;
    p.start_color.g = 0.0;
    p.start_color.b = 0.0;
    p.end_color.r = 0.0;
    p.end_color.g = 0.0;
    p.end_color.b = 0.0;
}

/// Passe au noir (départ d'une téléportation).
pub fn noircir() {
    if let Some(p) = plaque() {
        mettre_en_noir(p);
        p.fade_out(DUREE_NOIRCIR);
        noter("Fondu au noir.");
    }
}

/// Revient à l'image (arrivée).
pub fn eclaircir() {
    if let Some(p) = plaque() {
        mettre_en_noir(p);
        p.fade_in(DUREE_ECLAIRCIR);
        noter("Fin du fondu au noir.");
    }
}

/// Opacité de la teinte rouge pendant un suivi (outil de mesure, voir suivi.rs).
const TEINTE_SUIVI: f32 = 0.08;

/// Teinte l'écran en rouge léger pendant un suivi, ou l'enlève.
pub fn teinte_suivi(actif: bool) {
    if let Some(p) = plaque() {
        let a = if actif { TEINTE_SUIVI } else { 0.0 };
        p.start_color.r = 1.0;
        p.start_color.g = 0.0;
        p.start_color.b = 0.0;
        p.end_color.r = 1.0;
        p.end_color.g = 0.0;
        p.end_color.b = 0.0;
        p.start_color.a = a;
        p.end_color.a = a;
        p.current_color.a = a;
        p.fade_duration.time = 0.2;
        p.fade_timer.time = 0.2;
    }
}
