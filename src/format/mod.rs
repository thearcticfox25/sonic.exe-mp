//! File formats shared by the game and the GameMaker converter.
//!
//! Both sides must agree byte-for-byte on how a sprite or a map looks on disk.
//! The converter still writes a .toml beside each sprite; the game reads one
//! Textures.toml (textures.rs), made from those once. The halves
//! only the game uses (reading maps and placements back in) are kept here too,
//! so the two copies stay one file.
#![allow(dead_code)]

pub mod apng;
pub mod placement;
pub mod sounds;
pub mod textures;
pub mod tiled;
