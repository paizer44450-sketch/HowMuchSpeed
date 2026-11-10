//! Impact : un visuel de sort joué sur un ennemi chaque fois que le joueur le touche.
//! Purement visuel : aucun dégât, aucune statistique.
//!
//! 0.64.2 : l'impact se joue sur la lame du joueur (point 110 de l'arme de droite) au moment
//! du coup : la lame est dans l'ennemi à cet instant. L'étincelle d'origine du jeu naît au
//! point de contact exact, qui n'est lisible nulle part (mesure F11 : aucun effet).
//!
//! Détection d'un coup : à chaque image, on note les PV de chaque personnage chargé. Quand
//! les PV d'un personnage à moins de DISTANCE_MAX baissent pendant une attaque de l'arme du
//! joueur (ou moins de APRES_ATTAQUE après), c'est un coup du joueur : on pose sur ce
//! personnage un effet court dont le visuel est l'impact choisi. (Premier essai : le
//! « dernier agresseur » du personnage, `ChrIns::last_hit_by`, reste vide en jeu : journal du
//! 08/11/2026.) Limite : le visuel part du centre du corps de l'ennemi (point 220), pas de
//! l'endroit exact du coup.
//!
//! Lignes (voir `Main`) : recopiées de l'effet neutre 18553. Main droite : effet 102006
//! (« Deflecting Hardtear - Unused »), visuel 50959. Main gauche (0.68.0) : effet 1160
//! (Colisée en ligne), visuel 50975. Lignes de visuel libres : absentes des effets et
//! présentes une seule fois dans regulation.bin. Les deux impacts se déclenchent à chaque
//! coup détecté (on ne sait pas encore quelle main frappe).
//! Un coup qui retombe pendant que l'effet est encore là : l'effet est enlevé puis reposé à
//! l'image suivante (enlever et remettre dans la même image ne rejoue pas le visuel).

use eldenring::cs::{ChrIns, ChrInsExt, FieldInsHandle, SoloParamRepository, SpEffectParam, SpEffectVfxParam, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;
use crate::roulade::remplir_visuel;

/// Effet de Radagon sans aucune statistique : modèle de la ligne.
const EFFET_NEUTRE: u32 = 18553;
/// Côté de l'impact : lignes et point d'accroche.
///
/// Point : avant (0.64.1), point 110 de l'ARME de droite (type d'effet 1), invisible dès que
/// l'arme est cachée (arme fantôme active : journal du 09/11/2026, coups bien détectés mais
/// rien à l'écran). Depuis 0.67.8 : un point du PERSONNAGE (type 0) sur l'os de l'arme,
/// devant la main dans le sens de la lame (c0000.flver), toujours visible (validé).
/// Droite : point 100 (0,74 m devant la main). Gauche (0.68.0) : point 102 (1,1 m, le plus
/// proche dans le sens de la lame sur l'os de l'arme gauche).
#[derive(Clone, Copy, PartialEq)]
pub enum Main {
    /// Effet 102006 (« Deflecting Hardtear - Unused »), visuel 50959, point 100.
    Droite,
    /// Effet 1160 (« QuickMatch », Colisée en ligne, jamais utilisé hors ligne),
    /// visuel 50975 (ligne libre), point 102.
    Gauche,
}

impl Main {
    fn ligne_effet(self) -> u32 {
        match self {
            Main::Droite => 102006,
            Main::Gauche => 1160,
        }
    }
    fn ligne_visuel(self) -> u32 {
        match self {
            Main::Droite => 50959,
            Main::Gauche => 50975,
        }
    }
    fn point(self) -> i16 {
        match self {
            Main::Droite => 100,
            Main::Gauche => 102,
        }
    }
    fn nom(self) -> &'static str {
        match self {
            Main::Droite => "main droite",
            Main::Gauche => "main gauche",
        }
    }
}
/// Durée de l'effet posé (s) : le temps que l'impact se joue.
const DUREE: f32 = 0.8;
/// Distance maximale entre le joueur et le personnage touché (m).
const DISTANCE_MAX: f32 = 12.0;
/// Temps pendant lequel une perte de PV compte encore après la fin d'une attaque.
const APRES_ATTAQUE: std::time::Duration = std::time::Duration::from_millis(1000);

/// Les impacts proposés : (texte du menu, visuel du jeu, nom).
pub const IMPACTS: [(i32, i32, &str); 21] = [
    // Barrage de cristal (4060) : impact de ses cristaux.
    (69000527, 523723, "Barrage de cristal"),
    // Étoiles de la perdition (4220) : les deux impacts de ses étoiles.
    (69000528, 523903, "Étoiles de la perdition"),
    (69000529, 523908, "Étoiles de la perdition (variante)"),
    // Faveur sanguine (6310) : impact du projectile (sfxId_Hit de la balle 10631000) et
    // éclatement qui suit (balle 10631002, par HitBulletID).
    (69000559, 524133, "Faveur sanguine"),
    (69000560, 524134, "Faveur sanguine (éclatement)"),
    // Disque de Lumière (6700) : impact du disque (sfxId_Hit des balles 10670000 à 10670003).
    (69000561, 528123, "Disque de Lumière"),
    // Fronde bestiale (6800) : impact des pierres (sfxId_Hit des balles 10680000 / 10680001).
    (69000564, 529523, "Fronde bestiale"),
    // Lance de foudre (6900) : impact de la lance (sfxId_Hit de la balle 10690000). Retirés
    // car en double avec d'autres impacts (0.69.6) : Lance de foudre (variante) 525508,
    // Triple Halos de lumière 528113, Étoiles d'Elden 525404.
    (69000565, 525503, "Lance de foudre"),
    // Éclair des dragons anciens (6910) : les balles du sort n'ont pas de visuel ; l'impact
    // est sur les balles voisines du même bloc (sfxId_Hit 525533 : 10691015 / 10691016).
    (69000567, 525533, "Éclair des dragons anciens"),
    // Lance de foudre glacée (6921) : pas de sfxId_Hit ; l'impact est le visuel de la balle
    // créée au contact (HitBulletID 10692101 : sfxId_Bullet 525593).
    (69000568, 525593, "Lance de foudre glacée"),
    // Frénésie intolérable (7311) : petits projectiles = balles 10731100 et suivantes
    // (527012 au départ, 527015 en vol, puis 527013 à la fin : leur impact).
    (69000569, 527013, "Frénésie intolérable"),
    // Explosion exaltée (7320) : sfxId_Hit de la balle 10732000, et celui de la seconde
    // version du sort (refId2, balle 10732050).
    (69000570, 527033, "Explosion exaltée"),
    (69000571, 527038, "Explosion exaltée (variante)"),
    // Halos de lumière spectrale (2005000, DLC) : sfxId_Hit des balles 210500004 / 210500005,
    // et celui de la seconde version du sort (balles 210500054 / 210500055).
    (69000572, 553003, "Halos de lumière spectrale"),
    (69000573, 553008, "Halos de lumière spectrale (variante)"),
    // Royaume des ombres (2006680, DLC) : sfxId_Hit de la balle 210668009.
    (69000574, 542115, "Royaume des ombres"),
    // Papillons putrescents (2007200, DLC) : pas de sfxId_Hit. Chaîne des balles : papillons
    // en vol (541002, 3 s), puis 541003 (1 s), puis 541004 (0,3 s, le plus petit) : les deux
    // derniers proposés comme petit impact.
    (69000575, 541004, "Papillons putrescents"),
    (69000576, 541003, "Papillons putrescents (variante)"),
    // Plumes d'oiseau divin (2007740, DLC) : sfxId_Hit de la balle 210774000.
    (69000577, 541353, "Plumes d'oiseau divin"),
    // Pluie de feu (2007810, DLC) : pas de sfxId_Hit. Les gouttes (542023, balle 210781006)
    // finissent par 542024 (0,3 s, balle 210781007) : leur impact. Variante : 542022
    // (balle 210781005, 1,5 s).
    (69000578, 542024, "Pluie de feu"),
    (69000579, 542022, "Pluie de feu (variante)"),
];

pub struct Impact {
    main: Main,
    choisi: Option<usize>,
    pret: bool,
    /// PV de chaque personnage à l'image précédente.
    pv: Vec<(FieldInsHandle, i32)>,
    /// Effet enlevé du joueur, à reposer à l'image suivante.
    a_reposer: bool,
    /// Lignes de journal restantes pour comprendre les coups (diagnostic, 08/11/2026).
    notes_restantes: u32,
    /// Dernier instant où le joueur était dans une attaque de l'arme.
    derniere_attaque: Option<std::time::Instant>,
}

impl Impact {
    /// Choix actuel (lu par reglages.rs pour la sauvegarde).
    pub fn choisi(&self) -> Option<usize> {
        self.choisi
    }

    pub fn new(main: Main) -> Self {
        Self { main, choisi: None, pret: false, pv: Vec::new(), a_reposer: false, notes_restantes: 0, derniere_attaque: None }
    }

    pub fn choisir(&mut self, rang: Option<usize>) {
        self.choisi = rang.filter(|&r| r < IMPACTS.len());
        self.pret = false;
        self.a_reposer = false;
        // L'effet en cours est enlevé tout de suite (aussi quand on choisit « aucun »).
        if let Ok(monde) = unsafe { WorldChrMan::instance_mut() }
            && let Some(ref mut joueur) = monde.main_player
        {
            joueur.chr_ins.remove_speffect(self.main.ligne_effet() as i32);
        }
        self.notes_restantes = 20;
        match self.choisi {
            Some(r) => noter(&format!("Impact ({}) : « {} ».", self.main.nom(), IMPACTS[r].2)),
            None => noter(&format!("Impact ({}) : aucun.", self.main.nom())),
        }
    }

    /// Prépare les lignes d'effet et de visuel pour l'impact choisi.
    fn preparer(main: Main, sfx: i32) -> bool {
        let (ligne_effet, ligne_visuel) = (main.ligne_effet(), main.ligne_visuel());
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return false;
        };
        let Some(neutre) = parametres.get::<SpEffectParam>(EFFET_NEUTRE).cloned() else {
            noter(&format!("Impact : effet neutre {EFFET_NEUTRE} introuvable."));
            return false;
        };
        let Some(v) = parametres.get_mut::<SpEffectVfxParam>(ligne_visuel) else {
            noter(&format!("Impact : ligne de visuel {ligne_visuel} introuvable."));
            return false;
        };
        // Type d'effet 0 (laissé par remplir_visuel) : point du personnage.
        remplir_visuel(v, sfx, main.point(), -1, false);
        // Visuel de DÉBUT (joué une fois, il s'éteint de lui-même) et non visuel continu :
        // un impact de sort est fait pour se jouer une fois. En visuel continu, celui
        // d'Explosion exaltée restait à l'écran pour de bon (0.69.5, rapport du joueur).
        v.set_init_sfx_id(sfx);
        v.set_midst_sfx_id(-1);
        let Some(ligne) = parametres.get_mut::<SpEffectParam>(ligne_effet) else {
            noter(&format!("Impact : ligne d'effet {ligne_effet} introuvable."));
            return false;
        };
        *ligne = neutre;
        ligne.set_effect_endurance(DUREE);
        ligne.set_sp_category(0);
        ligne.set_vfx_id(ligne_visuel as i32);
        // L'effet neutre vient d'un PNJ : on autorise toutes les cibles, ennemis compris.
        ligne.set_effect_target_self(true);
        ligne.set_effect_target_friend(true);
        ligne.set_effect_target_enemy(true);
        ligne.set_effect_target_player(true);
        ligne.set_effect_target_ai(true);
        ligne.set_effect_target_live(true);
        ligne.set_effect_target_ghost(true);
        ligne.set_effect_target_attacker(true);
        ligne.set_effect_target_self_target(true);
        ligne.set_effect_target_friendly_target(true);
        ligne.set_effect_target_oppose_target(true);
        noter(&format!("Impact ({}) : visuel {sfx} prêt.", main.nom()));
        true
    }

    /// À appeler à chaque image.
    pub fn mettre_a_jour(&mut self) {
        let Some(rang) = self.choisi else {
            self.pv.clear();
            self.a_reposer = false;
            return;
        };
        if !self.pret {
            // Changement d'impact : on enlève d'abord l'effet en cours du joueur, avant de
            // réécrire sa ligne de visuel (sinon l'ancien visuel peut rester à l'écran).
            if let Ok(monde) = unsafe { WorldChrMan::instance_mut() }
                && let Some(ref mut joueur) = monde.main_player
            {
                joueur.chr_ins.remove_speffect(self.main.ligne_effet() as i32);
            }
            self.a_reposer = false;
            self.pret = Self::preparer(self.main, IMPACTS[rang].1);
            if !self.pret {
                return;
            }
        }
        let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
            return;
        };
        let Some(ref mut joueur) = monde.main_player else {
            self.pv.clear();
            return;
        };
        // Effet enlevé à l'image précédente (coup pendant que l'impact se jouait encore) :
        // on le repose maintenant pour rejouer le visuel.
        if self.a_reposer {
            self.a_reposer = false;
            joueur.chr_ins.apply_speffect(self.main.ligne_effet() as i32, true);
        }
        let main_joueur = joueur.chr_ins.field_ins_handle;
        let ta = &joueur.chr_ins.modules.time_act;
        let animation = ta.anim_queue[(ta.read_idx as usize) % ta.anim_queue.len()].anim_id;
        if crate::arme_fantome::est_attaque(animation) {
            self.derniere_attaque = Some(std::time::Instant::now());
        }
        let en_attaque = self.derniere_attaque.is_some_and(|t| t.elapsed() <= APRES_ATTAQUE);
        let pj = joueur.chr_ins.modules.physics.position;
        let mut touche = false;
        let mut pv_maintenant = Vec::with_capacity(self.pv.len());
        for liste in monde.chr_sets.iter().flatten() {
            for personnage in liste.characters() {
                let personnage: &ChrIns = personnage;
                let handle = personnage.field_ins_handle;
                if handle == main_joueur {
                    continue;
                }
                let pv = personnage.modules.data.hp;
                let p = personnage.modules.physics.position;
                let (dx, dy, dz) = (p.0 - pj.0, p.1 - pj.1, p.2 - pj.2);
                let proche = (dx * dx + dy * dy + dz * dz).sqrt() <= DISTANCE_MAX;
                let avant = self.pv.iter().find(|(h, _)| *h == handle).map(|&(_, p)| p);
                if let Some(avant) = avant
                    && pv < avant
                {
                    if self.notes_restantes > 0 {
                        self.notes_restantes -= 1;
                        noter(&format!(
                            "Impact : PV de {handle} (modèle c{:04}) {avant} -> {pv}, en attaque : {en_attaque}, proche : {proche}.",
                            personnage.character_id
                        ));
                    }
                    if en_attaque && proche {
                        touche = true;
                    }
                }
                pv_maintenant.push((handle, pv));
            }
        }
        self.pv = pv_maintenant;
        if touche {
            let Some(ref mut joueur) = monde.main_player else {
                return;
            };
            if effet_present(&joueur.chr_ins, self.main.ligne_effet()) {
                // Encore là : on l'enlève, il sera reposé à l'image suivante.
                joueur.chr_ins.remove_speffect(self.main.ligne_effet() as i32);
                self.a_reposer = true;
            } else {
                joueur.chr_ins.apply_speffect(self.main.ligne_effet() as i32, true);
            }
        }
    }
}

/// Vrai si l'effet d'impact est encore posé sur ce personnage.
fn effet_present(personnage: &ChrIns, ligne_effet: u32) -> bool {
    personnage.special_effect.entries().any(|e| e.param_id == ligne_effet as i32)
}
