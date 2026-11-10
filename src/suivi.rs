//! Outil de mesure : suivre les effets (SpEffect) qui apparaissent et disparaissent, et
//! les animations jouées, sur le joueur et sur le personnage le plus proche (un boss, par
//! exemple Radagon quand il se téléporte).
//!
//! F11 : début du suivi (écran légèrement teinté de rouge tant qu'il est actif), F11 de
//! nouveau : fin du suivi et résumé dans le journal. Chaque effet nouveau est noté avec ses
//! visuels (SpEffectVfx : visuels de début, continu et de fin, camouflage), sa durée, son
//! icône et sa catégorie. Chaque changement d'animation est noté avec son numéro.
//!
//! Drapeaux d'évènement (08/11/2026) : au début du suivi, une copie de tous les drapeaux
//! d'évènement du jeu ; à la fin, la liste de ceux qui ont changé (allumés ou éteints).
//! Sert à trouver le drapeau d'un évènement du jeu (ex. le déblocage des fioles aux grâces
//! après le premier miniboss).

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use eldenring::cs::{
    CSEventFlagMan, ChrIns, FieldInsHandle, FlagBlockDescriptor, SoloParamRepository,
    SpEffectVfxParam, WorldChrMan,
};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Distance maximale (m) du personnage suivi en plus du joueur.
const DISTANCE_CIBLE: f32 = 60.0;
/// Melina (Table ronde) n'est jamais prise comme cible.
const ENTITE_MELINA: u32 = 11105950;
/// Modèle c1000 : personnages invisibles des grâces, miroirs, etc. (mesure du 07/11/2026 :
/// pris comme cible à 7 m à la Table ronde). Jamais pris comme cible.
const MODELE_INVISIBLE: u32 = 1000;
/// Modèles c8000 à c8099 : Torrent et autres montures (mesure du 07/11/2026 : Torrent,
/// c8002, pris comme cible à 1,5 m à la place de Mohg). Jamais pris comme cible.
const MODELES_MONTURES: std::ops::Range<u32> = 8000..8100;

/// Ce qu'on retient pour un personnage suivi.
struct Sujet {
    nom: String,
    handle: FieldInsHandle,
    /// Effets présents au début du suivi (ignorés).
    au_debut: HashSet<i32>,
    /// Effets apparus pendant le suivi : description notée, encore présent ou non.
    apparus: HashMap<i32, (String, bool)>,
    /// Ordre d'apparition.
    ordre: Vec<i32>,
    /// Dernière animation vue.
    animation: i32,
    /// Animations jouées pendant le suivi, dans l'ordre (sans doublons à la suite).
    animations: Vec<i32>,
}

impl Sujet {
    fn new(nom: String, personnage: &ChrIns) -> Self {
        let animation = animation_de(personnage);
        Self {
            nom,
            handle: personnage.field_ins_handle,
            au_debut: effets_de(personnage).into_iter().map(|(id, _)| id).collect(),
            apparus: HashMap::new(),
            ordre: Vec::new(),
            animation,
            animations: vec![animation],
        }
    }

    fn mettre_a_jour(&mut self, personnage: &ChrIns, t: f32) {
        let animation = animation_de(personnage);
        if animation != self.animation {
            self.animation = animation;
            self.animations.push(animation);
            noter(&format!("Suivi ({t:.1} s, {}) : animation {animation}.", self.nom));
        }
        let effets = effets_de(personnage);
        let presents: HashSet<i32> = effets.iter().map(|(id, _)| *id).collect();
        for (id, description) in effets {
            if self.au_debut.contains(&id) {
                continue;
            }
            match self.apparus.get_mut(&id) {
                Some((_, present)) => {
                    if !*present {
                        *present = true;
                        noter(&format!("Suivi ({t:.1} s, {}) : réapparu {id}.", self.nom));
                    }
                }
                None => {
                    noter(&format!("Suivi ({t:.1} s, {}) : apparu {description}.", self.nom));
                    self.apparus.insert(id, (description, true));
                    self.ordre.push(id);
                }
            }
        }
        for (id, (_, present)) in self.apparus.iter_mut() {
            if *present && !presents.contains(id) {
                *present = false;
                noter(&format!("Suivi ({t:.1} s, {}) : disparu {id}.", self.nom));
            }
        }
    }

    fn resumer(&self) {
        noter(&format!(
            "Suivi : FIN pour {}. {} effet(s) apparu(s) :",
            self.nom,
            self.ordre.len()
        ));
        for id in &self.ordre {
            if let Some((description, present)) = self.apparus.get(id) {
                noter(&format!(
                    "  - {description} ; {}",
                    if *present { "encore présent à la fin" } else { "disparu avant la fin" }
                ));
            }
        }
        noter(&format!("  Animations de {} : {:?}", self.nom, self.animations));
    }
}

/// Ce que l'outil retient pendant un suivi.
pub struct Suivi {
    actif: bool,
    debut: Option<Instant>,
    sujets: Vec<Sujet>,
    /// Copie des drapeaux d'évènement au début du suivi (groupe -> ses 1000 drapeaux).
    drapeaux_debut: Option<HashMap<u32, [u8; TAILLE_BLOC]>>,
}

/// Un groupe de 1000 drapeaux d'évènement : 125 octets (un bit par drapeau).
const TAILLE_BLOC: usize = 125;
/// Au-delà, la liste des drapeaux changés est coupée (journal lisible).
const DRAPEAUX_MAX: usize = 500;

/// Copie de tous les drapeaux d'évènement du jeu : groupe (numéro / 1000) -> ses octets.
///
/// La table des groupes (fromsoftware-rs, CSFD4VirtualMemoryFlag) donne pour chaque
/// groupe un descripteur : mode (u32) puis, 8 octets plus loin, soit le rang du bloc dans
/// la réserve commune (mode 1), soit l'adresse du bloc (mode 2). fromsoftware-rs ne rend
/// pas cette lecture publique : on la refait ici, en lecture seule.
fn copier_drapeaux() -> Option<HashMap<u32, [u8; TAILLE_BLOC]>> {
    let gestionnaire = unsafe { CSEventFlagMan::instance() }.ok()?;
    let memoire = &gestionnaire.virtual_memory_flag;
    let reserve = memoire.flag_blocks as *const u8;
    let mut copie = HashMap::new();
    for paire in memoire.flag_block_descriptors.iter() {
        let descripteur = &paire.second as *const FlagBlockDescriptor as *const u8;
        let bloc: *const u8 = unsafe {
            match *(descripteur as *const u32) {
                1 if !reserve.is_null() => {
                    let rang = *(descripteur.add(8) as *const u32) as usize;
                    reserve.add(rang * TAILLE_BLOC)
                }
                2 => *(descripteur.add(8) as *const *const u8),
                _ => continue,
            }
        };
        if bloc.is_null() {
            continue;
        }
        let mut octets = [0u8; TAILLE_BLOC];
        unsafe { std::ptr::copy_nonoverlapping(bloc, octets.as_mut_ptr(), TAILLE_BLOC) };
        copie.insert(paire.first, octets);
    }
    Some(copie)
}

/// Note dans le journal les drapeaux qui ont changé entre deux copies.
fn noter_drapeaux_changes(avant: &HashMap<u32, [u8; TAILLE_BLOC]>) {
    let Some(apres) = copier_drapeaux() else {
        noter("Suivi : drapeaux d'évènement illisibles.");
        return;
    };
    let mut changes: Vec<(u32, bool)> = Vec::new();
    for (&groupe, octets) in &apres {
        let ancien = avant.get(&groupe).copied().unwrap_or([0u8; TAILLE_BLOC]);
        for (i, (&a, &b)) in ancien.iter().zip(octets.iter()).enumerate() {
            if a == b {
                continue;
            }
            for bit in 0..8 {
                // Même ordre que le jeu : le drapeau 0 du groupe est le bit de poids fort.
                let masque = 0b1000_0000u8 >> bit;
                if a & masque != b & masque {
                    let drapeau = groupe * 1000 + (i * 8 + bit) as u32;
                    changes.push((drapeau, b & masque != 0));
                }
            }
        }
    }
    changes.sort();
    noter(&format!(
        "Suivi : {} drapeau(x) d'évènement changé(s) ({} groupes lus).",
        changes.len(),
        apres.len()
    ));
    for (drapeau, allume) in changes.iter().take(DRAPEAUX_MAX) {
        let etat = if *allume { "allumé" } else { "éteint" };
        noter(&format!("  Drapeau {drapeau} : {etat}"));
    }
    if changes.len() > DRAPEAUX_MAX {
        noter(&format!("  ... et {} autres.", changes.len() - DRAPEAUX_MAX));
    }
}

impl Suivi {
    pub fn new() -> Self {
        Self { actif: false, debut: None, sujets: Vec::new(), drapeaux_debut: None }
    }

    pub fn est_actif(&self) -> bool {
        self.actif
    }

    /// F11 : début ou fin du suivi.
    pub fn basculer(&mut self) {
        if self.actif {
            self.arreter();
        } else {
            self.commencer();
        }
    }

    fn commencer(&mut self) {
        let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
            noter("Suivi : personnage pas chargé, suivi non lancé.");
            return;
        };
        let Some(joueur) = monde.main_player.as_ref() else {
            noter("Suivi : personnage pas chargé, suivi non lancé.");
            return;
        };
        let joueur = &joueur.chr_ins;
        self.sujets.clear();
        self.sujets.push(Sujet::new("joueur".to_string(), joueur));
        // Le personnage le plus proche (hors joueur et Melina), à moins de DISTANCE_CIBLE.
        let pj = joueur.modules.physics.position;
        let mut plus_proche: Option<(f32, &ChrIns)> = None;
        for liste in monde.chr_sets.iter().flatten() {
            for personnage in liste.characters() {
                let personnage: &ChrIns = personnage;
                if personnage.field_ins_handle == joueur.field_ins_handle
                    || personnage.event_entity_id == ENTITE_MELINA
                    || personnage.character_id == MODELE_INVISIBLE
                    || MODELES_MONTURES.contains(&personnage.character_id)
                {
                    continue;
                }
                let p = personnage.modules.physics.position;
                let (dx, dy, dz) = (p.0 - pj.0, p.1 - pj.1, p.2 - pj.2);
                let d = (dx * dx + dy * dy + dz * dz).sqrt();
                let plus_pres = match plus_proche {
                    Some((dmin, _)) => d < dmin,
                    None => true,
                };
                if d <= DISTANCE_CIBLE && plus_pres {
                    plus_proche = Some((d, personnage));
                }
            }
        }
        match plus_proche {
            Some((d, cible)) => {
                let nom = format!("cible PNJ {}", cible.npc_param_id);
                noter(&format!(
                    "Suivi : cible = personnage le plus proche, PNJ {} (modèle {}), à {d:.1} m.",
                    cible.npc_param_id, cible.character_id
                ));
                self.sujets.push(Sujet::new(nom, cible));
            }
            None => noter(&format!("Suivi : aucun personnage à moins de {DISTANCE_CIBLE} m.")),
        }
        self.debut = Some(Instant::now());
        self.actif = true;
        self.drapeaux_debut = copier_drapeaux();
        match &self.drapeaux_debut {
            Some(d) => noter(&format!("Suivi : drapeaux d'évènement copiés ({} groupes).", d.len())),
            None => noter("Suivi : drapeaux d'évènement illisibles."),
        }
        for s in &self.sujets {
            let mut ids: Vec<i32> = s.au_debut.iter().copied().collect();
            ids.sort();
            noter(&format!(
                "Suivi : DÉBUT ({}). {} effet(s) déjà présents (ignorés) : {:?} ; animation {}.",
                s.nom,
                ids.len(),
                ids,
                s.animation
            ));
        }
        crate::fondu::teinte_suivi(true);
    }

    fn arreter(&mut self) {
        self.actif = false;
        crate::fondu::teinte_suivi(false);
        for s in &self.sujets {
            s.resumer();
        }
        if let Some(avant) = self.drapeaux_debut.take() {
            noter_drapeaux_changes(&avant);
        }
    }

    /// À appeler à chaque image.
    pub fn mettre_a_jour(&mut self) {
        if !self.actif {
            return;
        }
        let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
            return;
        };
        let t = self.debut.map(|d| d.elapsed().as_secs_f32()).unwrap_or(0.0);
        for liste in monde.chr_sets.iter().flatten() {
            for personnage in liste.characters() {
                for s in self.sujets.iter_mut() {
                    if s.handle == personnage.field_ins_handle {
                        s.mettre_a_jour(personnage, t);
                    }
                }
            }
        }
    }
}

/// Animation en cours d'un personnage (la dernière jouée de sa file d'animations).
fn animation_de(personnage: &ChrIns) -> i32 {
    let ta = &personnage.modules.time_act;
    ta.anim_queue[(ta.read_idx as usize) % ta.anim_queue.len()].anim_id
}

/// Description d'un visuel d'effet : visuels de début, continu et de fin, camouflage.
fn decrire_visuel(vfx: i32) -> String {
    let Ok(parametres) = (unsafe { SoloParamRepository::instance() }) else {
        return format!("[visuel {vfx}]");
    };
    match parametres.get::<SpEffectVfxParam>(vfx as u32) {
        Some(v) => format!(
            "[visuel {vfx} : début {} (point {}), continu {} (point {}), fin {} (point {}), type {}, camouflage {}]",
            v.init_sfx_id(),
            v.init_dmy_id(),
            v.midst_sfx_id(),
            v.midst_dmy_id(),
            v.finish_sfx_id(),
            v.finish_dmy_id(),
            v.effect_type(),
            v.use_camouflage()
        ),
        None => format!("[visuel {vfx} introuvable]"),
    }
}

/// Effets actifs d'un personnage : (numéro, description lisible).
pub fn effets_de(personnage: &ChrIns) -> Vec<(i32, String)> {
    let mut liste = Vec::new();
    for e in personnage.special_effect.entries() {
        let mut description = format!("effet {} (durée {:.1} s)", e.param_id, e.duration);
        if let Some(p) = e.param_data {
            let p = unsafe { p.as_ref() };
            let visuels = [
                p.vfx_id(),
                p.vfx_id1(),
                p.vfx_id2(),
                p.vfx_id3(),
                p.vfx_id4(),
                p.vfx_id5(),
                p.vfx_id6(),
                p.vfx_id7(),
            ];
            description.push_str(&format!(
                ", visuels {:?}, icône {}, catégorie {}, info d'état {}, change l'arme {}",
                visuels,
                p.icon_id(),
                p.sp_category(),
                p.state_info(),
                p.wep_param_change()
            ));
            for v in visuels.into_iter().filter(|&v| v > 0) {
                description.push(' ');
                description.push_str(&decrire_visuel(v));
            }
        }
        liste.push((e.param_id, description));
    }
    liste
}
