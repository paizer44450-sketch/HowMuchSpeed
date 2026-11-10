//! Galerie des sorts : passer en revue tous les sorts du jeu (jeu de base puis extension),
//! pour choisir ensuite les morceaux qui serviront aux roulades, traînées et auras.
//!
//! Tant que la galerie est en marche :
//! - les emplacements de sorts du joueur sont pris par la galerie : le 1er contient le sort
//!   en cours, le 2e le suivant. Changer de sort (haut de la croix directionnelle) passe au
//!   sort suivant de la liste ; la galerie remet aussitôt le nouveau sort dans le 1er
//!   emplacement et le suivant dans le 2e ;
//! - tous les sorts se lancent sans caractéristiques, sans PC, sans endurance, sans place
//!   de mémoire, et même à la Table ronde (MagicParam, en mémoire seulement) ;
//! - les bâtons et les sceaux n'ont plus de caractéristiques requises (EquipParamWeapon,
//!   types 57 et 61), pour lancer avec n'importe quel catalyseur acheté chez Melina.
//!
//! En arrêtant la galerie, tout est remis comme avant : sorts équipés du joueur, fiches des
//! sorts et des catalyseurs. Rien n'est écrit dans le regulation.bin.
//! Le rang et le nom du sort s'affichent en haut de l'écran (voir hud.rs).

use eldenring::cs::{EquipParamWeapon, Magic, SoloParamRepository, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

pub struct Sort {
    /// Ligne de MagicParam (le même numéro que l'objet du sort dans EquipParamGoods).
    pub id: i32,
    /// Nom en français et en anglais (vide pour l'extension, pas encore exportée).
    pub fr: &'static str,
    pub en: &'static str,
}

/// Types d'arme des catalyseurs : bâtons (57) et sceaux sacrés (61).
const TYPES_CATALYSEURS: [u16; 2] = [57, 61];

/// Ce qu'on change dans la fiche d'un sort, pour le remettre à l'arrêt.
#[derive(Clone, Copy)]
struct FicheSort {
    intelligence: u8,
    foi: u8,
    esoterisme: u8,
    pc: i16,
    endurance: i16,
    pc_charge: i16,
    endurance_charge: i16,
    place: u8,
    table_ronde: bool,
}

/// Ce qu'on change dans la fiche d'un catalyseur.
#[derive(Clone, Copy)]
struct FicheCatalyseur {
    force: u8,
    dexterite: u8,
    intelligence: u8,
    foi: u8,
    esoterisme: u8,
}

/// Ce que le joueur avait comme sorts équipés avant la galerie.
struct SortsDuJoueur {
    emplacements: [(i32, i32); 14],
    choisi: i32,
}

pub struct Galerie {
    en_marche: bool,
    /// Rang du sort en cours dans SORTS.
    rang: usize,
    fiches_sorts: Vec<(i32, FicheSort)>,
    fiches_catalyseurs: Vec<(u32, FicheCatalyseur)>,
    sorts_du_joueur: Option<SortsDuJoueur>,
}

impl Galerie {
    pub fn new() -> Self {
        Self { en_marche: false, rang: 0, fiches_sorts: Vec::new(), fiches_catalyseurs: Vec::new(), sorts_du_joueur: None }
    }

    pub fn en_marche(&self) -> bool {
        self.en_marche
    }

    /// Met la galerie en marche (si besoin) et va au sort de ce rang.
    pub fn aller_au_sort(&mut self, rang: usize) {
        self.rang = rang % SORTS.len();
        if !self.en_marche {
            self.debloquer_les_fiches();
            self.en_marche = true;
            noter("Galerie des sorts en marche.");
        }
        self.noter_le_sort();
        self.remplir_les_emplacements(true);
    }

    /// Reprend où on en était (ou au premier sort la première fois).
    pub fn reprendre(&mut self) {
        self.aller_au_sort(self.rang);
    }

    pub fn precedent(&mut self) {
        self.aller_au_sort(self.rang + SORTS.len() - 1);
    }

    /// Arrête la galerie et remet tout comme avant.
    pub fn arreter(&mut self) {
        if !self.en_marche {
            return;
        }
        self.en_marche = false;
        self.rendre_les_sorts_du_joueur();
        self.remettre_les_fiches();
        crate::hud::afficher_galerie(None);
        noter("Galerie des sorts arrêtée, tout est remis comme avant.");
    }

    /// À chaque image : passe au sort suivant quand le joueur a changé de sort, et garde
    /// les deux emplacements remplis.
    pub fn mettre_a_jour(&mut self) {
        if !self.en_marche {
            return;
        }
        crate::hud::afficher_galerie(Some(self.rang));
        let Some(choisi) = donnees_magie().map(|magie| magie.selected_slot) else {
            return;
        };
        if choisi != 0 {
            // Le joueur a changé de sort : le 2e emplacement contenait le suivant.
            self.rang = (self.rang + 1) % SORTS.len();
            self.noter_le_sort();
            self.remplir_les_emplacements(true);
        } else {
            self.remplir_les_emplacements(false);
        }
    }

    fn noter_le_sort(&self) {
        let sort = &SORTS[self.rang];
        noter(&format!("Galerie : sort {}/{} — {} ({}).", self.rang + 1, SORTS.len(), nom_francais(sort), sort.id));
    }

    /// Le sort en cours dans le 1er emplacement, le suivant dans le 2e, les autres vides.
    /// `choisir_le_premier` : remettre la sélection sur le 1er emplacement.
    fn remplir_les_emplacements(&mut self, choisir_le_premier: bool) {
        let Some(magie) = donnees_magie() else {
            return;
        };
        if self.sorts_du_joueur.is_none() {
            let mut emplacements = [(-1, 0); 14];
            for (place, entree) in emplacements.iter_mut().zip(magie.entries.iter()) {
                *place = (entree.param_id, entree.charges);
            }
            noter(&format!("Galerie : sorts du joueur mis de côté {:?}.", emplacements.map(|e| e.0)));
            self.sorts_du_joueur = Some(SortsDuJoueur { emplacements, choisi: magie.selected_slot });
        }
        let suivant = (self.rang + 1) % SORTS.len();
        for (i, entree) in magie.entries.iter_mut().enumerate() {
            entree.param_id = match i {
                0 => SORTS[self.rang].id,
                1 => SORTS[suivant].id,
                _ => -1,
            };
            entree.charges = 0;
        }
        if choisir_le_premier {
            magie.selected_slot = 0;
        }
    }

    fn rendre_les_sorts_du_joueur(&mut self) {
        let Some(avant) = self.sorts_du_joueur.take() else {
            return;
        };
        let Some(magie) = donnees_magie() else {
            noter("Galerie : personnage absent, sorts du joueur non remis.");
            return;
        };
        for (entree, &(id, charges)) in magie.entries.iter_mut().zip(avant.emplacements.iter()) {
            entree.param_id = id;
            entree.charges = charges;
        }
        magie.selected_slot = avant.choisi;
    }

    fn debloquer_les_fiches(&mut self) {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            noter("Galerie : fiches des sorts introuvables.");
            return;
        };
        self.fiches_sorts.clear();
        for sort in SORTS.iter() {
            let Some(fiche) = parametres.get_mut::<Magic>(sort.id as u32) else {
                continue;
            };
            self.fiches_sorts.push((
                sort.id,
                FicheSort {
                    intelligence: fiche.requirement_intellect(),
                    foi: fiche.requirement_faith(),
                    esoterisme: fiche.requirement_luck(),
                    pc: fiche.mp(),
                    endurance: fiche.stamina(),
                    pc_charge: fiche.mp_charge(),
                    endurance_charge: fiche.stamina_charge(),
                    place: fiche.slot_length(),
                    table_ronde: fiche.is_use_no_attack_region(),
                },
            ));
            fiche.set_requirement_intellect(0);
            fiche.set_requirement_faith(0);
            fiche.set_requirement_luck(0);
            fiche.set_mp(0);
            fiche.set_stamina(0);
            fiche.set_mp_charge(0);
            fiche.set_stamina_charge(0);
            fiche.set_slot_length(1);
            fiche.set_is_use_no_attack_region(true);
        }
        self.fiches_catalyseurs.clear();
        for (id, arme) in parametres.rows_mut::<EquipParamWeapon>() {
            if !TYPES_CATALYSEURS.contains(&arme.wep_type()) {
                continue;
            }
            self.fiches_catalyseurs.push((
                id,
                FicheCatalyseur {
                    force: arme.proper_strength(),
                    dexterite: arme.proper_agility(),
                    intelligence: arme.proper_magic(),
                    foi: arme.proper_faith(),
                    esoterisme: arme.proper_luck(),
                },
            ));
            arme.set_proper_strength(0);
            arme.set_proper_agility(0);
            arme.set_proper_magic(0);
            arme.set_proper_faith(0);
            arme.set_proper_luck(0);
        }
        noter(&format!(
            "Galerie : {} sorts et {} catalyseurs sans conditions.",
            self.fiches_sorts.len(),
            self.fiches_catalyseurs.len()
        ));
    }

    fn remettre_les_fiches(&mut self) {
        let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
            return;
        };
        for (id, f) in self.fiches_sorts.drain(..) {
            if let Some(fiche) = parametres.get_mut::<Magic>(id as u32) {
                fiche.set_requirement_intellect(f.intelligence);
                fiche.set_requirement_faith(f.foi);
                fiche.set_requirement_luck(f.esoterisme);
                fiche.set_mp(f.pc);
                fiche.set_stamina(f.endurance);
                fiche.set_mp_charge(f.pc_charge);
                fiche.set_stamina_charge(f.endurance_charge);
                fiche.set_slot_length(f.place);
                fiche.set_is_use_no_attack_region(f.table_ronde);
            }
        }
        for (id, f) in self.fiches_catalyseurs.drain(..) {
            if let Some(arme) = parametres.get_mut::<EquipParamWeapon>(id) {
                arme.set_proper_strength(f.force);
                arme.set_proper_agility(f.dexterite);
                arme.set_proper_magic(f.intelligence);
                arme.set_proper_faith(f.foi);
                arme.set_proper_luck(f.esoterisme);
            }
        }
    }
}

/// Les sorts équipés du joueur (None si le personnage n'est pas chargé).
fn donnees_magie() -> Option<&'static mut eldenring::cs::EquipMagicData> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    let donnees = unsafe { &mut *joueur.player_game_data.as_ptr() };
    Some(unsafe { &mut *donnees.equipment.equip_magic_data.as_ptr() })
}

/// Nom du sort pour le journal et l'affichage en français.
pub fn nom_francais(sort: &Sort) -> &'static str {
    if sort.fr.is_empty() { "Sort de l'extension" } else { sort.fr }
}

/// Nom du sort pour l'affichage en anglais.
pub fn nom_anglais(sort: &Sort) -> &'static str {
    if sort.en.is_empty() { "DLC spell" } else { sort.en }
}

// Liste fabriquée à partir du MagicParam du mod et des noms d'objets (GoodsName) exportés par Smithbox.
// Noms des sorts de l'extension : pas encore exportés (le jeu les affiche quand le sort est équipé).
pub const SORTS: [Sort; 213] = [
    // Sorcelleries du jeu de base
    Sort { id: 4000, fr: "Flèche de pierre d'éclat", en: "Glintstone Pebble" },
    Sort { id: 4001, fr: "Grand fragment de pierre d'éclat", en: "Great Glintstone Shard" },
    Sort { id: 4010, fr: "Fragment de pierre d'éclat rapide", en: "Swift Glintstone Shard" },
    Sort { id: 4020, fr: "Comète de pierre d'éclat", en: "Glintstone Cometshard" },
    Sort { id: 4021, fr: "Comète", en: "Comet" },
    Sort { id: 4030, fr: "Spirale d'éclats", en: "Shard Spiral" },
    Sort { id: 4040, fr: "Étoiles de pierre d'éclat", en: "Glintstone Stars" },
    Sort { id: 4050, fr: "Rafale stellaire", en: "Star Shower" },
    Sort { id: 4060, fr: "Barrage de cristal", en: "Crystal Barrage" },
    Sort { id: 4070, fr: "Arc de pierre d'éclat", en: "Glintstone Arc" },
    Sort { id: 4080, fr: "Canon de Haima", en: "Cannon of Haima" },
    Sort { id: 4090, fr: "Volée de cristal", en: "Crystal Burst" },
    Sort { id: 4100, fr: "Brise-terre", en: "Shatter Earth" },
    Sort { id: 4110, fr: "Perce-roche", en: "Rock Blaster" },
    Sort { id: 4120, fr: "Marteau de Haima", en: "Gavel of Haima" },
    Sort { id: 4130, fr: "Domaine magique", en: "Terra Magica" },
    Sort { id: 4140, fr: "Lumière stellaire", en: "Starlight" },
    Sort { id: 4200, fr: "Comète d'Azur", en: "Comet Azur" },
    Sort { id: 4210, fr: "Pluie d'étoiles primordiale", en: "Founding Rain of Stars" },
    Sort { id: 4220, fr: "Étoiles de la perdition", en: "Stars of Ruin" },
    Sort { id: 4300, fr: "Phalange de laméclats", en: "Glintblade Phalanx" },
    Sort { id: 4301, fr: "Phalange de Caria", en: "Carian Phalanx" },
    Sort { id: 4302, fr: "Phalange de grandes lames", en: "Greatblade Phalanx" },
    Sort { id: 4360, fr: "Pleine lune de Rennala", en: "Rennala's Full Moon" },
    Sort { id: 4361, fr: "Lune noire de Ranni", en: "Ranni's Dark Moon" },
    Sort { id: 4370, fr: "Déluge magique", en: "Magic Downpour" },
    Sort { id: 4380, fr: "Grand arc de Loretta", en: "Loretta's Greatbow" },
    Sort { id: 4381, fr: "Virtuosité de Loretta", en: "Loretta's Mastery" },
    Sort { id: 4390, fr: "Laméclat magique", en: "Magic Glintblade" },
    Sort { id: 4400, fr: "Pierre d'éclat glaciale", en: "Glintstone Icecrag" },
    Sort { id: 4410, fr: "Tempête de givre de Zamor", en: "Zamor Ice Storm" },
    Sort { id: 4420, fr: "Brume glaciale", en: "Freezing Mist" },
    Sort { id: 4430, fr: "Espadon de Caria", en: "Carian Greatsword" },
    Sort { id: 4431, fr: "Lame lunaire d'Adula", en: "Adula's Moonblade" },
    Sort { id: 4440, fr: "Trancheuse de Caria", en: "Carian Slicer" },
    Sort { id: 4450, fr: "Empaleuse de Caria", en: "Carian Piercer" },
    Sort { id: 4460, fr: "Arme d'érudit", en: "Scholar's Armament" },
    Sort { id: 4470, fr: "Bouclier d'érudit", en: "Scholar's Shield" },
    Sort { id: 4480, fr: "Lucidité", en: "Lucidity" },
    Sort { id: 4490, fr: "Arme gelée", en: "Frozen Armament" },
    Sort { id: 4500, fr: "Cristal brisé", en: "Shattering Crystal" },
    Sort { id: 4510, fr: "Explosion de cristal", en: "Crystal Release" },
    Sort { id: 4520, fr: "Torrent de cristal", en: "Crystal Torrent" },
    Sort { id: 4600, fr: "Éclat pernicieux", en: "Ambush Shard" },
    Sort { id: 4610, fr: "Éclat nocturne", en: "Night Shard" },
    Sort { id: 4620, fr: "Comète nocturne", en: "Night Comet" },
    Sort { id: 4630, fr: "Champ de force de Thops", en: "Thops's Barrier" },
    Sort { id: 4640, fr: "Représailles de Caria", en: "Carian Retaliation" },
    Sort { id: 4650, fr: "Ténèbres éternelles", en: "Eternal Darkness" },
    Sort { id: 4660, fr: "Lame indiscernable", en: "Unseen Blade" },
    Sort { id: 4670, fr: "Forme indiscernable", en: "Unseen Form" },
    Sort { id: 4700, fr: "Météorite", en: "Meteorite" },
    Sort { id: 4701, fr: "Météorite d'Astel", en: "Meteorite of Astel" },
    Sort { id: 4710, fr: "Fronde", en: "Rock Sling" },
    Sort { id: 4720, fr: "Puits gravitationnel", en: "Gravity Well" },
    Sort { id: 4721, fr: "Effondrement stellaire", en: "Collapsing Stars" },
    Sort { id: 4800, fr: "Tir de magma", en: "Magma Shot" },
    Sort { id: 4810, fr: "Fureur de Gelmir", en: "Gelmir's Fury" },
    Sort { id: 4820, fr: "Coulée de magma", en: "Roiling Magma" },
    Sort { id: 4830, fr: "Rancœur de Rykard", en: "Rykard's Rancor" },
    Sort { id: 4900, fr: "Ronces du péché", en: "Briars of Sin" },
    Sort { id: 4910, fr: "Ronces du châtiment", en: "Briars of Punishment" },
    Sort { id: 5000, fr: "Invocation vindicative", en: "Rancorcall" },
    Sort { id: 5001, fr: "Antique rancœur mortelle", en: "Ancient Death Rancor" },
    Sort { id: 5010, fr: "Flammes spectrales explosives", en: "Explosive Ghostflame" },
    Sort { id: 5020, fr: "Brume de Fia", en: "Fia's Mist" },
    Sort { id: 5030, fr: "Semonce des os", en: "Tibia's Summons" },
    Sort { id: 5040, fr: "Éclairs funestes", en: "Death Lightning" },
    Sort { id: 5100, fr: "Bulles d'oracle", en: "Oracle Bubbles" },
    Sort { id: 5110, fr: "Grande sphère prophétique", en: "Great Oracular Bubble" },
    // Incantations du jeu de base
    Sort { id: 6000, fr: "Embrasement", en: "Catch Flame" },
    Sort { id: 6001, fr: "Ô Flamme !", en: "O, Flame!" },
    Sort { id: 6010, fr: "Boule de feu ardente", en: "Flame Sling" },
    Sort { id: 6020, fr: "Flamme, abats-toi sur eux !", en: "Flame, Fall Upon Them" },
    Sort { id: 6030, fr: "Tourbillonne, ô Flamme !", en: "Whirl, O Flame!" },
    Sort { id: 6040, fr: "Flamme, purifie-moi", en: "Flame, Cleanse Me" },
    Sort { id: 6050, fr: "Flamme, renforce-moi", en: "Flame, Grant Me Strength" },
    Sort { id: 6060, fr: "Flamme, protège-moi", en: "Flame, Protect Me" },
    Sort { id: 6100, fr: "Grande boule de feu", en: "Giantsflame Take Thee" },
    Sort { id: 6110, fr: "Flamme du dieu cruel", en: "Flame of the Fell God" },
    Sort { id: 6120, fr: "Brûle, ô Flamme !", en: "Burn, O Flame!" },
    Sort { id: 6210, fr: "Boule de feu noire", en: "Black Flame" },
    Sort { id: 6220, fr: "Déferle, ô Flamme !", en: "Surge, O Flame!" },
    Sort { id: 6230, fr: "Fouet de flammes noires", en: "Scouring Black Flame" },
    Sort { id: 6240, fr: "Rituel des flammes noires", en: "Black Flame Ritual" },
    Sort { id: 6250, fr: "Lame de flammes noires", en: "Black Flame Blade" },
    Sort { id: 6260, fr: "Flamme noire protectrice", en: "Black Flame's Protection" },
    Sort { id: 6270, fr: "Noble présence", en: "Noble Presence" },
    Sort { id: 6300, fr: "Serres de feu sanguin", en: "Bloodflame Talons" },
    Sort { id: 6310, fr: "Faveur sanguine", en: "Bloodboon" },
    Sort { id: 6320, fr: "Lame de feu sanguin", en: "Bloodflame Blade" },
    Sort { id: 6330, fr: "Barrière d'or", en: "Barrier of Gold" },
    Sort { id: 6340, fr: "Protection de l'Arbre-Monde", en: "Protection of the Erdtree" },
    Sort { id: 6400, fr: "Rejet", en: "Rejection" },
    Sort { id: 6410, fr: "Courroux doré", en: "Wrath of Gold" },
    Sort { id: 6420, fr: "Soin d'urgence", en: "Urgent Heal" },
    Sort { id: 6421, fr: "Soin", en: "Heal" },
    Sort { id: 6422, fr: "Soins supérieurs", en: "Great Heal" },
    Sort { id: 6423, fr: "Soins seigneuriaux", en: "Lord's Heal" },
    Sort { id: 6424, fr: "Soins de l'Arbre-Monde", en: "Erdtree Heal" },
    Sort { id: 6430, fr: "Bénédiction curative", en: "Blessing's Boon" },
    Sort { id: 6431, fr: "Bénédiction de l'Arbre-Monde", en: "Blessing of the Erdtree" },
    Sort { id: 6440, fr: "Soin du poison", en: "Cure Poison" },
    Sort { id: 6441, fr: "Assistance seigneuriale", en: "Lord's Aid" },
    Sort { id: 6450, fr: "Renforcement ardent", en: "Flame Fortification" },
    Sort { id: 6460, fr: "Renforcement magique", en: "Magic Fortification" },
    Sort { id: 6470, fr: "Renforcement électrique", en: "Lightning Fortification" },
    Sort { id: 6480, fr: "Renforcement divin", en: "Divine Fortification" },
    Sort { id: 6490, fr: "Renforcement divin seigneurial", en: "Lord's Divine Fortification" },
    Sort { id: 6500, fr: "Brume de noctedame", en: "Night Maiden's Mist" },
    Sort { id: 6510, fr: "Approche d'assassin", en: "Assassin's Approach" },
    Sort { id: 6520, fr: "Ombre trompeuse", en: "Shadow Bait" },
    Sort { id: 6530, fr: "Ténèbres", en: "Darkness" },
    Sort { id: 6600, fr: "Serment doré", en: "Golden Vow" },
    Sort { id: 6700, fr: "Disque de lumière", en: "Discus of Light" },
    Sort { id: 6701, fr: "Triple halos de lumière", en: "Triple Rings of Light" },
    Sort { id: 6710, fr: "Halos de lumière de Radagon", en: "Radagon's Rings of Light" },
    Sort { id: 6720, fr: "Étoiles d'Elden", en: "Elden Stars" },
    Sort { id: 6730, fr: "Décret de régression", en: "Law of Regression" },
    Sort { id: 6740, fr: "Bouclier immuable", en: "Immutable Shield" },
    Sort { id: 6750, fr: "Anathème", en: "Litany of Proper Death" },
    Sort { id: 6760, fr: "Décret de causalité", en: "Law of Causality" },
    Sort { id: 6770, fr: "Lame de l'Ordre", en: "Order's Blade" },
    Sort { id: 6780, fr: "Purification de l'Ordre", en: "Order Healing" },
    Sort { id: 6800, fr: "Fronde bestiale", en: "Bestial Sling" },
    Sort { id: 6810, fr: "Pierre de Gurranq", en: "Stone of Gurranq" },
    Sort { id: 6820, fr: "Griffes bestiales", en: "Beast Claw" },
    Sort { id: 6830, fr: "Griffes bestiales de Gurranq", en: "Gurranq's Beast Claw" },
    Sort { id: 6840, fr: "Vitalité bestiale", en: "Bestial Vitality" },
    Sort { id: 6850, fr: "Constitution bestiale", en: "Bestial Constitution" },
    Sort { id: 6900, fr: "Lance de foudre", en: "Lightning Spear" },
    Sort { id: 6910, fr: "Éclair des dragons anciens", en: "Ancient Dragons' Lightning Strike" },
    Sort { id: 6920, fr: "Foudroiement", en: "Lightning Strike" },
    Sort { id: 6921, fr: "Lance de foudre glacée", en: "Frozen Lightning Spear" },
    Sort { id: 6930, fr: "Foudre ciblée", en: "Honed Bolt" },
    Sort { id: 6940, fr: "Lance de foudre des dragons anciens", en: "Ancient Dragons' Lightning Spear" },
    Sort { id: 6941, fr: "Lance de foudre de Fortissax", en: "Fortissax's Lightning Spear" },
    Sort { id: 6950, fr: "Glaive de Lansseax", en: "Lansseax's Glaive" },
    Sort { id: 6960, fr: "Électrification de l'arme", en: "Electrify Armament" },
    Sort { id: 6970, fr: "Éclair draconique de Vyke", en: "Vyke's Dragonbolt" },
    Sort { id: 6971, fr: "Bénédiction électrifiante du dragon", en: "Dragonbolt Blessing" },
    Sort { id: 7000, fr: "Flamme draconique", en: "Dragonfire" },
    Sort { id: 7001, fr: "Flamme d'Agheel", en: "Agheel's Flame" },
    Sort { id: 7010, fr: "Souffle de magma", en: "Magma Breath" },
    Sort { id: 7011, fr: "Magma de Theodorix", en: "Theodorix's Magma" },
    Sort { id: 7020, fr: "Glace draconique", en: "Dragonice" },
    Sort { id: 7021, fr: "Blizzard de Borealis", en: "Borealis's Mist" },
    Sort { id: 7030, fr: "Souffle putréfié", en: "Rotten Breath" },
    Sort { id: 7031, fr: "Putréfaction d'Ekzykes", en: "Ekzykes's Decay" },
    Sort { id: 7040, fr: "Souffle de pierre d'éclat", en: "Glintstone Breath" },
    Sort { id: 7041, fr: "Souffle de pierre d'éclat de Smarag", en: "Smarag's Glintstone Breath" },
    Sort { id: 7050, fr: "Ruine de Placidusax", en: "Placidusax's Ruin" },
    Sort { id: 7060, fr: "Griffe de dragon", en: "Dragonclaw" },
    Sort { id: 7080, fr: "Gueule de dragon", en: "Dragonmaw" },
    Sort { id: 7090, fr: "Cri de Greyoll", en: "Greyoll's Roar" },
    Sort { id: 7200, fr: "Fils putréfiés", en: "Pest Threads" },
    Sort { id: 7210, fr: "Essaim de mouches", en: "Swarm of Flies" },
    Sort { id: 7220, fr: "Nuage de poison", en: "Poison Mist" },
    Sort { id: 7230, fr: "Empoisonnement de l'arme", en: "Poison Armament" },
    Sort { id: 7240, fr: "Floraison écarlate d'Ivoine", en: "Scarlet Aeonia" },
    Sort { id: 7300, fr: "Frénésie irrésistible", en: "Inescapable Frenzy" },
    Sort { id: 7310, fr: "Flamme exaltée", en: "The Flame of Frenzy" },
    Sort { id: 7311, fr: "Frénésie intolérable", en: "Unendurable Frenzy" },
    Sort { id: 7320, fr: "Explosion exaltée", en: "Frenzied Burst" },
    Sort { id: 7330, fr: "Cri de Shabriri", en: "Howl of Shabriri" },
    Sort { id: 7500, fr: "Aspects du Creuset : queue", en: "Aspects of the Crucible: Tail" },
    Sort { id: 7510, fr: "Aspects du Creuset : cornes", en: "Aspects of the Crucible: Horns" },
    Sort { id: 7520, fr: "Aspects du Creuset : souffle", en: "Aspects of the Crucible: Breath" },
    Sort { id: 7530, fr: "Lame d'ébène", en: "Black Blade" },
    Sort { id: 7900, fr: "Flammes pécheresses", en: "Fire's Deadly Sin" },
    Sort { id: 7903, fr: "Renforcement électrique doré", en: "Golden Lightning Fortification" },
    // Sorcelleries de l'extension
    Sort { id: 2004300, fr: "", en: "" },
    Sort { id: 2004310, fr: "", en: "" },
    Sort { id: 2004320, fr: "", en: "" },
    Sort { id: 2004500, fr: "", en: "" },
    Sort { id: 2004510, fr: "", en: "" },
    Sort { id: 2004700, fr: "", en: "" },
    Sort { id: 2004710, fr: "", en: "" },
    Sort { id: 2004900, fr: "", en: "" },
    Sort { id: 2004910, fr: "", en: "" },
    Sort { id: 2005000, fr: "", en: "" },
    // Incantations de l'extension
    Sort { id: 2006200, fr: "", en: "" },
    Sort { id: 2006210, fr: "", en: "" },
    Sort { id: 2006300, fr: "", en: "" },
    Sort { id: 2006400, fr: "", en: "" },
    Sort { id: 2006650, fr: "", en: "" },
    Sort { id: 2006660, fr: "", en: "" },
    Sort { id: 2006670, fr: "", en: "" },
    Sort { id: 2006680, fr: "", en: "" },
    Sort { id: 2006690, fr: "", en: "" },
    Sort { id: 2006700, fr: "", en: "" },
    Sort { id: 2006710, fr: "", en: "" },
    Sort { id: 2006800, fr: "", en: "" },
    Sort { id: 2006900, fr: "", en: "" },
    Sort { id: 2006910, fr: "", en: "" },
    Sort { id: 2006920, fr: "", en: "" },
    Sort { id: 2007000, fr: "", en: "" },
    Sort { id: 2007010, fr: "", en: "" },
    Sort { id: 2007020, fr: "", en: "" },
    Sort { id: 2007200, fr: "", en: "" },
    Sort { id: 2007210, fr: "", en: "" },
    Sort { id: 2007300, fr: "", en: "" },
    Sort { id: 2007410, fr: "", en: "" },
    Sort { id: 2007420, fr: "", en: "" },
    Sort { id: 2007600, fr: "", en: "" },
    Sort { id: 2007700, fr: "", en: "" },
    Sort { id: 2007710, fr: "", en: "" },
    Sort { id: 2007720, fr: "", en: "" },
    Sort { id: 2007730, fr: "", en: "" },
    Sort { id: 2007740, fr: "", en: "" },
    Sort { id: 2007800, fr: "", en: "" },
    Sort { id: 2007810, fr: "", en: "" },
    Sort { id: 2007820, fr: "", en: "" },
];
/// Rang du premier sort de chaque groupe dans SORTS.
pub const DEBUT_INCANTATIONS: usize = 70;
pub const DEBUT_EXTENSION: usize = 171;
pub const DEBUT_INCANTATIONS_EXTENSION: usize = 181;
