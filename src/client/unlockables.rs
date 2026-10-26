//! PersistentStorage/Game/Palettes.toml: the chosen skin of every character look (under
//! the character's name, alive or demonized), the chosen pet (0 for none) with the
//! colors of each pet by name, and the lobby icon (obj_unlockables).

use crate::client::palette::{self, Colors};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use crate::core::palettes::{MAX_SKIN_PACKET_COLORS, PALETTE_DEMON, PALETTE_EXE, PALETTE_TABLES};
use crate::core::player::{Character, ExeCharacter};

const PALETTES_FILE: &str = "PersistentStorage/Game/Palettes.toml";
/// The chosen pet when no pet follows the player; pet n is the n-th of the menu's list.
pub const NO_PET: u8 = 0;

/// Where pet number `pet` stands in the pet list, if it is a pet.
pub fn pet_list_index(pet: u8) -> Option<usize> {
    (pet as usize).checked_sub(1)
}

#[derive(Clone, Debug)]
pub struct Unlockables {
    /// Frame of spr_lobby_dicon shown for the player in the lobby.
    pub lobby_icon: u8,
    /// The pet's number (from 1), or NO_PET.
    pub pet: u8,
    /// Colors of the recolorable pets, by pet index.
    pub pet_palettes: Vec<PetPalette>,
    /// One entry per palette table (crate::core::palettes).
    pub skins: Vec<ChosenSkin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PetPalette {
    pub from: Colors,
    pub to: Colors,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChosenSkin {
    /// Character number, plus PALETTE_DEMON or PALETTE_EXE.
    pub table: usize,
    /// The skin's name in the menu ("default", "classic", ...).
    pub name: String,
    /// The custom skin was picked: `to` holds its shifted colors.
    pub custom: bool,
    pub from: Colors,
    pub to: Colors,
}

/// The characters as the file names them (the names --character takes): survivors,
/// whose skin has an alive and a demonized look, and the EXE, which has one.
const SURVIVORS: [(Character, &str); 6] = [
    (Character::Tails, "tails"),
    (Character::Knux, "knuckles"),
    (Character::Eggman, "eggman"),
    (Character::Amy, "amy"),
    (Character::Cream, "cream"),
    (Character::Sally, "sally"),
];
const EXES: [(ExeCharacter, &str); 4] = [
    (ExeCharacter::Original, "exe"),
    (ExeCharacter::Chaos, "chaos"),
    (ExeCharacter::Exetior, "exetior"),
    (ExeCharacter::Exeller, "exeller"),
];

/// How Palettes.toml is written: the pets and the characters by name (`[pet.flicky]`,
/// `[character.tails.alive]`), the chosen pet counted from 1 (0 for none), instead of
/// the original's numbers for all of them.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PalettesFile {
    lobby_icon: u8,
    pet: PetsFile,
    character: BTreeMap<String, CharacterSkins>,
}

/// `[pet]` holds the chosen pet, and under it each recolorable pet's colors.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PetsFile {
    chosen: u8,
    #[serde(flatten)]
    palettes: BTreeMap<String, PetPalette>,
}

/// `[character.tails.alive]` and `[character.tails.demonized]`, or `[character.exe]`.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CharacterSkins {
    Survivor { alive: SkinFile, demonized: SkinFile },
    Exe(SkinFile),
}

#[derive(Serialize, Deserialize)]
struct SkinFile {
    name: String,
    custom: bool,
    from: Colors,
    to: Colors,
}

impl SkinFile {
    fn of(unlockables: &Unlockables, table: usize) -> SkinFile {
        let skin = unlockables.skin(table);
        SkinFile { name: skin.name.clone(), custom: skin.custom, from: skin.from.clone(), to: skin.to.clone() }
    }

    fn chosen(self, table: usize) -> ChosenSkin {
        let (from, to) = capped(self.from, self.to, &format!("palette table {table}"));
        ChosenSkin { table, name: self.name, custom: self.custom, from, to }
    }
}

/// A skin or a pet's colors as the file has them, but at most MAX_SKIN_PACKET_COLORS of them.
fn capped(mut from: Colors, mut to: Colors, whose: &str) -> (Colors, Colors) {
    if from.len().max(to.len()) > MAX_SKIN_PACKET_COLORS {
        eprintln!("{PALETTES_FILE}: {whose} has more than {MAX_SKIN_PACKET_COLORS} colors, only the first {MAX_SKIN_PACKET_COLORS} are used");
        from.truncate(MAX_SKIN_PACKET_COLORS);
        to.truncate(MAX_SKIN_PACKET_COLORS);
    }
    (from, to)
}

impl From<&Unlockables> for PalettesFile {
    fn from(unlockables: &Unlockables) -> PalettesFile {
        let survivors = SURVIVORS.iter().map(|&(character, name)| {
            let table = character as usize;
            let skins = CharacterSkins::Survivor { alive: SkinFile::of(unlockables, table), demonized: SkinFile::of(unlockables, table + PALETTE_DEMON) };
            (name.to_string(), skins)
        });
        let exes = EXES.iter().map(|&(exe, name)| (name.to_string(), CharacterSkins::Exe(SkinFile::of(unlockables, exe as usize + PALETTE_EXE))));
        let chosen = unlockables.pet;
        let palettes = PET_COLORS.iter().zip(&unlockables.pet_palettes).map(|((name, _), colors)| (name.to_string(), colors.clone())).collect();
        PalettesFile { lobby_icon: unlockables.lobby_icon, pet: PetsFile { chosen, palettes }, character: survivors.chain(exes).collect() }
    }
}

impl From<PalettesFile> for Unlockables {
    /// A character the file names wrongly is told about and left out (it gets the
    /// default skins again).
    fn from(file: PalettesFile) -> Unlockables {
        let mut skins = Vec::new();
        for (name, character_skins) in file.character {
            let survivor = SURVIVORS.iter().find(|(_, known)| *known == name).map(|&(character, _)| character as usize);
            let exe = EXES.iter().find(|(_, known)| *known == name).map(|&(exe, _)| exe as usize + PALETTE_EXE);
            match (character_skins, survivor, exe) {
                (CharacterSkins::Survivor { alive, demonized }, Some(table), _) => {
                    skins.push(alive.chosen(table));
                    skins.push(demonized.chosen(table + PALETTE_DEMON));
                }
                (CharacterSkins::Exe(skin), _, Some(table)) => skins.push(skin.chosen(table)),
                _ => eprintln!("{PALETTES_FILE}: no character {name:?} has skins like these, they are left out"),
            }
        }
        let mut saved_pets = file.pet.palettes;
        let pet_palettes = PET_COLORS
            .iter()
            .map(|&(name, colors)| match saved_pets.remove(name) {
                Some(saved) => {
                    let (from, to) = capped(saved.from, saved.to, name);
                    PetPalette { from, to }
                }
                None => PetPalette::of(colors),
            })
            .collect();
        for name in saved_pets.keys() {
            eprintln!("{PALETTES_FILE}: no pet {name:?} can be recolored, its colors are left out");
        }
        Unlockables { lobby_icon: file.lobby_icon, pet: file.pet.chosen, pet_palettes, skins }
    }
}

/// The file with a comment above each pet and skin saying what the code calls it:
/// the pet's number and the palette table (crate::core::palettes).
fn with_ids(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if let Some(id) = line.strip_prefix('[').and_then(|header| header.strip_suffix(']')).and_then(id_of_header) {
            out.push_str(&format!("# {id}\n"));
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// "pet.flicky" -> "pet 1", "character.tails.demonized" -> "palette table 21".
fn id_of_header(header: &str) -> Option<String> {
    let mut parts = header.split('.');
    match (parts.next()?, parts.next(), parts.next()) {
        ("pet", Some(name), None) => PET_COLORS.iter().position(|(pet, _)| *pet == name).map(|index| format!("pet {}", index + 1)),
        ("character", Some(name), look) => {
            let survivor = SURVIVORS.iter().find(|(_, known)| *known == name).map(|&(character, _)| character as usize);
            let exe = EXES.iter().find(|(_, known)| *known == name).map(|&(exe, _)| exe as usize + PALETTE_EXE);
            let table = match look {
                Some("alive") => survivor?,
                Some("demonized") => survivor? + PALETTE_DEMON,
                None => exe?,
                Some(_) => return None,
            };
            Some(format!("palette table {table}"))
        }
        _ => None,
    }
}

/// global.petPallettes for the pets that can be recolored, the first seven of the
/// pet list, under the names the file gives them.
const PET_COLORS: [(&str, &[u32]); 7] = [
    ("flicky", &[0x8484e7, 0x4242a5, 0x212184]),
    ("chao", &[0xaddef8, 0x2ebee9, 0x0e6d89, 0xecde2f, 0xdcb936, 0x985000]),
    ("metal", &[0xe00000, 0x800000]),
    ("t_doll", &[0xe00000, 0x800000]),
    ("majin", &[0x000022, 0x000090, 0x482490, 0x6c48b4]),
    ("m_knux", &[0xff0000, 0x700000, 0xba0000]),
    ("eggor", &[0xfc0000, 0x900000]),
];

impl PetPalette {
    /// A pet's own colors, nothing changed.
    fn of(colors: &[u32]) -> PetPalette {
        PetPalette { from: palette::from_rgb(colors), to: palette::from_rgb(colors) }
    }
}

impl Default for Unlockables {
    fn default() -> Unlockables {
        Unlockables {
            lobby_icon: 0,
            pet: NO_PET,
            pet_palettes: PET_COLORS.iter().map(|(_, colors)| PetPalette::of(colors)).collect(),
            skins: PALETTE_TABLES
                .iter()
                .map(|table| ChosenSkin {
                    table: table.table,
                    name: "default".to_string(),
                    custom: false,
                    from: palette::default_colors(),
                    to: palette::default_colors(),
                })
                .collect(),
        }
    }
}

impl Unlockables {
    /// The saved choices, or the defaults (written to disk) when there are none.
    /// A broken file is reported and left alone until the next change saves over it.
    pub fn load() -> Unlockables {
        let Ok(text) = std::fs::read_to_string(PALETTES_FILE) else {
            let unlockables = Unlockables::default();
            unlockables.save();
            return unlockables;
        };
        match toml::from_str::<PalettesFile>(&text).map(Unlockables::from) {
            Ok(mut unlockables) => {
                unlockables.add_missing_entries();
                unlockables
            }
            Err(error) => {
                eprintln!("{PALETTES_FILE} is broken, using the defaults: {error}");
                Unlockables::default()
            }
        }
    }

    pub fn save(&self) {
        let written = Path::new(PALETTES_FILE)
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(PALETTES_FILE, with_ids(&toml::to_string(&PalettesFile::from(self)).expect("palettes always serialize"))));
        if let Err(error) = written {
            eprintln!("failed to save {PALETTES_FILE}: {error}");
        }
    }

    pub fn skin_mut(&mut self, table: usize) -> &mut ChosenSkin {
        let index = self.skins.iter().position(|skin| skin.table == table).expect("every palette table has a chosen skin");
        &mut self.skins[index]
    }

    pub fn skin(&self, table: usize) -> &ChosenSkin {
        self.skins.iter().find(|skin| skin.table == table).expect("every palette table has a chosen skin")
    }

    /// A file written before a table existed gets the defaults for it. (Pets missing
    /// from the file get their own colors as it is read.)
    fn add_missing_entries(&mut self) {
        for skin in Unlockables::default().skins {
            if !self.skins.iter().any(|existing| existing.table == skin.table) {
                self.skins.push(skin);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_survive_a_save_and_load() {
        let mut unlockables = Unlockables::default();
        unlockables.pet = 4;
        unlockables.skin_mut(41).custom = true;
        let text = toml::to_string(&PalettesFile::from(&unlockables)).unwrap();
        assert!(text.contains("chosen = 4") && text.contains("[pet.flicky]") && text.contains("[character.chaos]") && text.contains("[character.tails.demonized]"), "{text}");
        let loaded = Unlockables::from(toml::from_str::<PalettesFile>(&text).unwrap());
        assert_eq!(loaded.pet, 4);
        assert!(loaded.skin(41).custom);
        assert_eq!(loaded.pet_palettes, unlockables.pet_palettes);
    }

    #[test]
    fn no_pet_is_0_and_every_palette_table_has_a_name() {
        let unlockables = Unlockables::default();
        assert_eq!(PalettesFile::from(&unlockables).pet.chosen, 0);
        let named = Unlockables::from(PalettesFile::from(&unlockables));
        for table in PALETTE_TABLES.iter() {
            assert!(named.skins.iter().any(|skin| skin.table == table.table), "table {} has no name in the file", table.table);
        }
    }

    #[test]
    fn each_pet_and_skin_says_its_number_in_the_code() {
        let text = with_ids("[pet]\nchosen = 0\n\n[pet.flicky]\n\n[character.tails.demonized]\n\n[character.chaos]\n");
        assert!(text.contains("# pet 1\n[pet.flicky]"), "{text}");
        assert!(text.contains("# palette table 21\n[character.tails.demonized]"), "{text}");
        assert!(text.contains("# palette table 41\n[character.chaos]"), "{text}");
        assert!(text.starts_with("[pet]"), "the [pet] section is not one pet: {text}");
    }
}
