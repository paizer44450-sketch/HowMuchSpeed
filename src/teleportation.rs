//! Téléportation vers une grâce, avec la fonction de voyage rapide du jeu
//! (celle qu'utilise la carte quand on choisit une grâce).
//!
//! Méthode connue de la communauté (Elden Ring Debug Tool, Erd-Tools) :
//! la fonction « Warp » des scripts Lua du jeu reçoit deux objets du
//! gestionnaire Lua (CSLuaEventMan) et le numéro d'entité de la grâce.
//! On retrouve cette fonction dans le jeu par une « signature » d'octets,
//! pour que ça marche même après une mise à jour du jeu.

use std::ffi::c_void;
use std::sync::OnceLock;

use eldenring::cs::{
    BonfireWarpParam, BonfireWarpSubCategoryParam, CSLuaEventManImp, SoloParamRepository,
};
use fromsoftware_shared::FromStatic;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::journal::noter;

/// Onglet « Table ronde » dans le menu des grâces du jeu.
const ONGLET_TABLE_RONDE: u16 = 11100;

/// Signature de la fonction de téléportation. « None » = n'importe quel octet.
/// La fonction commence 2 octets après le début de la signature.
const SIGNATURE_WARP: [Option<u8>; 16] = [
    Some(0xC3), None, None, None, None, None, None, Some(0x57),
    Some(0x48), Some(0x83), Some(0xEC), None, Some(0x48), Some(0x8B), Some(0xFA), Some(0x44),
];
const DECALAGE_DEBUT_FONCTION: usize = 2;

/// Forme de la fonction du jeu : (script Lua, relais Lua, numéro de la grâce).
type FonctionWarp = unsafe extern "C" fn(*mut c_void, *mut c_void, u32);

/// La fonction trouvée, cherchée une seule fois.
static FONCTION_WARP: OnceLock<Option<FonctionWarp>> = OnceLock::new();

/// Téléporte le joueur vers la grâce donnée. Renvoie vrai si la demande est partie.
/// À appeler depuis une tâche du jeu (sur son fil principal), personnage chargé.
pub fn aller_a_la_grace(numero_grace: u32) -> bool {
    let Some(warp) = *FONCTION_WARP.get_or_init(trouver_fonction_warp) else {
        noter("Téléportation impossible : fonction du jeu introuvable.");
        return false;
    };

    // Les deux objets que la fonction attend, pris dans le gestionnaire Lua du jeu.
    let Ok(gestionnaire) = CSLuaEventManImp::instance_ptr() else {
        noter("Téléportation impossible : gestionnaire Lua introuvable.");
        return false;
    };
    let base = gestionnaire as *const u8;
    unsafe {
        let relais = *(base.add(0x8) as *const *mut c_void);
        let script = *(base.add(0x18) as *const *mut c_void);
        if relais.is_null() || script.is_null() {
            noter("Téléportation impossible : gestionnaire Lua pas encore prêt.");
            return false;
        }
        warp(script, relais, numero_grace);
    }
    noter(&format!("Téléportation demandée vers la grâce {numero_grace}."));
    true
}

/// Cherche dans les données du jeu la grâce de la Table ronde.
pub fn grace_de_la_table_ronde() -> Option<u32> {
    let parametres = unsafe { SoloParamRepository::instance() }.ok()?;

    // 1) Les sous-catégories rangées dans l'onglet « Table ronde ».
    let sous_categories: Vec<u32> = parametres
        .rows::<BonfireWarpSubCategoryParam>()
        .filter(|(_, ligne)| ligne.tab_id() == ONGLET_TABLE_RONDE)
        .map(|(id, _)| id)
        .collect();

    // 2) La grâce rangée dans une de ces sous-catégories.
    parametres
        .rows::<BonfireWarpParam>()
        .find(|(_, ligne)| {
            sous_categories.contains(&(ligne.bonfire_sub_category_id() as u32))
                && ligne.bonfire_entity_id() != 0
        })
        .map(|(_, ligne)| ligne.bonfire_entity_id())
}

/// Parcourt le code du jeu (sections exécutables de eldenring.exe) pour trouver la signature.
/// Par sécurité, on n'accepte qu'un seul résultat, qui doit ressembler à un début de fonction.
fn trouver_fonction_warp() -> Option<FonctionWarp> {
    let module = unsafe { GetModuleHandleW(windows::core::PCWSTR::null()) }.ok()?;
    let base = module.0 as *const u8;

    let mut trouvailles = Vec::new();
    for (debut, taille) in unsafe { sections_executables(base) } {
        let code = unsafe { std::slice::from_raw_parts(debut, taille) };
        for position in 0..code.len().saturating_sub(SIGNATURE_WARP.len()) {
            let correspond = SIGNATURE_WARP
                .iter()
                .enumerate()
                .all(|(i, octet)| octet.is_none_or(|attendu| code[position + i] == attendu));
            if correspond {
                trouvailles.push(unsafe { debut.add(position + DECALAGE_DEBUT_FONCTION) });
            }
        }
    }

    if trouvailles.len() != 1 {
        noter(&format!(
            "Signature de téléportation : {} résultat(s) au lieu de 1, téléportation désactivée.",
            trouvailles.len()
        ));
        return None;
    }
    let fonction = trouvailles[0];

    // Début de fonction habituel : « mov [rsp+...], rbx » = 48 89 5C 24.
    let debut_attendu = [0x48, 0x89, 0x5C, 0x24];
    let debut_lu = unsafe { std::slice::from_raw_parts(fonction, 4) };
    if debut_lu != debut_attendu {
        noter(&format!(
            "Signature de téléportation trouvée mais début inattendu {debut_lu:02X?}, téléportation désactivée."
        ));
        return None;
    }

    noter(&format!(
        "Fonction de téléportation trouvée (décalage 0x{:X}).",
        fonction as usize - base as usize
    ));
    Some(unsafe { std::mem::transmute::<*const u8, FonctionWarp>(fonction) })
}

/// Liste (début, taille) des sections de code exécutable du programme, lue dans son en-tête Windows (PE).
unsafe fn sections_executables(base: *const u8) -> Vec<(*const u8, usize)> {
    const SECTION_EXECUTABLE: u32 = 0x2000_0000;
    unsafe {
        let lire_u16 = |p: *const u8| (p as *const u16).read_unaligned() as usize;
        let lire_u32 = |p: *const u8| (p as *const u32).read_unaligned();

        let en_tete = base.add(lire_u32(base.add(0x3C)) as usize);
        let nombre_sections = lire_u16(en_tete.add(6));
        let taille_en_tete_optionnel = lire_u16(en_tete.add(0x14));
        let premiere_section = en_tete.add(0x18 + taille_en_tete_optionnel);

        (0..nombre_sections)
            .map(|i| premiere_section.add(i * 40))
            .filter(|section| lire_u32(section.add(36)) & SECTION_EXECUTABLE != 0)
            .map(|section| {
                let taille = lire_u32(section.add(8)) as usize;
                let adresse = lire_u32(section.add(12)) as usize;
                (base.add(adresse), taille)
            })
            .collect()
    }
}
