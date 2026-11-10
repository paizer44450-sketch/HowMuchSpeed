//! Arme fantôme : un visuel d'arme de sort tenu en main pendant chaque attaque de l'arme
//! (ex. le marteau du Marteau de Haima, 4120). Purement visuel : aucun dégât.
//!
//! Fichier d'animation du sort (a478.tae, exporté le 08/11/2026) : le marteau est le visuel
//! 523772 accroché au point 20 (main droite ; 21 pour la main gauche), sa traînée de coup
//! le visuel 523773 au même point. Le mouvement vient de l'animation du personnage : posé
//! sur la main pendant une attaque de l'arme, le marteau suit le geste de l'arme.
//!
//! Attaque de l'arme : animation dont le groupe (millions) est sous 400 (les sorts sont
//! dans les groupes 400 et plus, les cendres de guerre plus haut encore) et dont le numéro
//! dans le groupe est entre 30000 et 39999 (attaques à une et deux mains, en course, en
//! roulade, sautées). L'arme du joueur est cachée pendant le coup.
//!
//! Main gauche (0.65.0) : même chose au point 21 ; pour l'instant sur toutes les attaques
//! (les numéros d'attaque sont notés au journal pour repérer celles de la main gauche).
//!
//! Lignes : effet 1150 (« QuickMatch - Crown for Tied 1st Place », couronne des combats du
//! Colisée en ligne, jamais utilisée hors ligne), recopié de l'effet neutre 18553 ; visuels
//! 50961 et 50962 (lignes libres). Une nouvelle attaque pendant qu'une autre est en cours :
//! l'effet est enlevé puis reposé à l'image suivante, pour rejouer le visuel.

use eldenring::cs::{ChrInsExt, SoloParamRepository, SpEffectParam, SpEffectVfxParam, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;
use crate::roulade::remplir_visuel;

const EFFET_NEUTRE: u32 = 18553;
/// Lignes d'effet (une par visuel) et de visuel, par main : effets du Colisée en ligne
/// (QuickMatch, jamais utilisés hors ligne), visuels libres.
const LIGNES_DROITE: [(u32, u32); 2] = [(1150, 50961), (1140, 50962)];
const LIGNES_GAUCHE: [(u32, u32); 2] = [(1130, 50973), (1120, 50974)];

/// Main qui tient l'arme fantôme.
#[derive(Clone, Copy, PartialEq)]
pub enum Main {
    /// Main droite (point 20 d'après a478.tae, ou le point propre au visuel).
    Droite,
    /// Main gauche (point 21, ou le point propre au visuel).
    Gauche,
}

impl Main {
    fn lignes(self) -> &'static [(u32, u32); 2] {
        match self {
            Main::Droite => &LIGNES_DROITE,
            Main::Gauche => &LIGNES_GAUCHE,
        }
    }
    fn point(self, visuel: &Visuel) -> i16 {
        match self {
            Main::Droite => visuel.point_droite,
            Main::Gauche => visuel.point_gauche,
        }
    }
    fn nom(self) -> &'static str {
        match self {
            Main::Droite => "main droite",
            Main::Gauche => "main gauche",
        }
    }
}

/// Un visuel d'arme fantôme : numéro du FFX et points d'accroche (main droite, main gauche).
#[derive(Clone, Copy)]
pub struct Visuel {
    sfx: i32,
    point_droite: i16,
    point_gauche: i16,
}

/// Visuel accroché aux points 20 / 21 (mains), comme le Marteau de Haima.
const fn en_main(sfx: i32) -> Visuel {
    Visuel { sfx, point_droite: 20, point_gauche: 21 }
}

/// Une arme fantôme : texte du menu, nom, visuels (au plus 2).
pub struct Fantome {
    pub texte: i32,
    pub nom: &'static str,
    visuels: &'static [Visuel],
    /// Essai du premier visuel (vide = pas d'essai) : à chaque coup, il passe à l'élément
    /// suivant de la liste (point main droite, point main gauche, visuel ; visuel 0 = le
    /// même). Le joueur compte les coups pour dire lequel va.
    essai_points: &'static [(i16, i16, i32)],
}

pub const FANTOMES: [Fantome; 14] = [
    // Marteau de Haima (4120) : le marteau et sa traînée.
    Fantome { texte: 69000542, nom: "Marteau de Haima", essai_points: &[], visuels: &[en_main(523772), en_main(523773)] },
    // Espadon de Caria (4430) : même construction que le marteau (MagicParam effectSfxId
    // 523742 = la lame, comme 523772 pour le marteau) ; traînée 523743 supposée par analogie.
    Fantome { texte: 69000544, nom: "Espadon de Caria", essai_points: &[], visuels: &[en_main(523742), en_main(523743)] },
    // Lame lunaire d'Adula (4431) et Trancheuse de Caria (4440) : leur « effet du sort »
    // (MagicParam effectSfxId), traînée supposée au numéro suivant. L'Empaleuse de Caria
    // (4450) a la même lame que l'Espadon (523742).
    Fantome { texte: 69000545, nom: "Lame lunaire d'Adula", essai_points: &[], visuels: &[en_main(523892), en_main(523893)] },
    Fantome { texte: 69000546, nom: "Trancheuse de Caria", essai_points: &[], visuels: &[en_main(523752), en_main(523753)] },
    // Trancheuse de Caria en revers (0.66.8) : aux points 20 / 21 (haut vers +Z dans la pose
    // de référence), la lame sort vers l'avant ; elle suit donc l'axe « bas » du point. Pour
    // la retourner : points de l'os de l'arme dont le haut va vers -Z, 180 à droite (même
    // « avant » que 20) et 426 à gauche (pour Ansbach, l'avant n'a rien changé au rendu).
    Fantome {
        texte: 69000550,
        nom: "Trancheuse de Caria (revers)",
        essai_points: &[],
        visuels: &[
            Visuel { sfx: 523752, point_droite: 180, point_gauche: 426 },
            Visuel { sfx: 523753, point_droite: 180, point_gauche: 426 },
        ],
    },
    // Armes de lumière de Margit et Morgott (même modèle c2130 ; c2130.tae et c2130.flver
    // exportés le 09/11/2026). Chez lui, chaque arme est accrochée à un os d'arme de la main
    // gauche, le long de l'axe -Z de la pose de référence (comme l'arme du joueur, vers -Z) ;
    // le visuel est à SA taille (il est environ 2,5 fois plus grand que le joueur), et son
    // point est à une certaine distance de la main : dague / épée 0,4 m, marteau 2,2 m,
    // bâton 2,4 m, lance 6,3 m. On essaie sur le joueur des points de l'os de l'arme vers -Z
    // à une distance proche (un point par coup), à valider en jeu (0.67.0).
    // Ces visuels ne sont chargés qu'avec Margit / Morgott (paquet sfxbnd_c2130) : la 0.67.1
    // les recopie (fichiers .fxr et .ffxreslist) dans le paquet commun
    // sfx\sfxbnd_commoneffects_dlc02.ffxbnd.dcx du dossier du mod, toujours chargé.
    // L'autre épée (620380, points 25) n'est pas dans le paquet de Margit : retirée.
    // Dague : 621300 au point 340 (os L_Dagger, 0,4 m de la main).
    Fantome {
        texte: 69000551,
        nom: "Dague de lumière de Margit",
        // Points 120 / 102 (0.67.2). Tournée d'un quart de tour dans le fichier d'effet
        // (0.67.4, autour de l'axe Z, -90) : copie 699301, validée par le joueur (0.67.5).
        essai_points: &[],
        visuels: &[Visuel { sfx: 699301, point_droite: 120, point_gauche: 102 }],
    },
    // Épée : 621301 au point 300 (os L_Sword, 0,4 m).
    Fantome {
        texte: 69000552,
        nom: "Épée de lumière de Margit",
        // Validé (0.67.2) : points 120 / 102.
        essai_points: &[],
        visuels: &[Visuel { sfx: 621301, point_droite: 120, point_gauche: 102 }],
    },
    // Lance : 621302 au point 310 (os L_Spear, 6,3 m).
    Fantome {
        texte: 69000553,
        nom: "Lance de lumière de Margit",
        // Validé (0.67.2) : points 196 / 197.
        essai_points: &[],
        visuels: &[Visuel { sfx: 621302, point_droite: 196, point_gauche: 197 }],
    },
    // Lance, autre visuel (animation 3025) : 621304 au point 310.
    Fantome {
        texte: 69000554,
        nom: "Lance de lumière de Margit (2)",
        // Comme la première lance (0.67.2) : points 196 / 197.
        essai_points: &[],
        visuels: &[Visuel { sfx: 621304, point_droite: 196, point_gauche: 197 }],
    },
    // Marteau : 621303 au point 330 (os L_Hammer, 2,2 m, décalé de côté).
    Fantome {
        texte: 69000555,
        nom: "Marteau de lumière de Margit",
        // Points 856 / 174 (0.67.2 ; le point 174 existe sur les deux os d'arme : à droite,
        // il faisait apparaître un second marteau dans la main gauche). Chez Margit, le point
        // du marteau est à 0,7 m de l'axe de la main ; aucun point du joueur n'a ce décalage.
        // Copie décalée dans le fichier d'effet (+0,8 m en X, -0,1 m en Z) : 699344, choisie
        // par le joueur après les essais 0.67.4 à 0.67.6 (0.67.7).
        essai_points: &[],
        visuels: &[Visuel { sfx: 699344, point_droite: 856, point_gauche: 174 }],
    },
    // Lame du bâton (main droite chez lui) : 621380 au point 16 (os R_Wand, 2,4 m).
    Fantome {
        texte: 69000556,
        nom: "Lame du bâton de Margit",
        // Validé (0.67.3) : points 856 / 174.
        essai_points: &[],
        visuels: &[Visuel { sfx: 621380, point_droite: 856, point_gauche: 174 }],
    },
    // Lame d'ébène (7530) : aucun visuel dans MagicParam ; d'après a516.tae, la lame 525420
    // (points 20 / 21) et sa traînée 525421.
    Fantome { texte: 69000547, nom: "Lame d'ébène", essai_points: &[], visuels: &[en_main(525420), en_main(525421)] },
    // Lame furieuse d'Ansbach (2006300) : son effectSfxId est celui de la Trancheuse, mais
    // d'après a540.tae la vraie lame est 542200, posée aux points 851 (droite) / 852
    // (gauche) pendant tout le sort, et sa traînée de coup 542201 aux points 849 / 850.
    // Aux points du sort, la lame est tenue en revers : parfaite avec les armes en prise
    // inversée (lames revers), validé par le joueur le 08/11/2026.
    Fantome {
        texte: 69000548,
        nom: "Lame furieuse d'Ansbach",
        essai_points: &[],
        visuels: &[
            Visuel { sfx: 542200, point_droite: 851, point_gauche: 852 },
            Visuel { sfx: 542201, point_droite: 849, point_gauche: 850 },
        ],
    },
    // Version « lame droite » : le visuel ne peut pas être tourné (aucun réglage de rotation
    // dans SpEffectVfx), seul le point change. Points des mains 20 / 21 (0.66.4) : la lame
    // a pivoté sur son axe (courbure inversée) mais restait du mauvais côté de la main.
    // Points de l'arme (0.66.5) : rien ne s'affiche, l'arme étant cachée.
    // 0.66.6, d'après les points du modèle du joueur (c0000.flver, pose de référence) : la
    // lame suit l'axe « haut » du point. Au point 851 (revers), ce haut va vers l'arrière
    // (+Z), alors que les points de coup de l'arme (100, 110) sont devant (-Z). Points de
    // l'os de l'arme dont le haut va vers -Z : 425 (même « avant » que 851) et 180 (avant
    // inversé, l'autre courbure) à droite ; 426 à gauche (même « avant » que 852).
    // Traînée : 849 / 850 sont à 0,76 m derrière la main ; à la place, 100 / 102, devant.
    // Validé par le joueur (0.66.6) ; 425 et 180 donnent le même rendu : 425 gardé (0.66.7).
    Fantome {
        texte: 69000549,
        nom: "Lame furieuse d'Ansbach (lame droite)",
        essai_points: &[],
        visuels: &[
            Visuel { sfx: 542200, point_droite: 425, point_gauche: 426 },
            Visuel { sfx: 542201, point_droite: 100, point_gauche: 102 },
        ],
    },
];

pub struct ArmeFantome {
    main: Main,
    /// Lignes de journal restantes : numéros des attaques (pour repérer celles de la main
    /// gauche, 08/11/2026).
    notes_restantes: u32,
    choisi: Option<usize>,
    pret: bool,
    /// Animation d'attaque en cours (celle pour laquelle l'effet est posé).
    attaque: Option<i32>,
    a_reposer: bool,
    /// Rang du point d'essai en cours (pour `essai_points`).
    essai: usize,
}

/// Vrai si l'animation est une attaque de l'arme.
pub fn est_attaque(animation: i32) -> bool {
    let groupe = animation / 1_000_000;
    let numero = animation % 1_000_000;
    (0..400).contains(&groupe) && (30000..40000).contains(&numero)
}

impl ArmeFantome {
    /// Choix actuel (lu par reglages.rs pour la sauvegarde).
    pub fn choisi(&self) -> Option<usize> {
        self.choisi
    }

    pub fn new(main: Main) -> Self {
        Self { main, notes_restantes: 0, choisi: None, pret: false, attaque: None, a_reposer: false, essai: 0 }
    }

    pub fn choisir(&mut self, rang: Option<usize>) {
        self.enlever();
        self.choisi = rang.filter(|&r| r < FANTOMES.len());
        self.pret = false;
        self.essai = 0;
        self.notes_restantes = 30;
        match self.choisi {
            Some(r) => noter(&format!("Arme fantôme ({}) : « {} ».", self.main.nom(), FANTOMES[r].nom)),
            None => noter(&format!("Arme fantôme ({}) : aucune.", self.main.nom())),
        }
    }

    fn preparer(f: &Fantome, main: Main) -> bool {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(neutre) = parametres.get::<SpEffectParam>(EFFET_NEUTRE).cloned() else {
            noter(&format!("Arme fantôme : effet neutre {EFFET_NEUTRE} introuvable."));
            return false;
        };
        for (fx, &(ligne, visuel)) in f.visuels.iter().zip(main.lignes().iter()) {
            let Some(v) = parametres.get_mut::<SpEffectVfxParam>(visuel) else {
                noter(&format!("Arme fantôme : ligne de visuel {visuel} introuvable."));
                return false;
            };
            // Arme du joueur cachée pendant le coup : seul le fantôme se voit.
            remplir_visuel(v, fx.sfx, main.point(fx), -1, true);
            let Some(l) = parametres.get_mut::<SpEffectParam>(ligne) else {
                noter(&format!("Arme fantôme : ligne d'effet {ligne} introuvable."));
                return false;
            };
            *l = neutre.clone();
            l.set_effect_endurance(-1.0);
            l.set_sp_category(0);
            l.set_vfx_id(visuel as i32);
        }
        noter(&format!("Arme fantôme ({}) : « {} » prête.", main.nom(), f.nom));
        true
    }

    /// Essai de points : passe le premier visuel au couple de points suivant (les effets ne
    /// sont pas posés à ce moment-là) et le note au journal avec le numéro du coup.
    fn point_d_essai_suivant(&mut self, f: &Fantome) {
        if f.essai_points.is_empty() {
            return;
        }
        let rang = self.essai % f.essai_points.len();
        let (droite, gauche, sfx) = f.essai_points[rang];
        let point = match self.main {
            Main::Droite => droite,
            Main::Gauche => gauche,
        };
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return;
        };
        let visuel = self.main.lignes()[0].1;
        if let Some(v) = parametres.get_mut::<SpEffectVfxParam>(visuel) {
            v.set_init_dmy_id(point);
            v.set_midst_dmy_id(point);
            if sfx > 0 {
                v.set_midst_sfx_id(sfx);
            }
        }
        noter(&format!(
            "Arme fantôme ({}) : coup {} -> point {point}, visuel {sfx}.",
            self.main.nom(),
            rang + 1
        ));
        self.essai += 1;
    }

    /// Enlève les effets du joueur.
    fn enlever(&mut self) {
        self.attaque = None;
        self.a_reposer = false;
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            return;
        };
        for &(ligne, _) in self.main.lignes().iter() {
            joueur.chr_ins.remove_speffect(ligne as i32);
        }
    }

    /// À appeler à chaque image.
    pub fn mettre_a_jour(&mut self) {
        let Some(rang) = self.choisi else {
            return;
        };
        let f = &FANTOMES[rang];
        if !self.pret {
            self.pret = Self::preparer(f, self.main);
            if !self.pret {
                return;
            }
        }
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            self.attaque = None;
            return;
        };
        let lignes = self.main.lignes();
        let nombre = f.visuels.len().min(lignes.len());
        if self.a_reposer {
            self.a_reposer = false;
            for &(ligne, _) in lignes.iter().take(nombre) {
                joueur.chr_ins.apply_speffect(ligne as i32, true);
            }
            return;
        }
        let ta = &joueur.chr_ins.modules.time_act;
        let animation = ta.anim_queue[(ta.read_idx as usize) % ta.anim_queue.len()].anim_id;
        if est_attaque(animation) {
            if self.attaque != Some(animation) {
                if self.notes_restantes > 0 {
                    self.notes_restantes -= 1;
                    noter(&format!("Arme fantôme ({}) : attaque {animation}.", self.main.nom()));
                }
                if self.attaque.is_some() {
                    // Attaque suivante d'un enchaînement : on rejoue le visuel.
                    for &(ligne, _) in lignes.iter().take(nombre) {
                        joueur.chr_ins.remove_speffect(ligne as i32);
                    }
                    self.point_d_essai_suivant(f);
                    self.a_reposer = true;
                } else {
                    self.point_d_essai_suivant(f);
                    for &(ligne, _) in lignes.iter().take(nombre) {
                        joueur.chr_ins.apply_speffect(ligne as i32, true);
                    }
                }
                self.attaque = Some(animation);
            }
        } else if self.attaque.is_some() {
            self.attaque = None;
            for &(ligne, _) in lignes.iter().take(nombre) {
                joueur.chr_ins.remove_speffect(ligne as i32);
            }
        }
    }
}
