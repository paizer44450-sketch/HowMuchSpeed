//! Affichage de la vitesse à l'écran, en bas à droite, juste au-dessus du compteur de runes.
//!
//! Le dessin passe par hudhook (Andrea Venuta, licence MIT) : il s'accroche au rendu
//! DirectX 12 du jeu et nous laisse dessiner du texte par-dessus l'image, avec Dear ImGui.
//! On ne dessine aucune fenêtre : seulement le texte, sans bloquer ni souris, ni clavier,
//! ni manette (filtre de messages vide).
//!
//! La police est lue au lancement dans les polices de Windows (Georgia, sinon Times New
//! Roman) : elle n'est pas fournie avec le mod. Sans elle, la police intégrée d'ImGui sert.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::imgui::{Context, FontConfig, FontSource, ImColor32, Ui};
use hudhook::windows::Win32::Foundation::HINSTANCE;
use hudhook::{Hudhook, ImguiRenderLoop, RenderContext};

use crate::journal::noter;

/// Vitesse à afficher, en pourcentage. 0 = rien à afficher.
static VITESSE_AFFICHEE: AtomicU32 = AtomicU32::new(0);
/// Galerie des sorts : rang du sort affiché + 1 (0 = galerie arrêtée, rien à afficher).
static GALERIE_RANG: AtomicU32 = AtomicU32::new(0);
/// Mot affiché devant la vitesse, selon la langue du jeu (« Vitesse » ou « Speed »).
static MOT_VITESSE: OnceLock<&'static str> = OnceLock::new();

/// Langue du jeu. Le jeu affiche ses textes dans la langue que Steam lui donne : on la
/// demande à Steam de la même façon (bibliothèque steam_api64.dll déjà chargée par le jeu,
/// fonction GetCurrentGameLanguage). Si Steam ne répond pas, on la lit dans le fichier de
/// Steam du jeu (appmanifest_1245620.acf). Français → « Vitesse », toute autre langue (ou
/// langue introuvable) → « Speed ».
fn mot_vitesse() -> &'static str {
    let langue = langue_donnee_par_steam().or_else(langue_du_fichier_steam);
    noter(&format!("Langue du jeu : {}.", langue.as_deref().unwrap_or("introuvable")));
    if langue.is_some_and(|l| l.eq_ignore_ascii_case("french")) { "Vitesse" } else { "Speed" }
}

/// Langue que Steam donne au jeu (ex. « english », « french »), ou None si Steam ne répond pas.
fn langue_donnee_par_steam() -> Option<String> {
    use std::ffi::{CStr, c_char, c_void};
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    use windows::core::{s, w};

    unsafe {
        let module = GetModuleHandleW(w!("steam_api64.dll")).ok()?;
        let acces = GetProcAddress(module, s!("SteamAPI_SteamApps_v008"))?;
        let demande = GetProcAddress(module, s!("SteamAPI_ISteamApps_GetCurrentGameLanguage"))?;
        let acces: unsafe extern "C" fn() -> *mut c_void = std::mem::transmute(acces);
        let demande: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(demande);
        let applications = acces();
        if applications.is_null() {
            return None;
        }
        let texte = demande(applications);
        if texte.is_null() {
            return None;
        }
        CStr::from_ptr(texte).to_str().ok().map(str::to_owned)
    }
}

/// Langue lue dans le fichier de Steam du jeu, dans le dossier steamapps au-dessus de celui
/// du jeu (steamapps\common\ELDEN RING\Game). On prend la ligne « language » de la partie
/// « UserConfig » (choix du joueur), sinon la première trouvée.
fn langue_du_fichier_steam() -> Option<String> {
    let chemin = std::env::current_exe().ok()?.ancestors().nth(4)?.join("appmanifest_1245620.acf");
    let texte = std::fs::read_to_string(chemin).ok()?;
    let lignes: Vec<String> = texte.lines().map(|l| l.trim().to_lowercase()).collect();
    let debut = lignes.iter().position(|l| l == "\"userconfig\"").unwrap_or(0);
    let langue = |l: &String| {
        l.strip_prefix("\"language\"").map(|reste| reste.trim().trim_matches('"').to_owned())
    };
    lignes[debut..].iter().find_map(langue).or_else(|| lignes.iter().find_map(langue))
}

// Réglages de placement, en fraction de la hauteur de l'écran (comme l'interface du jeu,
// qui grandit avec la hauteur). Repères pris sur un écran 1920 x 1080.
/// Taille du texte : environ celle des chiffres des runes (28 px en 1080).
const TAILLE_TEXTE: f32 = 28.0 / 1080.0;
/// Distance entre le bord droit de l'écran et la fin du texte.
const MARGE_DROITE: f32 = 62.0 / 1080.0;
/// Distance entre le bas de l'écran et le bas du texte (au-dessus du cadre des runes).
const MARGE_BAS: f32 = 92.0 / 1080.0;

/// Couleur du texte : blanc cassé, comme les chiffres des runes.
const COULEUR_TEXTE: ImColor32 = ImColor32::from_rgba(235, 230, 215, 255);
/// Contour sombre, pour rester lisible sur un fond clair.
const COULEUR_CONTOUR: ImColor32 = ImColor32::from_rgba(0, 0, 0, 200);

/// Polices de Windows essayées, dans l'ordre.
const POLICES: [&str; 2] = [r"C:\Windows\Fonts\georgia.ttf", r"C:\Windows\Fonts\times.ttf"];

/// Change la vitesse affichée (0 pour ne rien afficher). Appelée à chaque image.
pub fn afficher_vitesse(pourcentage: u32) {
    VITESSE_AFFICHEE.store(pourcentage, Ordering::Relaxed);
}

/// Galerie des sorts : sort à afficher en haut de l'écran (None : rien).
pub fn afficher_galerie(rang: Option<usize>) {
    GALERIE_RANG.store(rang.map_or(0, |r| r as u32 + 1), Ordering::Relaxed);
}

/// Vrai si le jeu est en français (même règle que le mot « Vitesse »).
fn en_francais() -> bool {
    MOT_VITESSE.get().copied() == Some("Vitesse")
}

/// Dessine un texte avec son contour sombre.
fn texte_avec_contour(ui: &Ui, position: [f32; 2], texte: &str) {
    let dessin = ui.get_foreground_draw_list();
    let hauteur_ecran = ui.io().display_size[1];
    let epaisseur = (hauteur_ecran / 1080.0).round().max(1.0);
    let [x, y] = position;
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0)] {
        dessin.add_text([x + dx * epaisseur, y + dy * epaisseur], COULEUR_CONTOUR, texte);
    }
    dessin.add_text([x, y], COULEUR_TEXTE, texte);
}

/// Galerie des sorts : deux lignes centrées en haut de l'écran.
fn dessiner_galerie(ui: &Ui, rang: usize) {
    use crate::galerie::{SORTS, nom_anglais, nom_francais};
    let Some(sort) = SORTS.get(rang) else {
        return;
    };
    let francais = en_francais();
    let nom = if francais { nom_francais(sort) } else { nom_anglais(sort) };
    let ligne1 = if francais {
        format!("Galerie des sorts {} / {} : {}  (n° {})", rang + 1, SORTS.len(), nom, sort.id)
    } else {
        format!("Spell gallery {} / {}: {}  (#{})", rang + 1, SORTS.len(), nom, sort.id)
    };
    let ligne2 = if francais { "Changer de sort : sort suivant" } else { "Switch spell: next spell" };
    let [largeur_ecran, hauteur_ecran] = ui.io().display_size;
    let mut y = hauteur_ecran * MARGE_HAUT_GALERIE;
    for ligne in [ligne1.as_str(), ligne2] {
        let [largeur, hauteur] = ui.calc_text_size(ligne);
        texte_avec_contour(ui, [(largeur_ecran - largeur) / 2.0, y], ligne);
        y += hauteur * 1.2;
    }
}

/// Distance entre le haut de l'écran et la galerie des sorts.
const MARGE_HAUT_GALERIE: f32 = 40.0 / 1080.0;

/// Branche l'affichage sur le rendu du jeu. À appeler une fois, quand le jeu est prêt.
pub fn demarrer(module_du_mod: usize) {
    let mot = *MOT_VITESSE.get_or_init(mot_vitesse);
    noter(&format!("Affichage de la vitesse : « {mot} »."));
    let resultat = Hudhook::builder()
        .with::<ImguiDx12Hooks>(AffichageVitesse)
        .with_hmodule(HINSTANCE(module_du_mod as *mut _))
        .build()
        .apply();
    match resultat {
        Ok(()) => noter("Affichage de la vitesse branché."),
        Err(e) => noter(&format!("ERREUR : affichage de la vitesse impossible ({e:?}).")),
    }
}

struct AffichageVitesse;

impl ImguiRenderLoop for AffichageVitesse {
    fn initialize<'a>(&'a mut self, ctx: &mut Context, _rendu: &'a mut dyn RenderContext) {
        // Pas de fichier imgui.ini dans le dossier du jeu.
        ctx.set_ini_filename(None);

        // La police est préparée une fois, à la taille voulue pour cet écran.
        let hauteur = ctx.io().display_size[1].max(720.0);
        let taille = (hauteur * TAILLE_TEXTE).round();
        let police = POLICES.iter().find_map(|chemin| std::fs::read(chemin).ok());
        match police {
            Some(donnees) => {
                ctx.fonts().add_font(&[FontSource::TtfData {
                    data: &donnees,
                    size_pixels: taille,
                    config: Some(FontConfig { oversample_h: 3, oversample_v: 2, ..FontConfig::default() }),
                }]);
            }
            None => {
                noter("Police Georgia introuvable : police intégrée utilisée pour la vitesse.");
                ctx.fonts().add_font(&[FontSource::DefaultFontData {
                    config: Some(FontConfig { size_pixels: taille, ..FontConfig::default() }),
                }]);
            }
        }
    }

    fn render(&mut self, ui: &mut Ui) {
        let galerie = GALERIE_RANG.load(Ordering::Relaxed);
        if galerie > 0 {
            dessiner_galerie(ui, galerie as usize - 1);
        }

        let vitesse = VITESSE_AFFICHEE.load(Ordering::Relaxed);
        if vitesse == 0 {
            return;
        }

        let mot = MOT_VITESSE.get().copied().unwrap_or("Speed");
        // Espace avant « % » en français seulement.
        let texte = if mot == "Vitesse" { format!("{mot} {vitesse} %") } else { format!("{mot} {vitesse}%") };
        let [largeur_ecran, hauteur_ecran] = ui.io().display_size;
        let [largeur_texte, hauteur_texte] = ui.calc_text_size(&texte);
        let x = largeur_ecran - hauteur_ecran * MARGE_DROITE - largeur_texte;
        let y = hauteur_ecran - hauteur_ecran * MARGE_BAS - hauteur_texte;
        texte_avec_contour(ui, [x, y], &texte);
    }
}
