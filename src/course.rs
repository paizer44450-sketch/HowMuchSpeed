//! Traînée de course : un visuel de sort porté par le personnage tant qu'il sprinte.
//! Purement visuel : aucun dégât, aucune statistique.
//!
//! Sprint : le jeu note la demande de sprint du joueur (bouton de sprint tenu en se
//! déplaçant) dans son module d'action (`movement_request_flags.dash`). On attend
//! DELAI_AVANT_TRAINEE après le début de la demande, pour ne pas allumer la traînée sur un
//! simple appui de roulade, et on l'éteint pendant les roulades. À bout d'endurance le
//! personnage trottine, bouton tenu : la traînée reste.
//!
//! Visuels lus dans MagicParam et dans les fiches des projectiles des sorts (BulletParam,
//! en suivant HitBulletID), mesures F11 du 07/11/2026. Essai 0.53.1 : chaque visuel proposé
//! seul, le joueur a gardé :
//! - Éclat nocturne (4610) : l'impact 523243, « à chaque pas » → relancé (voir plus bas).
//! - Spirale d'éclats (4030) : l'éclat 523087.
//! - Tir de magma (4800), deux versions : boule 524502 + flaque 524505, et flaque 524505 +
//!   éclats 524504.
//! - Serpent de feu (2007800) : serpent 542004 + impact 542003.
//! - Spira (2007720) : spirale 541322 (la fin 541323, « trop », retirée) ; restait sur
//!   place → relancée.
//! - Rituel des flammes noires (6240) : flamme 524072 (premier essai, gardée).
//! - Pierre de Gurranq (6810) : l'impact 529503 « pour le moment » → relancé.
//! - Lames de roche (2004700) : l'effet du sort 551004 ; restait sur place → relancé.
//!
//! Visuels relancés : certains visuels de sort restent là où ils sont apparus, ou ne jouent
//! qu'une fois. On les repose régulièrement (temps de jeu) à la position du personnage :
//! soit un seul exemplaire (`Relance::Simple`), soit deux groupes de lignes qui se relaient
//! (`Relance::Alternee`) : le nouvel exemplaire apparaît pendant que le précédent finit, ce
//! qui laisse une traînée. (Enlever puis remettre un effet dans la même image ne relance
//! pas son visuel : on le remet à l'image suivante.) Le son d'Éclat nocturne et de Lames de
//! roche est DANS le visuel (FFX) : il rejoue à chaque relance, et aucun réglage des
//! paramètres ne le coupe (les sons du visuel, init / midst_se_id, sont déjà à -1).
//!
//! Lignes : les mêmes que les roulades (effets 7250 à 7280, visuels 57999, 57022, 57021,
//! 57020), jamais en même temps : la traînée s'éteint dès qu'une roulade commence, ne se
//! rallume qu'une fois la roulade finie, et remplit de nouveau les lignes à chaque
//! allumage (la roulade fera de même à la suivante).

use std::time::{Duration, Instant};

use eldenring::cs::{
    CSFlipper, ChrInsExt, SoloParamRepository, SpEffectParam, SpEffectVfxParam, WorldChrMan,
};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;
use crate::roulade::{ANIMATIONS_ROULADE, EFFET_ROULADE_DU_JEU, Roulade, remplir_visuel};

/// Effet de Radagon sans aucune statistique : modèle des lignes.
const EFFET_NEUTRE: u32 = 18553;
/// Lignes d'effet et de visuel (partagées avec les roulades). Un style relancé utilise
/// deux groupes : les visuels du groupe 0 sur les premières lignes, ceux du groupe 1 juste
/// après.
const LIGNES: [(u32, u32); 4] = [(7250, 57999), (7260, 57022), (7270, 57021), (7280, 57020)];
/// Point du personnage où le visuel est accroché : 900, au sol (comme les visuels au sol
/// des cendres du joueur). Le centre du corps (220) donnait le feu au niveau du torse.
const POINT: i16 = 900;
/// Temps de sprint demandé avant d'allumer la traînée.
const DELAI_AVANT_TRAINEE: Duration = Duration::from_millis(300);
/// Intervalle habituel entre deux relances, en temps de jeu (environ un pas de sprint) :
/// raccourci d'autant quand le jeu est accéléré.
const PERIODE_RELANCE: f32 = 0.3;

/// Relance d'un visuel qui reste sur place ou ne joue qu'une fois.
#[derive(Clone, Copy, PartialEq)]
enum Relance {
    /// Visuel posé une fois, tant que dure le sprint.
    Aucune,
    /// Un seul exemplaire, enlevé puis reposé à la position du personnage toutes les
    /// `periode` secondes (temps de jeu).
    Simple { periode: f32 },
    /// Deux exemplaires qui se relaient : le nouveau apparaît pendant que l'ancien finit.
    Alternee { periode: f32 },
}

/// Un style de traînée.
pub struct Style {
    /// Texte du menu.
    pub texte: i32,
    pub nom: &'static str,
    /// Visuels du jeu, joués ensemble (deux au plus pour une relance alternée).
    visuels: &'static [i32],
    relance: Relance,
}

/// Les styles proposés, dans l'ordre du menu.
pub const STYLES: [Style; 21] = [
    // Relance deux fois plus rapide (demande du joueur, essai 0.54.0).
    Style {
        texte: 69000400,
        nom: "Éclat nocturne (à modifier)",
        visuels: &[523243],
        relance: Relance::Alternee { periode: PERIODE_RELANCE / 2.0 },
    },
    Style { texte: 69000352, nom: "Spirale d'éclats", visuels: &[523087], relance: Relance::Aucune },
    Style {
        texte: 69000398,
        nom: "Tir de magma (boule et flaque)",
        visuels: &[524502, 524505],
        relance: Relance::Aucune,
    },
    Style {
        texte: 69000399,
        nom: "Tir de magma (flaque et éclats)",
        visuels: &[524505, 524504],
        relance: Relance::Aucune,
    },
    Style { texte: 69000354, nom: "Serpent de feu", visuels: &[542004, 542003], relance: Relance::Aucune },
    Style {
        texte: 69000355,
        nom: "Spira",
        // La fin 541323 retirée : « c'est trop » (essai 0.54.0).
        visuels: &[541322],
        relance: Relance::Alternee { periode: PERIODE_RELANCE },
    },
    Style { texte: 69000356, nom: "Rituel des flammes noires", visuels: &[524072], relance: Relance::Aucune },
    Style {
        texte: 69000450,
        nom: "Pierre de Gurranq (à modifier)",
        visuels: &[529503],
        relance: Relance::Alternee { periode: PERIODE_RELANCE },
    },
    // Un seul exemplaire : deux exemplaires à la fois faisaient « un deuxième effet » (0.54.0).
    Style {
        texte: 69000451,
        nom: "Lames de roche (à modifier)",
        visuels: &[551004],
        relance: Relance::Simple { periode: PERIODE_RELANCE },
    },
    // Essais du 07/11/2026 (0.55.0), visuels gardés par le joueur :
    // Papillons putrescents (2007200) : tout (papillons 541002, suite 541003, fin 541004).
    Style {
        texte: 69000319,
        nom: "Papillons putrescents",
        visuels: &[541002, 541003, 541004],
        relance: Relance::Aucune,
    },
    // Disque de lumière (6700) : deuxième disque 528124 + impact 528123.
    Style { texte: 69000345, nom: "Disque de lumière", visuels: &[528124, 528123], relance: Relance::Aucune },
    // Comète azur (4200) : la fin 526254, « sympa », à retravailler.
    Style { texte: 69000500, nom: "Comète azur (à modifier)", visuels: &[526254], relance: Relance::Aucune },
    // Ténèbres (6530) : les deux visuels, séparément.
    Style { texte: 69000501, nom: "Ténèbres", visuels: &[523572], relance: Relance::Aucune },
    Style { texte: 69000502, nom: "Ténèbres (nuage)", visuels: &[523573], relance: Relance::Aucune },
    // Fouet de flammes noires (Scouring Black Flame, 6230) : flammes non chargées 524062.
    Style { texte: 69000503, nom: "Fouet de flammes noires", visuels: &[524062], relance: Relance::Aucune },
    // Griffe bestiale (6820, griffe 530172 relancée) : retirée, « pas terrible » (0.56.0).
    // Fronde bestiale (6800) : impact 529523, relancé toutes les 0,15 s (même rythme
    // qu'Éclat nocturne).
    Style {
        texte: 69000505,
        nom: "Fronde bestiale",
        visuels: &[529523],
        relance: Relance::Alternee { periode: PERIODE_RELANCE / 2.0 },
    },
    // Faveur sanguine (Bloodboon, 6310) : sang 524132 + impact 524133.
    Style { texte: 69000506, nom: "Faveur sanguine", visuels: &[524132, 524133], relance: Relance::Aucune },
    // Étoiles d'Elden (6720) : étoiles 525402.
    Style { texte: 69000507, nom: "Étoiles d'Elden", visuels: &[525402], relance: Relance::Aucune },
    // Glaive de Lansseax (6950) : éclair 525583, relancé toutes les 0,3 s. Son dans le
    // visuel : ne se coupe pas.
    Style {
        texte: 69000496,
        nom: "Glaive de Lansseax",
        visuels: &[525583],
        relance: Relance::Alternee { periode: PERIODE_RELANCE },
    },
    // Lame d'éclat magique (4390) : le vortex de l'incantation (essai du 08/11/2026),
    // reposé toutes les 2 s. À finir.
    Style {
        texte: 69000525,
        nom: "Lame d'éclat (à modifier)",
        visuels: &[523090],
        relance: Relance::Simple { periode: 2.0 },
    },
    // Brume glaciale (4420) : la brume (essai du 08/11/2026), reposée toutes les 2 s.
    Style {
        texte: 69000540,
        nom: "Brume glaciale",
        visuels: &[523887],
        relance: Relance::Simple { periode: 2.0 },
    },
];

/// Essais en cours, proposés dans « Course (essais) » (onglet caché quand la liste est
/// vide). Méthode : chaque sort mesuré en entier, puis visuel par visuel, sans relance ; le
/// joueur garde ce qui lui plaît, qui passe dans STYLES, et la liste est vidée.
pub const ESSAIS: [Style; 0] = [];

/// Nombre de styles (STYLES puis ESSAIS).
pub const NOMBRE: usize = STYLES.len() + ESSAIS.len();

/// Nom du style de rang `rang` (pour la sauvegarde).
pub fn nom_du_style(rang: usize) -> &'static str {
    style(rang).nom
}

/// Le style de rang `rang` : d'abord les STYLES, puis les ESSAIS.
fn style(rang: usize) -> &'static Style {
    if rang < STYLES.len() { &STYLES[rang] } else { &ESSAIS[rang - STYLES.len()] }
}

impl Style {
    fn nombre_de_groupes(&self) -> usize {
        if matches!(self.relance, Relance::Alternee { .. }) { 2 } else { 1 }
    }

    /// Intervalle entre deux relances (None : pas de relance).
    fn periode(&self) -> Option<f32> {
        match self.relance {
            Relance::Aucune => None,
            Relance::Simple { periode } | Relance::Alternee { periode } => Some(periode),
        }
    }

    /// Lignes d'effet du groupe `groupe`.
    fn lignes(&self, groupe: usize) -> impl Iterator<Item = i32> + '_ {
        let n = self.visuels.len();
        LIGNES[groupe * n..(groupe + 1) * n].iter().map(|&(ligne, _)| ligne as i32)
    }
}

pub struct Course {
    /// Style choisi (rang dans STYLES puis ESSAIS, voir `style`).
    choisi: Option<usize>,
    /// Style actuellement posé sur le personnage.
    pose: Option<usize>,
    /// Groupes de lignes actuellement posés.
    groupes_poses: [bool; 2],
    /// Relance : groupe à reposer à l'image suivante, groupe à relancer ensuite, et quand.
    a_reposer: Option<usize>,
    groupe_suivant: usize,
    prochaine_relance: Instant,
    /// Début de la demande de sprint en cours.
    sprint_depuis: Option<Instant>,
}

impl Course {
    /// Choix actuel (lu par reglages.rs pour la sauvegarde).
    pub fn choisi(&self) -> Option<usize> {
        self.choisi
    }

    pub fn new() -> Self {
        Self {
            choisi: None,
            pose: None,
            groupes_poses: [false; 2],
            a_reposer: None,
            groupe_suivant: 0,
            prochaine_relance: Instant::now(),
            sprint_depuis: None,
        }
    }

    pub fn choisir(&mut self, rang: Option<usize>) {
        self.choisi = rang;
        match rang {
            Some(r) => noter(&format!("Traînée de course : {}.", style(r).nom)),
            None => noter("Traînée de course : aucune."),
        }
    }

    /// Remplit les lignes partagées pour ce style : copies de l'effet neutre, permanentes,
    /// sans catégorie, chacune avec un visuel continu (armes visibles).
    fn preparer(style: &Style) -> bool {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(neutre) = parametres.get::<SpEffectParam>(EFFET_NEUTRE).cloned() else {
            noter(&format!("Course : effet neutre {EFFET_NEUTRE} introuvable."));
            return false;
        };
        let nombre = style.visuels.len() * style.nombre_de_groupes();
        let visuels = style.visuels.iter().cycle().take(nombre);
        for (&(ligne, visuel), &sfx) in LIGNES.iter().zip(visuels) {
            let Some(v) = parametres.get_mut::<SpEffectVfxParam>(visuel) else {
                noter(&format!("Course : ligne de visuel {visuel} introuvable."));
                return false;
            };
            remplir_visuel(v, sfx, POINT, -1, false);
            let Some(l) = parametres.get_mut::<SpEffectParam>(ligne) else {
                noter(&format!("Course : ligne d'effet {ligne} introuvable."));
                return false;
            };
            *l = neutre.clone();
            l.set_effect_endurance(-1.0);
            l.set_sp_category(0);
            l.set_vfx_id(visuel as i32);
        }
        true
    }

    /// Prochaine relance : dans `periode` secondes de temps de jeu.
    fn dans(periode: f32) -> Instant {
        let vitesse = unsafe { CSFlipper::instance() }.map(|f| f.game_speed).unwrap_or(1.0);
        Instant::now() + Duration::from_secs_f32(periode / vitesse.max(0.1))
    }

    /// À appeler à chaque image, AVANT la mise à jour des roulades (pour que la traînée
    /// soit éteinte quand la roulade réécrit les lignes).
    pub fn mettre_a_jour(&mut self, roulade: &mut Roulade) {
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };

        let ta = &joueur.chr_ins.modules.time_act;
        let animation = ta.anim_queue[(ta.read_idx as usize) % ta.anim_queue.len()].anim_id;
        let en_roulade = roulade.en_cours()
            || ANIMATIONS_ROULADE.contains(&animation)
            || joueur
                .chr_ins
                .special_effect
                .entries()
                .any(|e| e.param_id == EFFET_ROULADE_DU_JEU);

        let demandes = &joueur.chr_ins.modules.action_request.movement_request_flags;
        if demandes.dash() && demandes.raw_input() && !en_roulade {
            self.sprint_depuis.get_or_insert_with(Instant::now);
        } else {
            self.sprint_depuis = None;
        }
        let sprinte = self.sprint_depuis.is_some_and(|t| t.elapsed() >= DELAI_AVANT_TRAINEE);
        let voulu = if sprinte { self.choisi } else { None };

        // Disparue (mort, chargement) : on la reposera.
        if let Some(rang) = self.pose {
            let style = style(rang);
            let mut presente = false;
            for g in 0..2 {
                if self.groupes_poses[g] {
                    for ligne in style.lignes(g) {
                        if joueur.chr_ins.special_effect.entries().any(|e| e.param_id == ligne) {
                            presente = true;
                        }
                    }
                }
            }
            if !presente && self.a_reposer.is_none() {
                self.pose = None;
                self.groupes_poses = [false; 2];
            }
        }

        if self.pose != voulu {
            // Éteindre l'ancienne traînée (fin du sprint, roulade, ou autre choix).
            if let Some(rang) = self.pose.take() {
                for g in 0..2 {
                    if self.groupes_poses[g] {
                        for ligne in style(rang).lignes(g) {
                            joueur.chr_ins.remove_speffect(ligne);
                        }
                    }
                }
                self.groupes_poses = [false; 2];
                self.a_reposer = None;
            }
            // Allumer la nouvelle (groupe 0).
            if let Some(rang) = voulu {
                let style = style(rang);
                if !Self::preparer(style) {
                    return;
                }
                roulade.oublier_preparation();
                for ligne in style.lignes(0) {
                    joueur.apply_speffect(ligne, true);
                }
                self.groupes_poses[0] = true;
                self.groupe_suivant = 1 % style.nombre_de_groupes();
                self.prochaine_relance = Self::dans(style.periode().unwrap_or(PERIODE_RELANCE));
                self.pose = Some(rang);
            }
            return;
        }

        // Relance : enlever un groupe, le remettre à l'image suivante.
        let Some(rang) = self.pose else {
            return;
        };
        let style = style(rang);
        let Some(periode) = style.periode() else {
            return;
        };
        if let Some(g) = self.a_reposer.take() {
            for ligne in style.lignes(g) {
                joueur.apply_speffect(ligne, true);
            }
            self.groupes_poses[g] = true;
        } else if Instant::now() >= self.prochaine_relance {
            let g = self.groupe_suivant;
            if self.groupes_poses[g] {
                for ligne in style.lignes(g) {
                    joueur.chr_ins.remove_speffect(ligne);
                }
                self.groupes_poses[g] = false;
            }
            self.a_reposer = Some(g);
            self.groupe_suivant = (g + 1) % style.nombre_de_groupes();
            self.prochaine_relance = Self::dans(periode);
        }
    }
}
