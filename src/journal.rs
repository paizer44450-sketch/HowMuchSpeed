//! Journal du mod : Documents\Mod Elden Ring\journal\mod.txt
//! Chaque ligne raconte ce que le mod a fait (utile pour comprendre un souci).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Ajoute une ligne au journal.
pub fn noter(message: &str) {
    let Ok(profil) = std::env::var("USERPROFILE") else {
        return;
    };
    let dossier = PathBuf::from(profil)
        .join("Documents")
        .join("Mod Elden Ring")
        .join("journal");
    let _ = fs::create_dir_all(&dossier);
    if let Ok(mut fichier) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dossier.join("mod.txt"))
    {
        let _ = writeln!(fichier, "{message}");
    }
}
