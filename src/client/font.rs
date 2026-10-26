//! Bitmap fonts: Fonts/<name>.png with its glyph table. The fonts tools/build_fonts.py
//! lays out have theirs in the [<name>] section of Fonts/Fonts.toml; big, a GameMaker
//! font, has its own Fonts/big.toml, in the layout GameMakerAssetConverter writes
//! GameMaker fonts in (its convert/fonts.rs).

use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub struct Font {
    pub image: crate::format::apng::SpriteImage,
    glyphs: HashMap<char, Glyph>,
}

/// What a character the sheet draws no sign of is written as.
const MISSING_CHARACTER: char = '?';
/// The glyph tables of the fonts tools/build_fonts.py lays out, a section each.
const LAID_OUT_TABLES: &str = "Fonts.toml";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Glyph {
    character: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// How far the next glyph starts.
    pub shift: i32,
    /// How far into its box the glyph is drawn.
    pub offset: i32,
}

#[derive(Deserialize)]
struct FontFile {
    glyph: Vec<Glyph>,
}

impl Font {
    /// `png` is the font's picture; its glyph table sits next to it (big.toml).
    pub fn load(png: &Path) -> Result<Font> {
        let table = png.with_extension("toml");
        let text = crate::core::resources::read_to_string(&table).with_context(|| format!("font table {}", table.display()))?;
        let file: FontFile = toml::from_str(&text).with_context(|| format!("font table {}", table.display()))?;
        Font::with_glyphs(png, file.glyph)
    }

    /// A font tools/build_fonts.py lays out: `<folder>/<name>.png` and the [name]
    /// section of Fonts.toml.
    pub fn load_laid_out(folder: &Path, name: &str) -> Result<Font> {
        let table = folder.join(LAID_OUT_TABLES);
        let text = crate::core::resources::read_to_string(&table).with_context(|| format!("font table {}", table.display()))?;
        let mut files: HashMap<String, FontFile> = toml::from_str(&text).with_context(|| format!("font table {}", table.display()))?;
        let file = files.remove(name).with_context(|| format!("{} has no [{name}]", table.display()))?;
        Font::with_glyphs(&folder.join(format!("{name}.png")), file.glyph)
    }

    fn with_glyphs(png: &Path, glyphs: Vec<Glyph>) -> Result<Font> {
        let image = crate::format::apng::read(png).with_context(|| format!("font {}", png.display()))?;
        Ok(Font { image, glyphs: glyph_map(glyphs) })
    }

    /// The glyph of a character, whichever case it is written in: these sheets are
    /// drawn in one case, and which one it is differs between them.
    pub fn glyph(&self, character: char) -> Option<&Glyph> {
        let other_case = |cased: Option<char>| cased.and_then(|cased| self.glyphs.get(&cased));
        self.glyphs
            .get(&character)
            .or_else(|| other_case(character.to_lowercase().next()))
            .or_else(|| other_case(character.to_uppercase().next()))
    }

    /// The glyph a character is drawn with: its own, or the question mark for one this
    /// font has no sign of. A letter the sheet lacks is then plain to see instead of
    /// leaving a hole in a name or a message.
    pub fn drawn_glyph(&self, character: char) -> Option<&Glyph> {
        self.glyph(character).or_else(|| self.glyphs.get(&MISSING_CHARACTER))
    }

    /// Where each drawable glyph of one line of `text` starts, from 0. A character the
    /// font has no sign of is drawn as the question mark.
    pub fn layout<'a>(&'a self, text: &'a str) -> impl Iterator<Item = (f64, &'a Glyph)> + 'a {
        let mut pen = 0.0;
        text.chars().filter_map(move |character| {
            let glyph = self.drawn_glyph(character)?;
            let start = pen;
            pen += glyph.shift as f64;
            Some((start, glyph))
        })
    }

    /// The tallest glyph: a line's height.
    pub fn line_height(&self) -> f64 {
        self.glyphs.values().map(|glyph| glyph.h).max().unwrap_or(0) as f64
    }

    pub fn width(&self, text: &str) -> f64 {
        text.chars().filter_map(|character| self.drawn_glyph(character)).map(|glyph| glyph.shift as f64).sum()
    }
}

fn glyph_map(glyphs: Vec<Glyph>) -> HashMap<char, Glyph> {
    glyphs.into_iter().filter_map(|glyph| Some((char::from_u32(glyph.character)?, glyph))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laid_out_tables() -> HashMap<String, FontFile> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Fonts").join(LAID_OUT_TABLES);
        toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The fonts build_fonts.py lays out write the same characters, # and * among them,
    /// so a text does not turn into question marks when it moves to another font.
    #[test]
    fn the_laid_out_fonts_have_the_same_characters() {
        let characters = |file: &FontFile| file.glyph.iter().map(|glyph| glyph.character).collect::<std::collections::BTreeSet<u32>>();
        let tables = laid_out_tables();
        let letter = characters(&tables["letter"]);
        assert!(letter.contains(&('#' as u32)) && letter.contains(&('*' as u32)));
        for (name, file) in &tables {
            assert_eq!(characters(file), letter, "{name} and letter");
        }
    }

    /// The format of Fonts/font.scheme: every glyph stands in a box of its own with a
    /// pixel of air around it, so a character that borrows a neighbour's sprite turned
    /// (the arrows) cannot pick up the glyph beside it. Laid out by tools/build_fonts.py.
    #[test]
    fn every_glyph_has_a_box_of_its_own_with_a_pixel_of_air() {
        for (name, file) in laid_out_tables() {
            let drawn: Vec<Glyph> = file.glyph.into_iter().filter(|glyph| glyph.w > 0 && glyph.h > 0).collect();
            for (index, glyph) in drawn.iter().enumerate() {
                for other in &drawn[index + 1..] {
                    // Two characters may share one sprite (a borrowed glyph, turned or not).
                    if (glyph.x, glyph.y) == (other.x, other.y) {
                        continue;
                    }
                    let apart = glyph.x + glyph.w < other.x
                        || other.x + other.w < glyph.x
                        || glyph.y + glyph.h < other.y
                        || other.y + other.h < glyph.y;
                    assert!(apart, "{name}: {glyph:?} and {other:?} are not a pixel apart");
                }
            }
        }
    }

    #[test]
    fn the_menu_font_lays_out_join_game_as_the_button_texture_did() {
        let mut tables = laid_out_tables();
        let glyphs = glyph_map(tables.remove("menu").unwrap().glyph);
        let width = |text: &str| text.chars().map(|character| glyphs[&character.to_ascii_lowercase()].shift).sum::<i32>();
        // spr_menu_buttons frame 0 is 69 px of ink, the last glyph's spacing column not included.
        assert_eq!(width("JOIN GAME"), 70);
        assert_eq!(width("a"), 8);
    }
}
