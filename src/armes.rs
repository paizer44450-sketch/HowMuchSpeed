//! Magasins de Melina (armes, armures, talismans, sorts, matériaux ; jeu de base puis DLC
//! dans chaque magasin) : de vraies lignes de magasin dans les données
//! du mod (regulation.bin du dossier modengine\mod, ShopLineupParam 9600000 et suivantes pour les armes, 9610000 et suivantes pour les
//! armures, 9620000 et suivantes pour les talismans).
//! Le fichier est fabriqué par outils\build_reg.py à partir du fichier d'origine du jeu et
//! de outils\magasins_tous.csv (armes puis armures), jamais enregistré par Smithbox (qui ajoute les noms de
//! lignes : fichier trop gros, et le jeu annonce une sauvegarde corrompue).
//! Chaque objet : stock illimité, prix = prix de revente + 100 (le jeu cache un objet vendu
//! moins cher que sa revente) ; armes à +0. L'achat est rendu gratuit par le menu : runes au
//! maximum pendant le magasin, rendues à la fermeture. Le jeu affiche la vraie grille du
//! magasin (icônes, statistiques). Munitions : plus tard.

/// Un magasin : son texte dans le menu, sa première et sa dernière ligne de magasin.
pub struct Magasin {
    pub texte: i32,
    pub premiere_ligne: i32,
    pub derniere_ligne: i32,
}

/// Un rayon de Melina : une liste de magasins. Chaque magasin contient les objets du jeu
/// de base puis ceux du DLC.
#[derive(Clone, Copy, PartialEq)]
pub enum Rayon {
    /// Dans « Équipement ».
    Armes,
    Armures,
    /// Un seul magasin : on l'ouvre directement depuis « Marchands ».
    Talismans,
    /// Deux magasins : sorcelleries et incantations.
    Sorts,
    /// Un seul magasin : les matériaux de confection.
    Materiaux,
    /// Un seul magasin, dans « Équipement » : les cendres de guerre.
    Cendres,
    /// Un seul magasin, dans « Niveau » : graines dorées, larmes sacrées, fragments de
    /// l'Arbre occulte et cendres vénérées (à utiliser ensuite à une grâce).
    Ameliorations,
    /// Trois magasins, dans le sous-menu « Cendres d'esprit » : cendres au niveau +0,
    /// au niveau +10, et muguets (pour les améliorer chez Melina).
    Esprits,
    /// Un seul magasin, dans « Équipement » : flèches, carreaux, grandes flèches et
    /// grands carreaux (jeu de base puis DLC).
    Munitions,
}

impl Rayon {
    pub fn magasins(self) -> &'static [Magasin] {
        match self {
            Rayon::Armes => MAGASINS_ARMES,
            Rayon::Armures => MAGASINS_ARMURES,
            Rayon::Talismans => MAGASINS_TALISMANS,
            Rayon::Sorts => MAGASINS_SORTS,
            Rayon::Materiaux => MAGASINS_MATERIAUX,
            Rayon::Cendres => MAGASINS_CENDRES,
            Rayon::Ameliorations => MAGASINS_AMELIORATIONS,
            Rayon::Esprits => MAGASINS_ESPRITS,
            Rayon::Munitions => MAGASINS_MUNITIONS,
        }
    }

    /// Vrai pour les rayons du sous-menu « Équipement » (on y revient en les quittant).
    pub fn dans_equipement(self) -> bool {
        matches!(self, Rayon::Armes | Rayon::Armures | Rayon::Cendres | Rayon::Munitions)
    }
}

pub const MAGASINS_ARMES: &[Magasin] = &[
    // Corps à corps (343)
    Magasin { texte: 69000041, premiere_ligne: 9600000, derniere_ligne: 9600342 },
    // Boucliers (81)
    Magasin { texte: 69000042, premiere_ligne: 9601000, derniere_ligne: 9601080 },
    // Bâtons et sceaux (32)
    Magasin { texte: 69000043, premiere_ligne: 9602000, derniere_ligne: 9602031 },
    // Arcs et arbalètes (31)
    Magasin { texte: 69000044, premiere_ligne: 9603000, derniere_ligne: 9603030 },
];

pub const MAGASINS_ARMURES: &[Magasin] = &[
    // Têtes (217)
    Magasin { texte: 69000046, premiere_ligne: 9610000, derniere_ligne: 9610216 },
    // Torses (256)
    Magasin { texte: 69000047, premiere_ligne: 9611000, derniere_ligne: 9611255 },
    // Bras (127)
    Magasin { texte: 69000048, premiere_ligne: 9612000, derniere_ligne: 9612126 },
    // Jambes (141)
    Magasin { texte: 69000049, premiere_ligne: 9613000, derniere_ligne: 9613140 },
];

pub const MAGASINS_TALISMANS: &[Magasin] = &[
    // Talismans (155)
    Magasin { texte: 69000050, premiere_ligne: 9620000, derniere_ligne: 9620154 },
];

pub const MAGASINS_SORTS: &[Magasin] = &[
    // Sorcelleries (84)
    Magasin { texte: 69000054, premiere_ligne: 9680000, derniere_ligne: 9680083 },
    // Incantations (129)
    Magasin { texte: 69000055, premiere_ligne: 9681000, derniere_ligne: 9681128 },
];

pub const MAGASINS_MATERIAUX: &[Magasin] = &[
    // Matériaux de confection (106)
    Magasin { texte: 69000051, premiere_ligne: 9630000, derniere_ligne: 9630105 },
];

pub const MAGASINS_CENDRES: &[Magasin] = &[
    // Cendres de guerre (115, EquipParamGem, jeu de base puis DLC)
    Magasin { texte: 69000056, premiere_ligne: 9690000, derniere_ligne: 9690114 },
];

pub const MAGASINS_AMELIORATIONS: &[Magasin] = &[
    // Graine dorée, larme sacrée, fragment de l'Arbre occulte, cendre vénérée (DLC)
    Magasin { texte: 69000057, premiere_ligne: 9700000, derniere_ligne: 9700003 },
];

pub const MAGASINS_ESPRITS: &[Magasin] = &[
    // Cendres d'esprit +0 (84, jeu de base puis DLC)
    Magasin { texte: 69000059, premiere_ligne: 9710000, derniere_ligne: 9710083 },
    // Les mêmes au niveau +10
    Magasin { texte: 69000060, premiere_ligne: 9711000, derniere_ligne: 9711083 },
    // Muguets (20 : tombaux [1] à [9] et grand, fantomatiques [1] à [9] et grand)
    Magasin { texte: 69000061, premiere_ligne: 9712000, derniere_ligne: 9712019 },
];

pub const MAGASINS_MUNITIONS: &[Magasin] = &[
    // Munitions (68 : 63 du jeu de base puis 5 du DLC)
    Magasin { texte: 69000064, premiere_ligne: 9720000, derniere_ligne: 9720067 },
];
