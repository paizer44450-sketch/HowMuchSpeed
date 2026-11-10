//! Styles de roulade : purement visuels, la roulade du jeu (distance, timing, frames
//! d'invincibilité) ne change pas.
//!
//! Mesures F11 du 07/11/2026 :
//! - roulade du joueur : animations 27100 à 27199 (a00.tae), qui posent l'effet 430
//!   (frames d'invincibilité) de 0 à 0,5 s. Le Pas de chevalier-limier pose aussi 430 : on
//!   regarde donc aussi le numéro d'animation pour ne réagir qu'aux roulades.
//! - téléportation de Radagon (PNJ 21900078, c2190.tae, animations 20001 / 20006) :
//!   visuels 621985 et 621986 (point 220), puis 621987 à l'arrivée (point 900). Le joueur
//!   garde 621985 + 621986 ensemble ; 621987 « explose et éblouit » (retiré).
//! - Pas de chevalier-limier (a756.tae, animations 40580 / 40680) : visuel 440310 (point
//!   900) de 0,03 à 0,37 s avec le son 440310, visuel 440315 (point 900) de 0,47 à 0,80 s,
//!   opacité 1 → 0 (0,03 à 0,17 s), 0 jusqu'à 0,33 s, 0 → 1 (0,33 à 0,60 s), armes cachées
//!   de 0,17 à 0,33 s.
//! - esquive des assassins du Couteau noir (PNJ 21000034, c2100.tae) : animation 8190 /
//!   8191 (départ) qui pose l'effet 14507 (1,1 s, visuel 52252 : camouflage, sons de
//!   c2100), puis le saut 6012 / 6013. Aucun FFX : tout le rendu est le camouflage du
//!   visuel 52252, recopié tel quel. Essai 0.44.0 : sur le joueur, ce camouflage ne se
//!   voit pas (effet bien posé d'après le journal). Le camouflage dépend de la distance
//!   (début 20 m, fin 19 m) : fait pour que les autres ne le voient plus. On garde le
//!   visuel (sons) et on fait l'effacement nous-mêmes par l'opacité.
//! - téléportation du Revenant royal (PNJ 40000020, c4000.tae) : animation 10010
//!   (disparition : visuel commun 201 au point 960 de 1,0 à 2,8 s, opacité 1 → 0 de 2,27 à
//!   3,5 s) puis 10001 (réapparition). Sur 4,9 s chez lui : resserré sur la durée de la
//!   roulade, visuel au centre du corps (le point 960 est propre à son modèle).
//! - téléportation du Chevalier de la mort (cendre de guerre du joueur, a914.tae,
//!   animation 40000, effet 430 à partir de 0,467 s) : visuel 441052 (point 905) de 0,17 à
//!   0,63 s, son et visuel 441050 (point 900) à 0,53 / 0,63 s, visuel 441051 (point 900)
//!   de 0,63 à 0,97 s, opacité 1 → 0 (0,53 à 0,63 s), 0 jusqu'à 1,03 s, 0 → 1 (1,03 à
//!   1,13 s). Recalé sur le début de la roulade (= début de 430), la préparation
//!   (441052) resserrée au tout début.
//! - sorcellerie Ténèbres éternelles (MagicParam 4650) : visuel de lancement 523250,
//!   visuel au tir 523251, projectile 10465000 / 10465001 (visuel 523252 : le trou noir,
//!   rayon de 4 m). Un sort n'a pas de visuel dans un effet : on lit ses visuels dans
//!   MagicParam (castSfxId, fireSfxId) et dans la fiche de son projectile (BulletParam,
//!   sfxId_Bullet / sfxId_Hit / sfxId_Flick, en suivant HitBulletID).
//! - autres sorts (lus de la même façon) : Champ de force de Thops (4630, dôme 523212),
//!   Pleine lune de Rennala (4360, lune 523803 puis éclatement 523804 ; les Lunes
//!   jumelles de Rellana n'ont aucun visuel dans leurs projectiles), Projectile
//!   gravitationnel (2004710, tir 551011, projectile 551012), Tir de magma (4800 :
//!   boule 524502, éclaboussure 524503 à l'impact ; éclats 524504 et flaques 524505 au
//!   sol laissés de côté).
//! - sorts du 07/11/2026 (projectiles seulement, sans les visuels de lancement ni les
//!   explosions d'impact, trop gros sur les essais précédents) : Halos de lumière de
//!   Radagon (6710 : projectiles sans visuel en vol, seulement les éclats d'impact
//!   528133 / 528138 ; les anneaux doivent être dans l'animation), Boule de feu noire
//!   (Flamme noire 6210, boule 524542), Serpent de feu (2007800, 542004 et 542009),
//!   Pierre de Gurranq (6810, rocher 529502), Disque de lumière (6700, 528122 et 528124),
//!   Foudroiement (6920, éclair 525572 du projectile 10692010).
//! - retirés le 07/11/2026 : Ronces du péché (8000, ronces 523912 / 523913 / 523914 :
//!   « énorme truc ») et Papillons putrescents (2007200 : projectiles sans visuel ; le tir
//!   523231 et l'effet 523234 du sort ne donnent qu'un petit éclat bleu, les papillons
//!   doivent être joués par l'animation du sort, a535.tae).
//!
//! Lignes partagées entre styles : un seul style est actif, et ses lignes sont remplies
//! de nouveau quand on change de style (`prepare_pour`). Lignes d'effet des roulades :
//! 7250 à 7280 seulement (7210 / 7220 : effets d'arme, 7230 / 7240 : aura). Elles servent
//! aussi aux traînées de course (course.rs), jamais en même temps : la traînée s'arrête
//! dès qu'une roulade commence, et ne réécrit les lignes que hors roulade.
//!
//! Chaque visuel est un visuel CONTINU porté par une ligne d'effet inutilisée du jeu
//! (copie de l'effet neutre 18553 : aucune statistique) ; on pose l'effet au début de sa
//! plage et on l'enlève à la fin, ce qui coupe le visuel. Leçons du premier essai
//! (0.42.0) : un visuel de début / de fin d'effet n'est jamais coupé (paillettes laissées
//! à l'infini), et les lignes « Area Scaling » ne sont pas vides (PV, attaque, défenses et
//! résistances multipliées) : toujours les recopier depuis l'effet neutre.

use std::time::Instant;

use eldenring::cs::{ChrInsExt, SoloParamRepository, SpEffectParam, SpEffectVfxParam, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Effet que le jeu pose sur le joueur pendant la roulade (frames d'invincibilité).
pub(crate) const EFFET_ROULADE_DU_JEU: i32 = 430;
/// Animations de roulade (toutes directions et charges).
pub(crate) const ANIMATIONS_ROULADE: std::ops::Range<i32> = 27100..27200;
/// Effet de Radagon sans aucune statistique (mesure du 07/11/2026) : modèle des lignes.
const EFFET_NEUTRE: u32 = 18553;

/// Un visuel joué pendant la roulade.
struct Piste {
    /// Ligne d'effet inutilisée (« Area Scaling - (Unused) »).
    ligne: u32,
    /// Ligne de visuel inutilisée (présente une seule fois dans regulation.bin).
    visuel: u32,
    /// Visuel du jeu recopié tel quel (0 = aucun : visuel construit avec `sfx`, `point`,
    /// `son`).
    modele: u32,
    /// Visuel (FFX) et son point d'attache sur le personnage.
    sfx: i32,
    point: i16,
    /// Son joué au début (-1 = aucun).
    son: i32,
    /// Plage de temps (s) depuis le début de la roulade.
    debut: f32,
    fin: f32,
}

/// Styles de roulade proposés dans le menu.
#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    Normale,
    Radagon,
    Limier,
    CouteauNoir,
    Revenant,
    ChevalierMort,
    TenebresEternelles,
    ChampThops,
    PleineLune,
    ProjectileGravitationnel,
    TirMagma,
    HalosRadagon,
    FlammeNoire,
    SerpentFeu,
    PierreGurranq,
    DisqueLumiere,
    Foudroiement,
}

impl Style {
    pub fn nom(self) -> &'static str {
        match self {
            Style::Normale => "normale",
            Style::Radagon => "téléportation de Radagon",
            Style::Limier => "pas de chevalier-limier",
            Style::CouteauNoir => "esquive du Couteau noir",
            Style::Revenant => "téléportation du Revenant royal",
            Style::ChevalierMort => "téléportation du Chevalier de la mort",
            Style::TenebresEternelles => "ténèbres éternelles",
            Style::ChampThops => "champ de force de Thops",
            Style::PleineLune => "pleine lune de Rennala",
            Style::ProjectileGravitationnel => "projectile gravitationnel",
            Style::TirMagma => "tir de magma",
            Style::HalosRadagon => "halos de lumière de Radagon",
            Style::FlammeNoire => "boule de feu noire",
            Style::SerpentFeu => "serpent de feu",
            Style::PierreGurranq => "pierre de Gurranq",
            Style::DisqueLumiere => "disque de lumière",
            Style::Foudroiement => "foudroiement",
        }
    }

    fn pistes(self) -> &'static [Piste] {
        match self {
            Style::Normale => &[],
            Style::Radagon => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 621985, point: 220, son: -1, debut: 0.0, fin: 0.53 },
                Piste { modele: 0, ligne: 7260, visuel: 57022, sfx: 621986, point: 220, son: -1, debut: 0.0, fin: 0.53 },
            ],
            Style::Limier => &[
                Piste { modele: 0, ligne: 7270, visuel: 57021, sfx: 440310, point: 900, son: 440310, debut: 0.03, fin: 0.37 },
                Piste { modele: 0, ligne: 7280, visuel: 57020, sfx: 440315, point: 900, son: -1, debut: 0.47, fin: 0.80 },
            ],
            Style::CouteauNoir => &[
                Piste { modele: 52252, ligne: 7250, visuel: 50121, sfx: -1, point: -1, son: -1, debut: 0.0, fin: 1.1 },
            ],
            Style::Revenant => &[
                Piste { modele: 0, ligne: 7250, visuel: 50958, sfx: 201, point: 220, son: -1, debut: 0.0, fin: 0.6 },
            ],
            Style::ChevalierMort => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 441052, point: 905, son: -1, debut: 0.0, fin: 0.17 },
                Piste { modele: 0, ligne: 7260, visuel: 57022, sfx: 441050, point: 900, son: 441050, debut: 0.07, fin: 0.27 },
                Piste { modele: 0, ligne: 7270, visuel: 57021, sfx: 441051, point: 900, son: -1, debut: 0.17, fin: 0.50 },
            ],
            // Essai 0.48.1 (un visuel retiré à tour de rôle) : le joueur ne garde que 523251,
            // le même trou noir que 523252 mais sans le son du lancement ; 523250
            // (lancement) et 523252 retirés.
            Style::TenebresEternelles => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 523251, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::ChampThops => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 523212, point: 220, son: -1, debut: 0.0, fin: 0.6 },
            ],
            // L'éclatement 523804 à l'arrivée « fait un peu beaucoup » (joueur) : retiré.
            Style::PleineLune => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 523803, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            // Le son vient du visuel de tir 551011 (le son est dans le visuel lui-même, comme
            // pour le lancement de Ténèbres éternelles) : retiré à la demande du joueur.
            Style::ProjectileGravitationnel => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 551012, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            // L'éclaboussure 524503 à l'arrivée est trop grosse (joueur) : retirée.
            Style::TirMagma => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 524502, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::HalosRadagon => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 528133, point: 220, son: -1, debut: 0.05, fin: 0.6 },
                Piste { modele: 0, ligne: 7260, visuel: 57022, sfx: 528138, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::FlammeNoire => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 524542, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::SerpentFeu => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 542004, point: 220, son: -1, debut: 0.05, fin: 0.6 },
                Piste { modele: 0, ligne: 7260, visuel: 57022, sfx: 542009, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::PierreGurranq => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 529502, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::DisqueLumiere => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 528122, point: 220, son: -1, debut: 0.05, fin: 0.6 },
                Piste { modele: 0, ligne: 7260, visuel: 57022, sfx: 528124, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
            Style::Foudroiement => &[
                Piste { modele: 0, ligne: 7250, visuel: 57999, sfx: 525572, point: 220, son: -1, debut: 0.05, fin: 0.6 },
            ],
        }
    }

    /// Opacité du personnage : (temps en s, opacité), interpolée entre les points ; 1 après
    /// le dernier point.
    fn opacite(self) -> &'static [(f32, f32)] {
        match self {
            Style::Normale => &[],
            Style::Radagon => &[(0.0, 0.0), (0.53, 0.0)],
            Style::Limier => &[(0.0, 1.0), (0.03, 1.0), (0.17, 0.0), (0.33, 0.0), (0.60, 1.0)],
            // Presque invisible pendant 1,1 s, comme l'assassin.
            Style::CouteauNoir => &[(0.0, 1.0), (0.1, 0.1), (0.9, 0.1), (1.1, 1.0)],
            Style::Revenant => &[(0.0, 1.0), (0.12, 0.0), (0.4, 0.0), (0.6, 1.0)],
            Style::ChevalierMort => &[(0.0, 1.0), (0.07, 1.0), (0.17, 0.0), (0.57, 0.0), (0.67, 1.0)],
            Style::TenebresEternelles
            | Style::ChampThops
            | Style::PleineLune
            | Style::ProjectileGravitationnel
            | Style::TirMagma
            | Style::HalosRadagon
            | Style::FlammeNoire
            | Style::SerpentFeu
            | Style::PierreGurranq
            | Style::DisqueLumiere
            | Style::Foudroiement => &[(0.0, 1.0), (0.12, 0.0), (0.45, 0.0), (0.6, 1.0)],
        }
    }

    /// Durée totale de l'effet visuel.
    fn duree(self) -> f32 {
        let fin_pistes = self.pistes().iter().map(|p| p.fin).fold(0.0, f32::max);
        let fin_opacite = self.opacite().last().map(|&(t, _)| t).unwrap_or(0.0);
        fin_pistes.max(fin_opacite)
    }
}

/// Remplit une ligne de visuel : un visuel continu (`sfx`) sur le point `point` du
/// personnage, un son au début (-1 = aucun), rien d'autre. Sert aussi aux traînées de
/// course (course.rs).
pub(crate) fn remplir_visuel(
    v: &mut eldenring::param::SP_EFFECT_VFX_PARAM_ST,
    sfx: i32,
    point: i16,
    son: i32,
    armes_cachees: bool,
) {
    v.set_init_sfx_id(-1);
    v.set_init_se_id(son);
    v.set_init_dmy_id(point);
    v.set_midst_sfx_id(sfx);
    v.set_midst_se_id(-1);
    v.set_midst_dmy_id(point);
    v.set_finish_sfx_id(-1);
    v.set_finish_se_id(-1);
    v.set_finish_dmy_id(-1);
    v.set_effect_type(0);
    v.set_soul_param_id_for_wep_enchant(0);
    v.set_trace_sfx_id_offset_type(0);
    v.set_use_camouflage(false);
    v.set_is_invisible_weapon(armes_cachees);
    v.set_enchant_start_dmy_id_0(-1);
    v.set_enchant_end_dmy_id_0(-1);
    v.set_enchant_start_dmy_id_1(-1);
    v.set_enchant_end_dmy_id_1(-1);
    v.set_enchant_start_dmy_id_2(-1);
    v.set_enchant_end_dmy_id_2(-1);
    v.set_enchant_start_dmy_id_3(-1);
    v.set_enchant_end_dmy_id_3(-1);
    v.set_enchant_start_dmy_id_4(-1);
    v.set_enchant_end_dmy_id_4(-1);
    v.set_enchant_start_dmy_id_5(-1);
    v.set_enchant_end_dmy_id_5(-1);
    v.set_enchant_start_dmy_id_6(-1);
    v.set_enchant_end_dmy_id_6(-1);
    v.set_enchant_start_dmy_id_7(-1);
    v.set_enchant_end_dmy_id_7(-1);
}

/// Opacité à l'instant `t` d'après les points (interpolation linéaire).
fn opacite_a(points: &[(f32, f32)], t: f32) -> f32 {
    let Some(&(t0, o0)) = points.first() else {
        return 1.0;
    };
    if t <= t0 {
        return o0;
    }
    for paire in points.windows(2) {
        let ((ta, oa), (tb, ob)) = (paire[0], paire[1]);
        if t <= tb {
            if tb <= ta {
                return ob;
            }
            return oa + (ob - oa) * (t - ta) / (tb - ta);
        }
    }
    1.0
}

pub struct Roulade {
    style: Style,
    /// Style dont les lignes d'effet et de visuel sont prêtes.
    prepare_pour: Option<Style>,
    /// Début de la roulade en cours.
    debut: Option<Instant>,
    /// Pistes actuellement posées (même ordre que `pistes()`).
    posees: Vec<bool>,
}

/// Les styles dans l'ordre du menu « Roulade » (lignes 1 à 17), pour la sauvegarde.
pub const STYLES_MENU: [Style; 17] = [
    Style::Normale,
    Style::Radagon,
    Style::Limier,
    Style::CouteauNoir,
    Style::Revenant,
    Style::ChevalierMort,
    Style::TenebresEternelles,
    Style::ChampThops,
    Style::PleineLune,
    Style::ProjectileGravitationnel,
    Style::TirMagma,
    Style::HalosRadagon,
    Style::FlammeNoire,
    Style::SerpentFeu,
    Style::PierreGurranq,
    Style::DisqueLumiere,
    Style::Foudroiement,
];

impl Roulade {
    /// Style actuel (lu par reglages.rs pour la sauvegarde).
    pub fn style(&self) -> Style {
        self.style
    }

    pub fn new() -> Self {
        Self { style: Style::Normale, prepare_pour: None, debut: None, posees: Vec::new() }
    }

    /// Vrai pendant une roulade stylée (ses visuels peuvent encore être posés).
    pub fn en_cours(&self) -> bool {
        self.debut.is_some()
    }

    /// Les lignes partagées ont été réécrites par la traînée de course : elles seront
    /// remplies de nouveau à la prochaine roulade.
    pub fn oublier_preparation(&mut self) {
        self.prepare_pour = None;
    }

    pub fn choisir(&mut self, style: Style) {
        self.style = style;
        noter(&format!("Roulade : {}.", style.nom()));
    }

    /// Remplit les lignes d'effet (copies de l'effet neutre, permanentes, sans catégorie :
    /// n'enlèvent aucun autre effet) et leurs lignes de visuel (un visuel continu, rien
    /// d'autre).
    fn preparer(style: Style) -> bool {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(neutre) = parametres.get::<SpEffectParam>(EFFET_NEUTRE).cloned() else {
            noter(&format!("Roulade : effet neutre {EFFET_NEUTRE} introuvable."));
            return false;
        };
        for p in style.pistes() {
            if p.modele != 0 {
                let Some(modele) = parametres.get::<SpEffectVfxParam>(p.modele).cloned() else {
                    noter(&format!("Roulade : visuel modèle {} introuvable.", p.modele));
                    return false;
                };
                let Some(v) = parametres.get_mut::<SpEffectVfxParam>(p.visuel) else {
                    noter(&format!("Roulade : ligne de visuel {} introuvable.", p.visuel));
                    return false;
                };
                *v = modele;
            } else {
                let Some(v) = parametres.get_mut::<SpEffectVfxParam>(p.visuel) else {
                    noter(&format!("Roulade : ligne de visuel {} introuvable.", p.visuel));
                    return false;
                };
                // Armes cachées tant que le personnage est invisible.
                remplir_visuel(v, p.sfx, p.point, p.son, true);
            }
            let Some(ligne) = parametres.get_mut::<SpEffectParam>(p.ligne) else {
                noter(&format!("Roulade : ligne d'effet {} introuvable.", p.ligne));
                return false;
            };
            *ligne = neutre.clone();
            ligne.set_effect_endurance(-1.0);
            ligne.set_sp_category(0);
            ligne.set_vfx_id(p.visuel as i32);
        }
        noter(&format!("Roulade : effets « {} » prêts.", style.nom()));
        true
    }

    /// À appeler à chaque image.
    pub fn mettre_a_jour(&mut self) {
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        if self.debut.is_none() {
            let style = self.style;
            if style == Style::Normale {
                return;
            }
            let ta = &joueur.chr_ins.modules.time_act;
            let animation = ta.anim_queue[(ta.read_idx as usize) % ta.anim_queue.len()].anim_id;
            let roule = ANIMATIONS_ROULADE.contains(&animation)
                && joueur
                    .chr_ins
                    .special_effect
                    .entries()
                    .any(|e| e.param_id == EFFET_ROULADE_DU_JEU);
            if !roule {
                return;
            }
            if self.prepare_pour != Some(style) {
                if !Self::preparer(style) {
                    return;
                }
                self.prepare_pour = Some(style);
            }
            self.debut = Some(Instant::now());
            self.posees = vec![false; style.pistes().len()];
        }
        let (Some(debut), Some(style)) = (self.debut, self.prepare_pour) else {
            return;
        };
        // On suit le style pour lequel les lignes sont prêtes : si le joueur change de style
        // en pleine roulade, celle-ci finit avec l'ancien.
        let t = debut.elapsed().as_secs_f32();
        let fini = t > style.duree();
        for (i, p) in style.pistes().iter().enumerate() {
            let doit_etre_posee = !fini && t >= p.debut && t < p.fin;
            if doit_etre_posee != self.posees[i] {
                if doit_etre_posee {
                    joueur.apply_speffect(p.ligne as i32, true);
                } else {
                    joueur.chr_ins.remove_speffect(p.ligne as i32);
                }
                self.posees[i] = doit_etre_posee;
            }
        }
        joueur.chr_ins.base_transparency = if fini { 1.0 } else { opacite_a(style.opacite(), t) };
        if fini {
            self.debut = None;
        }
    }
}
