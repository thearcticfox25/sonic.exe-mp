//! Textures/Textures.toml: every sprite of the game, one line each, by the name the
//! code knows it by, as Sounds/Sounds.toml lists the sounds:
//!
//!     bigring = { file = "Objects/bigring.apng", origin = [32, 64], bbox_mode = "manual", bbox = [20, 11, 43, 53] }
//!
//! `file` is relative to Textures/, so a texture pack replaces a file or edits its
//! line. Everything but `file` may be left out when it is the default below. The
//! name is also the file's name: maps find a sprite by the file their tileset shows.

use serde::Deserialize;
use std::collections::BTreeMap;

pub const FILE_NAME: &str = "Textures.toml";

/// How collisions are checked against a sprite, as in the GameMaker sprite editor.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum CollisionKind {
    /// Pixel mask combined from all frames.
    Precise,
    #[default]
    Rectangle,
    Ellipse,
    Diamond,
    /// Separate pixel mask for every frame.
    PrecisePerFrame,
    RotatedRectangle,
}

/// Where the bounding box comes from, as in the GameMaker sprite editor.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum BboxMode {
    /// Computed from non-transparent pixels, so it follows texture pack edits.
    #[default]
    Automatic,
    FullImage,
    /// Uses `bbox`.
    Manual,
}

/// An animation stored once and played back and forth: its file holds the frames from
/// the first to the turn, and they are played again backwards to the second.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PingPong {
    /// a b c d -> a b c d c b
    TurnAtLast,
    /// a b c d -> a b c d d c b: the last frame shows twice in a row.
    RepeatLast,
    /// a b c d -> a b c d c b a: the first frame shows twice in a row, as the loop
    /// comes back to it.
    RepeatFirst,
}

impl PingPong {
    /// The whole animation, back and forth, from the frames its file holds.
    pub fn play_back<T: Clone>(self, frames: &mut Vec<T>) {
        // The frames played backwards: from the one before the turn (or the turn itself)
        // down to the second (or the first).
        let last = frames.len().saturating_sub(1);
        let (down_to, back_from) = match self {
            PingPong::TurnAtLast => (1, last),
            PingPong::RepeatLast => (1, frames.len()),
            PingPong::RepeatFirst => (0, last),
        };
        let back: Vec<T> = frames[down_to.min(back_from)..back_from].iter().rev().cloned().collect();
        frames.extend(back);
    }
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct SpriteEntry {
    pub file: String,
    pub origin: [i32; 2],
    pub collision: CollisionKind,
    pub bbox_mode: BboxMode,
    /// left, top, right, bottom (inclusive). Only read when bbox_mode = "manual".
    pub bbox: Option<[i32; 4]>,
    /// Alpha at or below this value counts as empty space for automatic bbox and precise masks.
    pub alpha_tolerance: u8,
    /// Overrides the playback speed stored in the APNG. Needed for animated
    /// sprites with speed 0, whose frame is chosen by code through image_index.
    pub fps: Option<u16>,
    pub ping_pong: Option<PingPong>,
}

pub type TextureList = BTreeMap<String, SpriteEntry>;

pub fn parse(text: &str) -> Result<TextureList, toml::de::Error> {
    toml::from_str(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_says_only_what_differs_from_the_defaults() {
        let list = parse("ring = { file = \"Objects/ring.apng\" }\nbigring = { file = \"Objects/bigring.apng\", origin = [32, 64], bbox_mode = \"manual\", bbox = [20, 11, 43, 53] }\n").unwrap();
        assert_eq!(list["ring"], SpriteEntry { file: "Objects/ring.apng".into(), ..SpriteEntry::default() });
        assert_eq!(list["bigring"].origin, [32, 64]);
        assert_eq!(list["bigring"].bbox, Some([20, 11, 43, 53]));
    }

    #[test]
    fn a_ping_pong_animation_plays_back_to_its_second_frame() {
        let mut turning = vec!['a', 'b', 'c', 'd'];
        PingPong::TurnAtLast.play_back(&mut turning);
        assert_eq!(turning, ['a', 'b', 'c', 'd', 'c', 'b']);
        let mut repeating = vec!['a', 'b', 'c'];
        PingPong::RepeatLast.play_back(&mut repeating);
        assert_eq!(repeating, ['a', 'b', 'c', 'c', 'b']);
        let mut back_to_first = vec!['a', 'b', 'c', 'd'];
        PingPong::RepeatFirst.play_back(&mut back_to_first);
        assert_eq!(back_to_first, ['a', 'b', 'c', 'd', 'c', 'b', 'a']);
    }
}
