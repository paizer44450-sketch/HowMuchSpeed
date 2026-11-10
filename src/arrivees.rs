//! Points d'arrivée devant les bosses.
//!
//! La téléportation du jeu ne sait aller qu'à une grâce. Pour certains bosses, la grâce la
//! plus proche est mal placée (celle de Margit est DANS l'arène : on y réapparaît après
//! chaque mort et le combat ne se relance plus). On part donc d'une grâce de la même
//! carte, puis, à chaque chargement (arrivée et réapparition après une mort), la DLL
//! déplace le joueur juste devant le mur de brume, tourné vers lui.
//!
//! Positions notées en jeu avec F9 : position dans la carte = position physique - position
//! fixe (chunk_position). Ce repère ne se décale pas d'une partie à l'autre.

use eldenring::cs::{BonfireWarpParam, SoloParamRepository, WorldChrMan};
use eldenring::position::HavokPosition;
use eldenring::rotation::Quaternion;
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

pub struct Arrivee {
    /// Nom pour le journal.
    pub nom: &'static str,
    /// Grâce indiquée dans le menu de Melina pour ce boss (sert à le reconnaître).
    pub grace_menu: u32,
    /// Ligne de BonfireWarpParam d'une autre grâce de départ (dans la même carte), si la
    /// grâce du menu ne convient pas. None : on part de la grâce du menu.
    pub ligne_grace: Option<u32>,
    /// Carte où se trouve le point d'arrivée, ou début de son nom : dans le monde ouvert,
    /// la carte change d'un carré à l'autre (ex. « m60_51_3 » couvre m60_51_36 et m60_51_37).
    pub carte: &'static str,
    /// Position dans la carte (x, y, z). Si `relatif` : trajet depuis la grâce d'arrivée.
    pub position: (f32, f32, f32),
    /// Vrai : `position` est un trajet à ajouter à la position d'arrivée à la grâce.
    /// Sert dans le monde ouvert, où le repère du jeu change d'un carré de carte à l'autre.
    pub relatif: bool,
    /// Orientation autour de l'axe vertical, en degrés.
    pub orientation: f32,
    /// Faux : on part seulement de la bonne grâce, sans déplacer le joueur.
    pub placement: bool,
    /// Vrai : on replace aussi le joueur après chaque mort. Faux : seulement à l'arrivée
    /// depuis le menu (quand une effigie de Marika fait déjà réapparaître devant la brume).
    pub apres_mort: bool,
    /// Monde ouvert : carré de carte exigé comme origine du repère (ex. « m60_52_53_00 »).
    /// Si Some, `position` est la position dans la carte de CE carré (celle des fichiers
    /// MSB, = position physique - position fixe quand l'origine est ce carré), et le joueur
    /// n'est placé que si l'origine du repère du jeu est bien ce carré ; sinon on ne le
    /// déplace pas (il marche depuis la grâce). Vérifié le 06/11/2026 : la mesure F9 du
    /// Géant de feu tombe sur la zone de début de combat du fichier MSB m60_52_53_00.
    pub repere_exige: Option<&'static str>,
}

/// Points d'arrivée connus (mesures F9 du 06/11/2026).
pub const ARRIVEES: [Arrivee; 19] = [
    // Margit : sa grâce est dans l'arène. Départ de la grâce « Tunnel du château »
    // (ligne 100002), dans la même carte, puis placement devant la brume.
    Arrivee {
        nom: "Margit",
        grace_menu: 10001951,
        ligne_grace: Some(100002),
        carte: "m10_00_00_00",
        position: (22.44, -7.42, -11.66),
        orientation: 87.8,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Godrick : départ de la grâce « Cellule isolée », puis placement devant la brume.
    Arrivee {
        nom: "Godrick",
        grace_menu: 10001957,
        ligne_grace: None,
        carte: "m10_00_00_00",
        position: (-202.00, 81.87, 313.06),
        orientation: 139.0,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Rennala : départ de la grâce « Salle de débat », puis placement devant la brume.
    Arrivee {
        nom: "Rennala",
        grace_menu: 14001951,
        ligne_grace: None,
        carte: "m14_00_00_00",
        position: (96.56, 154.08, -46.55),
        orientation: 112.8,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Radahn : sa grâce est dans l'arène des dunes. Départ de la grâce « Place du château
    // du Lion rouge » (ligne 64513600), puis placement devant le portail, en position
    // dans la carte du carré m60_51_36_00 (vérifié le 06/11/2026 : la mesure F9 à
    // l'arrivée tombe exactement sur le point d'arrivée c1000_9005 du fichier MSB).
    // Mesure F9 au portail : physique (7.37, -6.98, 25.18), position fixe (-56, -8, -152).
    Arrivee {
        nom: "Radahn",
        grace_menu: 1052381950,
        ligne_grace: Some(64513600),
        carte: "m60_",
        position: (63.37, 0.98, 177.18),
        orientation: -156.4,
        relatif: false,
        placement: true,
        repere_exige: Some("m60_51_36_00"),
        apres_mort: true,
    },
    // Morgott : sa grâce (« Trône d'Elden ») est dans l'arène. Départ de la grâce
    // « Chambre de la reine » (ligne 110007), puis placement devant la brume.
    Arrivee {
        nom: "Morgott",
        grace_menu: 11001950,
        ligne_grace: Some(110007),
        carte: "m11_00_00_00",
        position: (3.47, 64.99, -371.89),
        orientation: -35.2,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Géant de feu : sa grâce est dans l'arène. Départ de la grâce « Pied de la forge »
    // (ligne 65525300, la plus proche), puis placement seulement si le jeu est dans le même repère qu'à la
    // mesure F9 (06/11/2026) ; sinon, marche depuis la grâce.
    Arrivee {
        nom: "Géant de feu",
        grace_menu: 1053521950,
        ligne_grace: Some(65525300),
        carte: "m60_",
        // Deuxième entrée de l'arène (mesure F9 : physique (8.28, 1.19, -12.54), position
        // fixe (-8, -1808, 0), carte m60_52_53_00), juste à côté de la zone de début de
        // combat « 巨人ボス戦開始領域_0 » (20.03, 1809.15, -16.98) du fichier MSB.
        position: (16.28, 1809.19, -12.54),
        orientation: -35.8,
        relatif: false,
        placement: true,
        repere_exige: Some("m60_52_53_00"),
        apres_mort: true,
    },
    // Maliketh : sa grâce est dans l'arène. Départ de la grâce « À côté du grand pont »
    // (ligne 130010), puis placement devant la brume (mesure F9 du 06/11/2026).
    Arrivee {
        nom: "Maliketh",
        grace_menu: 13001950,
        ligne_grace: Some(130010),
        carte: "m13_00_00_00",
        position: (63.42, -51.88, 459.10),
        orientation: -21.8,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Godfrey (Leyndell en cendres) : sa grâce (« Trône d'Elden ») est dans l'arène.
    // Départ de la grâce « Chambre de la reine » (ligne 110504), puis placement devant la
    // brume (mesure F9 du 06/11/2026).
    Arrivee {
        nom: "Godfrey",
        grace_menu: 11051950,
        ligne_grace: Some(110504),
        carte: "m11_05_00_00",
        position: (3.62, 64.98, -372.10),
        orientation: -37.1,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: true,
    },
    // Mohg (palais de Mohgwyn) : sa grâce (« Cocon de l'Empyréen ») est dans l'arène.
    // Départ de la grâce « Mausolée de la dynastie, mi-parcours » (ligne 120503), puis
    // placement devant la brume (mesure F9 du 06/11/2026). Après une mort, pas de
    // placement : l'effigie de Marika fait déjà réapparaître au même endroit.
    Arrivee {
        nom: "Mohg",
        grace_menu: 12051950,
        ligne_grace: Some(120503),
        carte: "m12_05_00_00",
        position: (1616.99, -505.78, 1204.39),
        orientation: 115.1,
        relatif: false,
        placement: true,
        repere_exige: None,
        apres_mort: false,
    },
    // Radagon / Bête d'Elden : essai du 06/11/2026 : depuis le Trône d'Elden, l'arène
    // (carte m19_00, derrière la brume) ne se chargeait pas. On part donc directement de la
    // grâce « Marika brisée », dans l'arène, sans placement. L'ancienne mesure devant la
    // brume est gardée ci-dessous.
    Arrivee {
        nom: "Radagon",
        grace_menu: 19001950,
        ligne_grace: None,
        carte: "m11_05_00_00",
        position: (74.42, 79.06, -464.36),
        orientation: -37.6,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: true,
    },
    // Rykard : départ de la grâce « Allée de l'Audience » (Audience Pathway, ligne
    // 160005), lue dans la liste des grâces du jeu : le numéro calculé 16001955 envoyait à
    // la porte du manoir du volcan. Sans placement.
    Arrivee {
        nom: "Rykard",
        grace_menu: 16001955,
        ligne_grace: Some(160005),
        carte: "m16_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Rellana (DLC, château d'Ensis) : départ de la grâce « Chambre du seigneur du
    // château » (ligne 61484401), juste devant le combat, sans placement.
    Arrivee {
        nom: "Rellana",
        grace_menu: 2048441951,
        ligne_grace: Some(61484401),
        carte: "m61_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Bayle (DLC) : départ de la grâce « Sommet du pic déchiqueté » (ligne 68543900), juste devant le combat.
    Arrivee {
        nom: "Bayle",
        grace_menu: 2054391950,
        ligne_grace: Some(68543900),
        carte: "m61_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Messmer (DLC) : départ de la grâce « Entrée de la chambre obscure » (ligne 210104), juste devant le combat.
    Arrivee {
        nom: "Messmer",
        grace_menu: 21011954,
        ligne_grace: Some(210104),
        carte: "m21_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Avatar de l'arbre occulte (DLC) : départ de la grâce « Sanctuaire de l'arbre vénéré »
    // (Tree-Worship Sanctum, ligne 210009, quartier de l'église du château noir), juste
    // devant le combat.
    Arrivee {
        nom: "Avatar de l'arbre occulte",
        grace_menu: 2050481951,
        ligne_grace: Some(210009),
        carte: "m21_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Gaius (DLC) : départ de la grâce « Château noir, porte arrière » (ligne 69494801), juste devant le combat.
    Arrivee {
        nom: "Gaius",
        grace_menu: 2049481951,
        ligne_grace: Some(69494801),
        carte: "m61_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Romina (DLC) : départ de la grâce « Église du Bourgeon, narthex » (ligne 69444600), juste devant le combat.
    Arrivee {
        nom: "Romina",
        grace_menu: 2044461950,
        ligne_grace: Some(69444600),
        carte: "m61_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Radahn Consort (DLC) : départ de la grâce « Escalier devant le portail divin » (ligne 200106), juste devant le combat.
    Arrivee {
        nom: "Radahn Consort",
        grace_menu: 20011956,
        ligne_grace: Some(200106),
        carte: "m20_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
    // Metyr (DLC) — RETIRÉ DU MENU pour le moment (entrée gardée pour plus tard) : son arène est un lieu à part (Terres natales des doigts), où l'on n'entre
    // normalement qu'en sonnant la cloche des ruines de Miyr, au bout de la quête du comte
    // Ymir. Comme pour Radagon, on part directement de la grâce de l'arène, « Terres natales
    // des doigts » (ligne 250000), sans placement. (Ancienne mesure devant la cathédrale :
    // carte m61_51_45_00, position (-101.45, 420.78, 137.19), orientation 177.2.)
    Arrivee {
        nom: "Metyr",
        grace_menu: 2051451950,
        ligne_grace: Some(250000),
        carte: "m25_",
        position: (0.0, 0.0, 0.0),
        orientation: 0.0,
        relatif: false,
        placement: false,
        repere_exige: None,
        apres_mort: false,
    },
];

/// Le point d'arrivée du boss dont la grâce du menu est donnée.
pub fn pour_le_boss(grace_menu: u32) -> Option<&'static Arrivee> {
    ARRIVEES.iter().find(|a| a.grace_menu == grace_menu)
}

/// Numéro (entité) de la grâce de départ.
pub fn grace_de_depart(arrivee: &Arrivee) -> Option<u32> {
    let Some(ligne) = arrivee.ligne_grace else {
        return Some(arrivee.grace_menu);
    };
    let parametres = unsafe { SoloParamRepository::instance() }.ok()?;
    let entite = parametres.get::<BonfireWarpParam>(ligne)?.bonfire_entity_id();
    (entite != 0).then_some(entite)
}

/// Résultat d'un essai de placement.
pub enum Placement {
    /// Joueur placé depuis cette position (celle de la grâce, en général).
    Place(HavokPosition),
    /// Rien à faire : déjà sur place, ou le joueur s'est déplacé lui-même.
    RienAFaire,
    /// Pas encore possible (personnage ou carte pas encore prêts) : réessayer plus tard.
    /// Contient la carte où le jeu dit que se trouve le joueur, si on la connaît.
    PasPret(Option<String>),
}

/// Distance (en mètres) en dessous de laquelle le joueur est considéré déjà sur place.
const DISTANCE_SUR_PLACE: f32 = 2.0;

/// Place le joueur au point d'arrivée, une fois qu'il est bien dans la bonne carte.
/// S'il y est déjà (à moins de 2 m), ne fait rien.
///
/// `revenu_a` : lors d'une vérification, la position (dans le repère de la carte, c.-à-d.
/// physique - position fixe) d'où on l'avait déplacé. On ne le
/// replace que s'il y est revenu (le jeu l'a remis à la grâce) ; s'il est ailleurs, c'est
/// qu'il s'est déplacé lui-même (vers la brume) : on le laisse.
pub fn placer_le_joueur(arrivee: &Arrivee, revenu_a: Option<HavokPosition>) -> Placement {
    let Ok(monde) = (unsafe { WorldChrMan::instance_mut() }) else {
        return Placement::PasPret(None);
    };
    let Some(joueur) = monde.main_player.as_mut() else {
        return Placement::PasPret(None);
    };
    let chr = &mut joueur.chr_ins;
    let carte = format!("{}", chr.block_id);
    if !carte.starts_with(arrivee.carte) {
        let c = chr.chunk_position;
        return Placement::PasPret(Some(format!(
            "{carte} (origine {}, repère ({:.2}, {:.2}, {:.2}))",
            chr.block_origin, c.0, c.1, c.2
        )));
    }
    let actuelle = chr.modules.physics.position;
    let repere = chr.chunk_position;
    let (x, y, z) = arrivee.position;
    let cible = if let Some(origine_exigee) = arrivee.repere_exige {
        // Monde ouvert : l'origine du repère doit être le carré de la mesure (« m255 »
        // veut dire : le carré où se trouve le joueur), sinon pas de placement.
        let origine = format!("{}", chr.block_origin);
        let origine = if origine.starts_with("m255") { carte.clone() } else { origine };
        if origine != origine_exigee {
            if revenu_a.is_none() {
                noter(&format!(
                    "{} : repère différent de la mesure (origine {origine}, repère ({:.2}, {:.2}, {:.2})), pas de placement : marche depuis la grâce.",
                    arrivee.nom, repere.0, repere.1, repere.2
                ));
            }
            return Placement::RienAFaire;
        }
        HavokPosition::from_xyz(x + repere.0, y + repere.1, z + repere.2)
    } else if arrivee.relatif {
        // Trajet ajouté à la position d'arrivée à la grâce (celle d'avant le placement).
        let base = match revenu_a {
            Some(a) => HavokPosition::from_xyz(a.0 + repere.0, a.1 + repere.1, a.2 + repere.2),
            None => actuelle,
        };
        HavokPosition::from_xyz(base.0 + x, base.1 + y, base.2 + z)
    } else {
        HavokPosition::from_xyz(x + repere.0, y + repere.1, z + repere.2)
    };

    let (dx, dy, dz) = (actuelle.0 - cible.0, actuelle.1 - cible.1, actuelle.2 - cible.2);
    if (dx * dx + dy * dy + dz * dz).sqrt() <= DISTANCE_SUR_PLACE {
        return Placement::RienAFaire;
    }
    if let Some(avant) = revenu_a {
        // Comparaison dans le repère de la carte : le repère physique du jeu peut avoir été
        // recalé entre-temps (sinon faux « retour à la grâce » et joueur ramené en arrière).
        let (ax, ay, az) = (
            actuelle.0 - repere.0 - avant.0,
            actuelle.1 - repere.1 - avant.1,
            actuelle.2 - repere.2 - avant.2,
        );
        if (ax * ax + ay * ay + az * az).sqrt() > DISTANCE_SUR_PLACE {
            return Placement::RienAFaire;
        }
        noter(&format!("{} : le jeu a remis le joueur à la grâce, nouveau placement.", arrivee.nom));
    }

    let demi_angle = arrivee.orientation.to_radians() / 2.0;
    let rotation = Quaternion(0.0, demi_angle.sin(), 0.0, demi_angle.cos());
    // Gravité coupée le temps que le sol autour du point d'arrivée se charge (la DLL la
    // rétablit quelques secondes plus tard) : sinon, dans le monde ouvert, le joueur
    // traverse un sol pas encore chargé et tombe dans le vide.
    chr.chr_flags1c4.set_no_gravity(true);
    // Le jeu compterait le déplacement (jusqu'à 90 m plus bas pour Radahn) comme une
    // chute mortelle : on remet à zéro tout ce qui mesure une chute.
    chr.modules.fall.fall_timer = 0.0;
    let physique = &mut chr.modules.physics;
    physique.last_update_position = cible;
    physique.is_falling = false;
    physique.position = cible;
    physique.orientation = rotation;
    physique.interpolated_orientation = rotation;
    physique.chr_proxy_pos_update_requested = true;
    noter(&format!(
        "{} : joueur placé devant la brume (carte {carte}, repère ({:.2}, {:.2}, {:.2}), depuis ({:.2}, {:.2}, {:.2}) vers ({:.2}, {:.2}, {:.2})).",
        arrivee.nom, repere.0, repere.1, repere.2,
        actuelle.0, actuelle.1, actuelle.2, cible.0, cible.1, cible.2
    ));
    // Position de départ, dans le repère de la carte (stable même si le repère est recalé).
    Placement::Place(HavokPosition::from_xyz(
        actuelle.0 - repere.0,
        actuelle.1 - repere.1,
        actuelle.2 - repere.2,
    ))
}

/// Rétablit la gravité du joueur (après un placement).
pub fn retablir_la_gravite() {
    if let Ok(monde) = unsafe { WorldChrMan::instance_mut() }
        && let Some(joueur) = monde.main_player.as_mut()
    {
        joueur.chr_ins.modules.fall.fall_timer = 0.0;
        joueur.chr_ins.chr_flags1c4.set_no_gravity(false);
    }
}

/// Pendant que la gravité est coupée après un placement : empêche le jeu de compter une
/// chute (sinon il tue le joueur à l'atterrissage, comme après une grande chute).
pub fn annuler_la_chute() {
    if let Ok(monde) = unsafe { WorldChrMan::instance_mut() }
        && let Some(joueur) = monde.main_player.as_mut()
    {
        let chr = &mut joueur.chr_ins;
        chr.modules.fall.fall_timer = 0.0;
        chr.modules.physics.is_falling = false;
    }
}
