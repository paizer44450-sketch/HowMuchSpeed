//! Les marchands de Melina.
//!
//! - Pierres de forge : Melina donne 99 de chaque pierre de forge (normales et sombres),
//!   gratuitement, autant de fois qu'on veut. Le magasin du jeu (OpenRegularShop) et le
//!   forgeron (OpenEnhanceShop) ouverts depuis le menu de Melina restaient vides (essais
//!   0.14.0 à 0.14.2) : on donne donc les pierres comme la larme larvaire de la
//!   renaissance, par un lot d'objets (AwardItemLot). Pour améliorer, il y a Hewg, le
//!   forgeron de la Table ronde.
//!   Le lot utilisé est celui de la récompense du tableau « Instinct de retour » (ligne
//!   80300 d'ItemLotParam_map, sans ligne 80301 derrière : il ne donne que lui). Il est
//!   modifié en mémoire seulement, une pierre à la fois.
//! - Montée de niveau gratuite : pendant que l'écran est ouvert, le joueur a le maximum de
//!   runes ; à la fermeture, on lui remet exactement ses runes d'avant.

use eldenring::cs::{ItemCategory, ItemLotParam_map, SoloParamRepository, WorldChrMan};
use fromsoftware_shared::FromStatic;

use crate::journal::noter;

/// Lot d'objets réutilisé pour donner les pierres, une par une.
pub const LOT_PIERRES: u32 = 80300;

/// Nombre de chaque pierre donnée à chaque fois.
const QUANTITE_PAR_PIERRE: u8 = 99;

/// Les pierres de forge (EquipParamGoods). Normales : jusqu'à +25 ; sombres : jusqu'à +10.
pub const PIERRES_DE_FORGE: [i32; 19] = [
    10100, // Pierre de forge [1]
    10101, // Pierre de forge [2]
    10102, // Pierre de forge [3]
    10103, // Pierre de forge [4]
    10104, // Pierre de forge [5]
    10105, // Pierre de forge [6]
    10106, // Pierre de forge [7]
    10107, // Pierre de forge [8]
    10140, // Pierre de forge de dragon ancien
    10160, // Pierre de forge sombre [1]
    10161, // Pierre de forge sombre [2]
    10162, // Pierre de forge sombre [3]
    10163, // Pierre de forge sombre [4]
    10164, // Pierre de forge sombre [5]
    10165, // Pierre de forge sombre [6]
    10166, // Pierre de forge sombre [7]
    10167, // Pierre de forge sombre [8]
    10200, // Pierre de forge sombre [9]
    10168, // Pierre de forge sombre de dragon ancien
];

/// Catégories d'objets d'un lot (ItemLotParam).
pub const LOT_OBJET: i32 = 1;
pub const LOT_ARME: i32 = 2;

/// Règle le lot pour qu'il donne cette pierre (99 exemplaires, toujours, répétable).
/// Renvoie faux si le lot est introuvable.
pub fn preparer_lot_pierre(pierre: i32) -> bool {
    preparer_lot(pierre, LOT_OBJET, QUANTITE_PAR_PIERRE)
}

/// Règle le lot pour qu'il donne cet objet (catégorie LOT_OBJET ou LOT_ARME), en ce
/// nombre, toujours, répétable. Renvoie faux si le lot est introuvable.
pub fn preparer_lot(objet: i32, categorie: i32, nombre: u8) -> bool {
    let Ok(parametres) = (unsafe { SoloParamRepository::instance_mut() }) else {
        return false;
    };
    let Some(lot) = parametres.get_mut::<ItemLotParam_map>(LOT_PIERRES) else {
        noter("Pierres de forge : lot introuvable.");
        return false;
    };
    lot.set_lot_item_id01(objet);
    lot.set_lot_item_category01(categorie);
    lot.set_lot_item_num01(nombre);
    lot.set_lot_item_base_point01(1000);
    lot.set_get_item_flag_id(0);
    lot.set_get_item_flag_id01(0);
    true
}

/// Runes données pendant la montée de niveau (le maximum du jeu).
const RUNES_MAX: u32 = 999_999_999;

/// Donne le maximum de runes et renvoie les runes que le joueur avait avant.
pub fn donner_les_runes() -> Option<u32> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    let donnees = unsafe { &mut *joueur.player_game_data.as_ptr() };
    let avant = donnees.rune_count;
    donnees.rune_count = RUNES_MAX;
    noter(&format!("Runes mises au maximum (le joueur en avait {avant})."));
    Some(avant)
}

/// Remet au joueur les runes qu'il avait avant.
pub fn rendre_les_runes(avant: u32) {
    let Ok(monde) = (unsafe { WorldChrMan::instance() }) else {
        return;
    };
    let Some(joueur) = monde.main_player.as_ref() else {
        return;
    };
    let donnees = unsafe { &mut *joueur.player_game_data.as_ptr() };
    donnees.rune_count = avant;
    noter(&format!("Runes remises à {avant}."));
}

/// Nombre d'exemplaires de cet objet (EquipParamGoods) dans l'inventaire du joueur, objets
/// clés compris (lus directement dans l'inventaire : la question « le joueur a-t-il
/// l'objet ? » du script de dialogue ne voit pas les objets clés). None si le personnage
/// n'est pas chargé.
pub fn nombre_d_objets(objet: u32) -> Option<u32> {
    let monde = unsafe { WorldChrMan::instance() }.ok()?;
    let joueur = monde.main_player.as_ref()?;
    let donnees = unsafe { &*joueur.player_game_data.as_ptr() };
    let total = donnees
        .equipment
        .equip_inventory_data
        .items_data
        .items()
        .filter(|e| e.item_id.category() == ItemCategory::Goods && e.item_id.param_id() == objet)
        .map(|e| e.quantity)
        .sum();
    Some(total)
}
