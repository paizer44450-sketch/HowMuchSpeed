//! Sauvegarde des effets choisis chez Melina (09/11/2026) : ils sont remis à la partie
//! suivante, même après avoir fermé le jeu.
//!
//! Fichier : Documents\Mod Elden Ring\reglages_effets.txt, une ligne « clé=valeur » par
//! réglage. La valeur est le NOM de l'effet (pas son rang dans la liste) : si une liste
//! change d'ordre dans une version suivante, le bon effet est quand même retrouvé. Un nom
//! inconnu (effet retiré du mod) est ignoré. « - » = aucun.
//!
//! Le fichier est réécrit dès qu'un choix change (comparaison à chaque image d'une
//! description texte des choix). Il est relu une seule fois par session, après la fin du
//! premier chargement : avant, le personnage n'existe pas et les effets ne se posent pas.
//!
//! « Réinitialiser tous les effets » (Effets d'arme) remet tout à zéro : effets et traînées
//! des deux mains, roulade normale, aura, course, impacts, armes fantômes. La caméra est
//! sauvegardée aussi, mais n'est pas touchée par la réinitialisation.

use std::fs;
use std::path::PathBuf;

use crate::effets_arme::{EFFETS, Emplacement, Genre, Main};
use crate::journal::noter;
use crate::menu::MenuMelina;

/// Chemin du fichier de sauvegarde.
fn chemin() -> Option<PathBuf> {
    let profil = std::env::var("USERPROFILE").ok()?;
    Some(PathBuf::from(profil).join("Documents").join("Mod Elden Ring").join("reglages_effets.txt"))
}

/// Suivi de la sauvegarde pendant la session.
pub struct Suivi {
    /// Vrai une fois le fichier relu (une seule fois par session).
    restaures: bool,
    /// Dernière description écrite dans le fichier.
    dernier: String,
}

impl Suivi {
    pub fn new() -> Self {
        Self { restaures: false, dernier: String::new() }
    }

    /// À appeler à chaque image, une fois le chargement sûrement terminé.
    pub fn mettre_a_jour(&mut self, menu: &mut MenuMelina) {
        if !self.restaures {
            self.restaures = true;
            restaurer(menu);
            self.dernier = description(menu);
            return;
        }
        let maintenant = description(menu);
        if maintenant != self.dernier {
            if let Some(chemin) = chemin() {
                if fs::write(&chemin, &maintenant).is_err() {
                    noter("Réglages : impossible d'écrire reglages_effets.txt.");
                }
            }
            self.dernier = maintenant;
        }
    }
}

fn nom_ou_rien(nom: Option<&str>) -> &str {
    nom.unwrap_or("-")
}

/// Description texte de tous les choix (contenu du fichier).
fn description(menu: &MenuMelina) -> String {
    let effet = |r: Option<usize>| r.map(|r| EFFETS[r].4);
    let (droite_effet, droite_trainee) = menu.effets_arme.choix_main(Main::Droite);
    let (gauche_effet, gauche_trainee) = menu.effets_arme.choix_main(Main::Gauche);
    let impact = |r: Option<usize>| r.map(|r| crate::impact::IMPACTS[r].2);
    let fantome = |r: Option<usize>| r.map(|r| crate::arme_fantome::FANTOMES[r].nom);
    let lignes = [
        ("effet_droite", nom_ou_rien(effet(droite_effet)).to_string()),
        ("trainee_droite", nom_ou_rien(effet(droite_trainee)).to_string()),
        ("effet_gauche", nom_ou_rien(effet(gauche_effet)).to_string()),
        ("trainee_gauche", nom_ou_rien(effet(gauche_trainee)).to_string()),
        ("roulade", menu.roulade.style().nom().to_string()),
        ("aura", nom_ou_rien(menu.aura.choisie().map(|r| crate::aura::AURAS[r].2)).to_string()),
        ("course", nom_ou_rien(menu.course.choisi().map(crate::course::nom_du_style)).to_string()),
        ("impact_droite", nom_ou_rien(impact(menu.impact.choisi())).to_string()),
        ("impact_gauche", nom_ou_rien(impact(menu.impact_gauche.choisi())).to_string()),
        ("fantome_droite", nom_ou_rien(fantome(menu.arme_fantome.choisi())).to_string()),
        ("fantome_gauche", nom_ou_rien(fantome(menu.arme_fantome_gauche.choisi())).to_string()),
        ("camera", menu.camera.recul.to_string()),
    ];
    let mut texte = String::new();
    for (cle, valeur) in lignes {
        texte.push_str(cle);
        texte.push('=');
        texte.push_str(&valeur);
        texte.push('\n');
    }
    texte
}

/// Rang du nom `nom` dans une liste de `nombre` noms (None si « - » ou inconnu).
fn rang_de(nom: &str, nombre: usize, nom_de: impl Fn(usize) -> &'static str) -> Option<usize> {
    if nom == "-" {
        return None;
    }
    let rang = (0..nombre).find(|&r| nom_de(r) == nom);
    if rang.is_none() {
        noter(&format!("Réglages : « {nom} » n'existe plus, ignoré."));
    }
    rang
}

/// Relit le fichier et remet les effets.
fn restaurer(menu: &mut MenuMelina) {
    let Some(chemin) = chemin() else {
        return;
    };
    let Ok(texte) = fs::read_to_string(&chemin) else {
        noter("Réglages : pas encore de reglages_effets.txt (rien à remettre).");
        return;
    };
    noter("Réglages : effets de la dernière partie remis.");
    let effet = |nom: &str| rang_de(nom, EFFETS.len(), |r| EFFETS[r].4);
    let impact = |nom: &str| rang_de(nom, crate::impact::IMPACTS.len(), |r| crate::impact::IMPACTS[r].2);
    let fantome = |nom: &str| rang_de(nom, crate::arme_fantome::FANTOMES.len(), |r| crate::arme_fantome::FANTOMES[r].nom);
    for ligne in texte.lines() {
        let Some((cle, valeur)) = ligne.split_once('=') else {
            continue;
        };
        let valeur = valeur.trim();
        let emplacement = |main, genre| Emplacement { main, genre };
        match cle.trim() {
            "effet_droite" => menu.effets_arme.choisir(emplacement(Main::Droite, Genre::Effet), effet(valeur)),
            "trainee_droite" => menu.effets_arme.choisir(emplacement(Main::Droite, Genre::Trainee), effet(valeur)),
            "effet_gauche" => menu.effets_arme.choisir(emplacement(Main::Gauche, Genre::Effet), effet(valeur)),
            "trainee_gauche" => menu.effets_arme.choisir(emplacement(Main::Gauche, Genre::Trainee), effet(valeur)),
            "roulade" => {
                if let Some(&style) = crate::roulade::STYLES_MENU.iter().find(|s| s.nom() == valeur) {
                    menu.roulade.choisir(style);
                }
            }
            "aura" => menu.aura.choisir(rang_de(valeur, crate::aura::AURAS.len(), |r| crate::aura::AURAS[r].2)),
            "course" => menu.course.choisir(rang_de(valeur, crate::course::NOMBRE, crate::course::nom_du_style)),
            "impact_droite" => menu.impact.choisir(impact(valeur)),
            "impact_gauche" => menu.impact_gauche.choisir(impact(valeur)),
            "fantome_droite" => menu.arme_fantome.choisir(fantome(valeur)),
            "fantome_gauche" => menu.arme_fantome_gauche.choisir(fantome(valeur)),
            "camera" => {
                if let Ok(recul) = valeur.parse::<u32>() {
                    menu.camera.choisir(recul.min(crate::camera::RECUL_MAX));
                }
            }
            _ => {}
        }
    }
}

/// « Réinitialiser tous les effets » : tout à zéro (sauf la caméra).
pub fn reinitialiser(menu: &mut MenuMelina) {
    for main in [Main::Droite, Main::Gauche] {
        for genre in [Genre::Effet, Genre::Trainee] {
            menu.effets_arme.choisir(Emplacement { main, genre }, None);
        }
    }
    menu.roulade.choisir(crate::roulade::Style::Normale);
    menu.aura.choisir(None);
    menu.course.choisir(None);
    menu.impact.choisir(None);
    menu.impact_gauche.choisir(None);
    menu.arme_fantome.choisir(None);
    menu.arme_fantome_gauche.choisir(None);
    noter("Réglages : tous les effets réinitialisés.");
}
