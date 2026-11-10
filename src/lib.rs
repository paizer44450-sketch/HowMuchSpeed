//! Mod « Vitesse des bosses » pour Elden Ring.
//!
//! ÉTAPE 3 : la Table ronde comme lieu principal, vidée de ses PNJ.
//!   - Au chargement de la partie (5 s après la fin du chargement), envoi à la Table ronde.
//!   - Les PNJ de la Table ronde sont retirés du fichier de la carte (Smithbox :
//!     modengine\mod\map\mapstudio\m11_10_00_00.msb.dcx), plus par la DLL.
//!   - F8 : retour à la Table ronde à tout moment.
//!   - F9 : note la position du joueur dans le journal (pour placer Melina).
//!   - Au chargement : toutes les grâces et toute la carte débloquées (voir deblocage.rs).
//!   - Fondu au noir pendant les téléportations (voir fondu.rs).
//!   - F11 : début / fin du suivi des effets du joueur (outil de mesure, voir suivi.rs).
//!   - F10 : note la position de Melina dans le journal.
//!   - Melina (PNJ de la carte du mod, entité 11105950) : son menu (Bosses, Vitesse, Fioles), voir menu.rs.
//!   - Vitesse : F6 = +10 %, F5 = -10 %, F7 = 100 % (de 100 % à 300 %), ou par le menu.
//!     À la Table ronde, le jeu reste à 100 % (les discussions avec les PNJ
//!     peuvent planter en accéléré) ; la vitesse choisie revient dès qu'on en sort.
//!
//! Journal : Documents\Mod Elden Ring\journal\mod.txt
//! HORS LIGNE UNIQUEMENT (Mod Engine 2 lance le jeu sans l'anti-triche).

mod arrivees;
mod aura;
mod course;
mod galerie;
mod impact;
mod arme_fantome;
mod mannequin;
mod camera;
mod deblocage;
mod effets_arme;
mod fondu;
mod hud;
mod suivi;
mod armes;
mod journal;
mod marchands;
mod melina;
mod menu;
mod reglages;
mod roulade;
mod teleportation;
mod vitesse;

use std::time::{Duration, Instant};

use eldenring::{
    cs::{CSActionButtonManImp, CSTaskGroupIndex, CSTaskImp, GameMan, WorldChrMan},
    fd4::FD4TaskData,
    position::HavokPosition,
    util::input,
};
use fromsoftware_shared::{FromStatic, SharedTaskImpExt};

use journal::noter;
use melina::Melina;
use menu::MenuMelina;
use vitesse::Vitesse;

/// Invite « Parler » du jeu (ActionButtonParam 6000).
const INVITE_PARLER: i32 = 6000;

// Codes des touches du clavier (codes « virtual key » de Windows).
const TOUCHE_F5: i32 = 0x74;
const TOUCHE_F6: i32 = 0x75;
const TOUCHE_F7: i32 = 0x76;
const TOUCHE_F8: i32 = 0x77;
const TOUCHE_F9: i32 = 0x78;
const TOUCHE_F10: i32 = 0x79;
const TOUCHE_F11: i32 = 0x7A;

/// Fondu au noir : délai avant de revenir à l'image après l'arrivée (sans placement), délai
/// après le placement devant la brume, et délai maximal (sécurité : jamais d'écran noir
/// bloqué).
const FONDU_APRES_ARRIVEE: Duration = Duration::from_millis(800);
const FONDU_APRES_PLACEMENT: Duration = Duration::from_millis(1500);
const FONDU_MAX: Duration = Duration::from_secs(12);

/// Pour une mesure du début du jeu (08/11/2026 : drapeau des fioles) : vrai = la partie se
/// charge normalement, sans déblocage ni envoi à la Table ronde. Toujours false en jeu.
const MESURE_PARTIE_NORMALE: bool = false;

/// Carte de la Table ronde : m11_10.
pub const ZONE_TABLE_RONDE: u8 = 11;
pub const BLOC_TABLE_RONDE: u8 = 10;

/// Attente après l'apparition du personnage, pour être sûr que l'écran de chargement
/// est fini, avant d'agir (envoi à la Table ronde, retrait des PNJ).
/// Agir pendant le chargement peut le bloquer à l'infini.
const ATTENTE_APRES_CHARGEMENT: Duration = Duration::from_secs(5);

/// Attente après un chargement avant de placer le joueur devant un boss (la carte et le
/// repère du joueur ne sont pas justes tout de suite). Les grâces de départ sont toutes
/// hors des arènes, donc aucun combat ne démarre pendant ce temps.
const ATTENTE_AVANT_PLACEMENT: Duration = Duration::from_secs(3);

/// Même attente dans le monde ouvert, plus longue : le jeu y finit l'arrivée à la grâce plus
/// tard, et annulait un placement fait trop tôt (Metyr, 06/11/2026).
const ATTENTE_AVANT_PLACEMENT_MONDE_OUVERT: Duration = Duration::from_secs(6);

/// Après un placement, on vérifie encore à ces intervalles que le joueur est resté sur
/// place (le jeu peut le remettre à la grâce), jusqu'à ce nombre d'essais.
const INTERVALLE_VERIFICATION: Duration = Duration::from_secs(2);
const ESSAIS_DE_PLACEMENT: u8 = 3;

/// Au-delà, on abandonne le placement pour ce chargement (carte jamais reconnue).
const ATTENTE_MAX_PLACEMENT: Duration = Duration::from_secs(8);

/// Durée pendant laquelle la gravité reste coupée après un placement (chargement du sol).
const GRAVITE_COUPEE: Duration = Duration::from_secs(5);

/// Après le retour de la gravité, on continue d'effacer toute chute comptée pendant ce temps
/// (atterrissage après un placement bien plus bas que la grâce : Radahn, Metyr).
const CHUTE_ANNULEE_APRES: Duration = Duration::from_secs(3);

/// Un chargement (mort) moins de ce temps après un placement suspend les placements.
const SECURITE_MORT_APRES_PLACEMENT: Duration = Duration::from_secs(20);

/// Ce dont le mod se souvient d'une image à l'autre.
struct Etat {
    vitesse: Vitesse,
    /// Mannequin d'entraînement de la Table ronde (voir mannequin.rs).
    mannequin: mannequin::Mannequin,
    /// Sauvegarde des effets choisis chez Melina (voir reglages.rs).
    reglages: reglages::Suivi,
    /// Numéro de la grâce de la Table ronde (trouvé une fois dans les données du jeu).
    grace_table_ronde: Option<u32>,
    /// Moment où le personnage est apparu (None tant qu'il n'est pas chargé).
    personnage_apparu: Option<Instant>,
    /// Vrai une fois l'envoi automatique à la Table ronde fait pour cette session.
    arrivee_faite: bool,
    /// Vrai entre une téléportation demandée et le chargement qui suit.
    depart_en_cours: bool,
    /// Position du joueur à son arrivée à la Table ronde (devant la table).
    arrivee_table_ronde: Option<HavokPosition>,
    /// Boss visé (point d'arrivée devant son mur de brume), tant qu'on ne revient pas à la
    /// Table ronde. Le joueur y est replacé après chaque chargement (mort comprise).
    boss_vise: Option<&'static arrivees::Arrivee>,
    /// Placements faits depuis le dernier chargement, et moment du dernier.
    placements: u8,
    dernier_placement: Option<Instant>,
    /// Position d'où le joueur a été déplacé (pour savoir si le jeu l'y a remis).
    position_avant_placement: Option<HavokPosition>,
    /// Moment du dernier placement (gardé d'un chargement à l'autre).
    placement_precedent: Option<Instant>,
    /// Vrai si le placement est suspendu (mort juste après un placement) jusqu'au retour à
    /// la Table ronde : évite les morts en boucle si un point d'arrivée est mauvais.
    placement_suspendu: bool,
    /// Vrai entre un placement (gravité coupée) et le rétablissement de la gravité.
    gravite_coupee: bool,
    /// Chargements depuis le dernier départ vers un boss (1 = l'arrivée).
    chargements_depuis_depart: u32,
    /// Vrai entre le départ d'une téléportation (écran noirci) et le retour à l'image.
    fondu_actif: bool,
    /// Vrai dès qu'un chargement s'annonce (téléportation demandée au jeu, ou mort du joueur),
    /// jusqu'à l'écran de chargement : la vitesse est remise à 100 % avant le chargement.
    chargement_annonce: bool,
    /// Outil de mesure des effets (F11).
    suivi: suivi::Suivi,
    melina: Melina,
    menu: MenuMelina,
}

// Le jeu appelle notre tâche toujours depuis son fil principal : on peut y garder des pointeurs.
unsafe impl Send for Etat {}

/// Point d'entrée de la DLL : Windows l'appelle quand le jeu charge le mod.
///
/// # Safety
/// Appelée uniquement par Windows au chargement de la DLL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DllMain(module_du_mod: usize, raison: u32) -> bool {
    // 1 = la DLL vient d'être chargée. Les autres appels ne nous intéressent pas.
    if raison != 1 {
        return true;
    }

    // On attend que le jeu soit prêt dans un fil à part, pour ne pas bloquer son démarrage.
    std::thread::spawn(move || {
        noter("==== Mod chargé, en attente du jeu... ====");
        let Ok(taches_du_jeu) = CSTaskImp::wait_for_instance(Duration::MAX) else {
            noter("ERREUR : gestionnaire de tâches du jeu introuvable.");
            return;
        };
        noter("Jeu prêt.");

        // Vitesse affichée à l'écran (en bas à droite, au-dessus des runes).
        hud::demarrer(module_du_mod);

        let mut etat = Etat {
            mannequin: mannequin::Mannequin::new(),
            reglages: reglages::Suivi::new(),
            vitesse: Vitesse::new(),
            grace_table_ronde: None,
            personnage_apparu: None,
            arrivee_faite: false,
            depart_en_cours: false,
            arrivee_table_ronde: None,
            boss_vise: None,
            placements: 0,
            dernier_placement: None,
            position_avant_placement: None,
            placement_precedent: None,
            placement_suspendu: false,
            gravite_coupee: false,
            chargements_depuis_depart: 0,
            fondu_actif: false,
            chargement_annonce: false,
            suivi: suivi::Suivi::new(),
            melina: Melina::new(),
            menu: MenuMelina::new(),
        };

        // Appelé par le jeu au début de chaque image.
        let tache = taches_du_jeu.run_recurring(
            move |_: &FD4TaskData| chaque_image(&mut etat),
            CSTaskGroupIndex::FrameBegin,
        );
        // Garder la tâche active pour toute la partie (sinon elle s'arrêterait ici).
        std::mem::forget(tache);
    });

    true
}

/// Tout ce que le mod fait à chaque image.
fn chaque_image(etat: &mut Etat) {
    // Données de PNJ de Melina : levée de ses conditions d'apparition (une seule fois).
    melina::regler_donnees_pnj();

    // 1) Touches de vitesse.
    if input::is_key_pressed(TOUCHE_F6) {
        etat.vitesse.accelerer();
    }
    if input::is_key_pressed(TOUCHE_F5) {
        etat.vitesse.ralentir();
    }
    if input::is_key_pressed(TOUCHE_F7) {
        etat.vitesse.remettre_normale();
    }

    // 1 bis) F9 : noter la position du joueur.
    if input::is_key_pressed(TOUCHE_F9)
        && let Some(p) = position_du_joueur()
    {
        noter(&format!(
            "F9 : position du joueur ({:.2}, {:.2}, {:.2}), orientation {:.1}°.",
            p.0, p.1, p.2, orientation_du_joueur().unwrap_or(0.0)
        ));
        if let Some((carte, origine, morceau)) = position_dans_la_carte() {
            noter(&format!(
                "     carte {carte}, origine {origine}, position fixe ({:.2}, {:.2}, {:.2}).",
                morceau.0, morceau.1, morceau.2
            ));
        }
    }

    // 1 bis bis) F11 : début / fin du suivi des effets (outil de mesure, voir suivi.rs).
    if input::is_key_pressed(TOUCHE_F11) {
        etat.suivi.basculer();
    }
    etat.suivi.mettre_a_jour();

    // 1 ter) F10 : position de Melina dans le journal.
    if input::is_key_pressed(TOUCHE_F10) {
        etat.melina.noter_etat();
    }

    // 2) Où est le personnage ? (rien à faire tant qu'il n'est pas chargé)
    let dans_table_ronde = match personnage_dans_table_ronde() {
        Some(dedans) => dedans,
        None => {
            etat.personnage_apparu = None; // écran de chargement ou menu principal
            etat.depart_en_cours = false;
            etat.arrivee_table_ronde = None;
            etat.placements = 0;
            etat.dernier_placement = None;
            etat.position_avant_placement = None;
            etat.gravite_coupee = false;
            // Nouveau chargement moins de 20 s après un placement : sans doute une mort due
            // au placement (chute). On arrête de placer jusqu'au retour à la Table ronde.
            if let Some(t) = etat.placement_precedent.take()
                && t.elapsed() < SECURITE_MORT_APRES_PLACEMENT
                && !etat.placement_suspendu
            {
                etat.placement_suspendu = true;
                noter("Sécurité : chargement juste après un placement (mort ?), placement suspendu jusqu'au retour à la Table ronde.");
            }
            etat.chargement_annonce = false;
            etat.melina.oublier();
            etat.menu.fermer();
            etat.vitesse.appliquer(false);
            hud::afficher_vitesse(0);
            hud::afficher_galerie(None);
            return;
        }
    };
    if etat.personnage_apparu.is_none() {
        etat.chargements_depuis_depart += 1; // nouveau chargement (arrivée ou mort)
    }
    let apparu = *etat.personnage_apparu.get_or_insert_with(Instant::now);

    // Fondu au noir : retour à l'image une fois arrivé, après le placement devant la brume
    // s'il y en a un, et au plus tard après FONDU_MAX.
    if etat.fondu_actif {
        let placement_prevu = !dans_table_ronde
            && !etat.placement_suspendu
            && etat.boss_vise.is_some_and(|a| a.placement)
            && etat.placements < ESSAIS_DE_PLACEMENT;
        let pret = if placement_prevu {
            etat.dernier_placement.is_some_and(|t| t.elapsed() >= FONDU_APRES_PLACEMENT)
        } else {
            apparu.elapsed() >= FONDU_APRES_ARRIVEE
        };
        if pret || apparu.elapsed() >= FONDU_MAX {
            fondu::eclaircir();
            etat.fondu_actif = false;
        }
    }

    // 3) Vitesse : 100 % à la Table ronde, vitesse choisie ailleurs.
    // Un chargement lancé en vitesse augmentée peut rester bloqué à 0 % : la vitesse revient
    // à 100 % dès qu'un chargement s'annonce (téléportation par une grâce, mort), et la vitesse
    // choisie ne revient qu'une fois le chargement sûrement terminé.
    if !etat.chargement_annonce && chargement_annonce() {
        etat.chargement_annonce = true;
        noter("Chargement annoncé (téléportation ou mort) : vitesse remise à 100 %.");
    }
    let vitesse_autorisee = !dans_table_ronde
        && !etat.chargement_annonce
        && apparu.elapsed() >= ATTENTE_APRES_CHARGEMENT;
    etat.vitesse.appliquer(vitesse_autorisee);
    // Vitesse à l'écran seulement quand elle est vraiment appliquée (pas à la Table ronde,
    // ni pendant le fondu au noir, ni autour d'un chargement où elle est remise à 100 %).
    let montrer = vitesse_autorisee && !etat.fondu_actif;
    hud::afficher_vitesse(if montrer { etat.vitesse.choisie } else { 0 });

    // Position d'arrivée à la Table ronde (premier instant, devant la table).
    if dans_table_ronde && etat.arrivee_table_ronde.is_none() {
        etat.arrivee_table_ronde = position_du_joueur();
    }

    // Pendant la gravité coupée : aucune chute comptée ; puis gravité rétablie.
    if etat.gravite_coupee
        || etat
            .dernier_placement
            .is_some_and(|t| t.elapsed() < GRAVITE_COUPEE + CHUTE_ANNULEE_APRES)
            && etat.placements > 0
    {
        arrivees::annuler_la_chute();
    }
    if etat.dernier_placement.is_some_and(|t| t.elapsed() >= GRAVITE_COUPEE)
        && etat.gravite_coupee
    {
        arrivees::retablir_la_gravite();
        etat.gravite_coupee = false;
    }

    // Devant un boss : placer le joueur devant le mur de brume (arrivée et après une mort).
    if dans_table_ronde && !etat.depart_en_cours {
        etat.boss_vise = None; // revenu à la Table ronde
        etat.placement_suspendu = false;
    } else if !dans_table_ronde
        && !etat.placement_suspendu
        && let Some(arrivee) = etat.boss_vise
        && arrivee.placement
        && (arrivee.apres_mort || etat.chargements_depuis_depart <= 1)
        && etat.placements < ESSAIS_DE_PLACEMENT
        && apparu.elapsed()
            >= (if arrivee.repere_exige.is_some() {
                ATTENTE_AVANT_PLACEMENT_MONDE_OUVERT
            } else {
                ATTENTE_AVANT_PLACEMENT
            })
        && etat.dernier_placement.is_none_or(|t| t.elapsed() >= INTERVALLE_VERIFICATION)
    {
        match arrivees::placer_le_joueur(arrivee, etat.position_avant_placement) {
            arrivees::Placement::Place(avant) => {
                etat.placements += 1;
                etat.dernier_placement = Some(Instant::now());
                etat.placement_precedent = Some(Instant::now());
                etat.gravite_coupee = true;
                etat.position_avant_placement.get_or_insert(avant);
            }
            arrivees::Placement::RienAFaire => {
                etat.placements += 1;
                etat.dernier_placement = Some(Instant::now());
            }
            arrivees::Placement::PasPret(carte) => {
                if apparu.elapsed() >= ATTENTE_MAX_PLACEMENT {
                    noter(&format!(
                        "{} : carte jamais reconnue après le chargement (le jeu dit : {}), placement abandonné.",
                        arrivee.nom,
                        carte.as_deref().unwrap_or("personnage pas chargé")
                    ));
                    etat.placements = ESSAIS_DE_PLACEMENT;
                }
            }
        }
    }

    // Rien d'autre tant que le chargement n'est pas sûrement terminé.
    if apparu.elapsed() < ATTENTE_APRES_CHARGEMENT {
        return;
    }

    // Effets sauvegardés : remis une fois par session, puis le fichier suit chaque
    // changement (voir reglages.rs).
    etat.reglages.mettre_a_jour(&mut etat.menu);
    // Effet d'arme choisi chez Melina : remis s'il a disparu (mort, chargement).
    etat.menu.effets_arme.mettre_a_jour();
    // Galerie des sorts (voir galerie.rs).
    etat.menu.galerie.mettre_a_jour();
    // Impact sur l'ennemi touché (voir impact.rs).
    etat.menu.impact.mettre_a_jour();
    etat.menu.impact_gauche.mettre_a_jour();
    // Arme fantôme pendant les attaques (voir arme_fantome.rs).
    etat.menu.arme_fantome.mettre_a_jour();
    etat.menu.arme_fantome_gauche.mettre_a_jour();
    // Mannequin de la Table ronde : PV rendus (voir mannequin.rs).
    etat.mannequin.mettre_a_jour();
    // Traînée de course, avant les roulades (lignes partagées, voir course.rs).
    etat.menu.course.mettre_a_jour(&mut etat.menu.roulade);
    // Style de roulade choisi chez Melina (voir roulade.rs).
    etat.menu.roulade.mettre_a_jour();
    // Aura choisie chez Melina (voir aura.rs).
    etat.menu.aura.mettre_a_jour();

    // 4) La grâce de la Table ronde, cherchée une fois dans les données du jeu.
    if etat.grace_table_ronde.is_none() {
        etat.grace_table_ronde = teleportation::grace_de_la_table_ronde();
        match etat.grace_table_ronde {
            Some(numero) => noter(&format!("Grâce de la Table ronde : {numero}")),
            None => {
                noter("Grâce de la Table ronde introuvable, nouvel essai plus tard.");
                return;
            }
        }
    }
    let Some(grace) = etat.grace_table_ronde else {
        return;
    };

    // 5) Envoi automatique à la Table ronde, une fois par session, après le chargement.
    //    MESURE_PARTIE_NORMALE (temporaire) : ni déblocage ni envoi, pour mesurer le début du
    //    jeu tel quel.
    if MESURE_PARTIE_NORMALE && !etat.arrivee_faite {
        etat.arrivee_faite = true;
        noter("MESURE : partie normale, ni déblocage ni envoi à la Table ronde (à retirer).");
    }
    if !etat.arrivee_faite {
        etat.arrivee_faite = true;
        if !deblocage::tout_debloquer() {
            noter("Déblocage : données du jeu pas prêtes, rien n'a été fait.");
        }
        // Essais du 08/11/2026 (effets_arme::autoriser_les_boucliers) : bâtons, sceaux et
        // boucliers n'affichent pas les effets d'arme même autorisés : retiré.
        if !deblocage::debloquer_les_talismans() {
            noter("Talismans : personnage pas prêt, rien n'a été fait.");
        }
        if dans_table_ronde {
            noter("Partie chargée : déjà à la Table ronde.");
        } else {
            noter("Partie chargée : envoi à la Table ronde.");
            partir_vers(etat, grace, dans_table_ronde);
        }
    }

    // 5 bis) Melina et son menu, à la Table ronde.
    if dans_table_ronde {
        etat.melina.mettre_a_jour();
        if etat.menu.est_ouvert() {
            if let Some(grace_du_boss) = etat.menu.mettre_a_jour(&mut etat.vitesse) {
                noter(&format!("Départ vers le boss (vitesse {} %).", etat.vitesse.choisie));
                // Boss avec un point d'arrivée devant la brume : départ de la bonne
                // grâce, puis placement du joueur (voir arrivees.rs).
                let mut grace_de_depart = grace_du_boss;
                etat.boss_vise = None;
                etat.chargements_depuis_depart = 0;
                if let Some(arrivee) = arrivees::pour_le_boss(grace_du_boss) {
                    match arrivees::grace_de_depart(arrivee) {
                        Some(g) => {
                            grace_de_depart = g;
                            etat.boss_vise = Some(arrivee);
                            noter(&format!(
                                "{} : départ de la grâce {g}, puis placement devant la brume.",
                                arrivee.nom
                            ));
                        }
                        None => noter(&format!(
                            "{} : grâce de départ introuvable, départ vers la grâce du menu.",
                            arrivee.nom
                        )),
                    }
                }
                partir_vers(etat, grace_de_depart, true);
            }
        } else if let Some(joueur) = position_du_joueur()
            && etat.melina.joueur_assez_pres(joueur)
            && let Ok(invites) = unsafe { CSActionButtonManImp::instance_mut() }
            && invites.present_action_button(INVITE_PARLER)
            && let Some(melina) = etat.melina.personnage()
        {
            etat.menu.ouvrir(melina.field_ins_handle);
        }
    }

    // 6) F8 : retour à la Table ronde.
    if input::is_key_pressed(TOUCHE_F8) && !dans_table_ronde {
        noter("F8 : retour à la Table ronde.");
        partir_vers(etat, grace, dans_table_ronde);
    }

}

/// Toute téléportation du mod passe par ici.
fn partir_vers(etat: &mut Etat, grace: u32, _depuis_table_ronde: bool) {
    if teleportation::aller_a_la_grace(grace) {
        etat.depart_en_cours = true;
        fondu::noircir();
        etat.fondu_actif = true;
    }
}

/// Carte du joueur, carte d'origine du repère, et position dans le repère fixe des
/// morceaux de carte (chunk_position). Contrairement à la position physique, celle-ci ne
/// se décale pas d'une partie à l'autre : on s'en sert pour noter des emplacements.
fn position_dans_la_carte() -> Option<(String, String, (f32, f32, f32))> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    let chr = &joueur.chr_ins;
    let c = chr.chunk_position;
    Some((format!("{}", chr.block_id), format!("{}", chr.block_origin), (c.0, c.1, c.2)))
}

/// Direction vers laquelle regarde le joueur, en degrés autour de l'axe vertical.
fn orientation_du_joueur() -> Option<f32> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    Some(crate::melina::angle_vertical(joueur.chr_ins.modules.physics.orientation))
}

/// Position du joueur dans le monde (coordonnées physiques).
fn position_du_joueur() -> Option<HavokPosition> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    Some(joueur.chr_ins.modules.physics.position)
}

/// Some(vrai) si le personnage est à la Table ronde, Some(faux) s'il est ailleurs,
/// None s'il n'est pas chargé.
/// Vrai quand un chargement va commencer : téléportation demandée au jeu (grâce, retour
/// à la Table ronde…) ou joueur mort (réapparition à la dernière grâce).
fn chargement_annonce() -> bool {
    let teleportation = unsafe { GameMan::instance() }.is_ok_and(|jeu| jeu.warp_requested);
    let mort = unsafe { WorldChrMan::instance() }
        .ok()
        .and_then(|monde| monde.main_player.as_ref())
        .is_some_and(|joueur| joueur.chr_ins.modules.data.hp <= 0);
    teleportation || mort
}

fn personnage_dans_table_ronde() -> Option<bool> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    let carte = &joueur.chr_ins.block_id;
    Some(carte.area() == ZONE_TABLE_RONDE && carte.block() == BLOC_TABLE_RONDE)
}
