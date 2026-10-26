//! obj_lobby_prolet: the slowly scrolling pictures of a map behind the vote, the
//! round intro and the results.

use crate::client::canvas::Canvas;
use macroquad::rand;
use crate::core::resources::names::sprite::*;
use crate::core::resources::SpriteId;
use crate::core::world::C_WHITE;
use crate::core::config::step;

/// mapData: the pictures of every map, by level number. `vertical` pictures scroll up instead of left.
struct MapPictures {
    tiles: &'static [SpriteId],
    background: SpriteId,
    vertical: &'static [bool],
}

const MAP_PICTURES: [MapPictures; 22] = [
    // The alpha had no pictures for level1; these are cut from its map, over its own background.
    MapPictures { tiles: &[SPR_PREVIEW_LEVEL1_1, SPR_PREVIEW_LEVEL1_2, SPR_PREVIEW_LEVEL1_3], background: SPR_BACKGROUND_TEST, vertical: &[false, false, true] },
    MapPictures { tiles: &[SPR_PREVIEW_HIDE_AND_SEEK_2_1, SPR_PREVIEW_HIDE_AND_SEEK_2_2, SPR_PREVIEW_HIDE_AND_SEEK_2_3], background: SPR_PREVIEW_HIDE_AND_SEEK_2_BACKGROUND, vertical: &[true, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_RAVINE_MIST_1, SPR_PREVIEW_RAVINE_MIST_2], background: SPR_PREVIEW_RAVINE_MIST_BACKGROUND, vertical: &[false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_DOTDOTDOT_1, SPR_PREVIEW_DOTDOTDOT_2, SPR_PREVIEW_DOTDOTDOT_3], background: BACKGROUND_DOTDOTDOT, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_DESERT_TOWN_1, SPR_PREVIEW_DESERT_TOWN_2, SPR_PREVIEW_DESERT_TOWN_3], background: SPR_PREVIEW_DESERT_TOWN_BACKGROUND, vertical: &[false, true, false] },
    MapPictures { tiles: &[SPR_PREVIEW_YOU_CANT_RUN_1, SPR_PREVIEW_YOU_CANT_RUN_2, SPR_PREVIEW_YOU_CANT_RUN_3], background: SPR_PREVIEW_YOU_CANT_RUN_BACKGROUND, vertical: &[false, false, true] },
    MapPictures { tiles: &[SPR_PREVIEW_LIMP_CITY_1, SPR_PREVIEW_LIMP_CITY_2, SPR_PREVIEW_LIMP_CITY_3], background: SPR_PREVIEW_LIMP_CITY_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_NOT_PERFECT_1, SPR_PREVIEW_NOT_PERFECT_2, SPR_PREVIEW_NOT_PERFECT_3, SPR_PREVIEW_NOT_PERFECT_4], background: SPR_PREVIEW_NOT_PERFECT_BACKGROUND, vertical: &[false, true, false, true] },
    MapPictures { tiles: &[SPR_PREVIEW_KIND_AND_FAIR_1, SPR_PREVIEW_KIND_AND_FAIR_2, SPR_PREVIEW_KIND_AND_FAIR_3], background: SPR_PREVIEW_KIND_AND_FAIR_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_ACT9_1, SPR_PREVIEW_ACT9_2, SPR_PREVIEW_ACT9_3], background: BACKGROUND_ACT9, vertical: &[false, true, false] },
    MapPictures { tiles: &[SPR_PREVIEW_NASTY_PARADISE_1, SPR_PREVIEW_NASTY_PARADISE_2, SPR_PREVIEW_NASTY_PARADISE_3], background: SPR_PREVIEW_NASTY_PARADISE_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_PRICELESS_FREEDOM_1, SPR_PREVIEW_PRICELESS_FREEDOM_2, SPR_PREVIEW_PRICELESS_FREEDOM_3], background: BACKGROUND_PF, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_VOLCANO_VALLEY_1, SPR_PREVIEW_VOLCANO_VALLEY_2, SPR_PREVIEW_VOLCANO_VALLEY_3], background: SPR_PREVIEW_VOLCANO_VALLEY_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_GREEN_HILL_1, SPR_PREVIEW_GREEN_HILL_2, SPR_PREVIEW_GREEN_HILL_3], background: SPR_PREVIEW_GREEN_HILL_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_MAJIN_FOREST_1, SPR_PREVIEW_MAJIN_FOREST_2, SPR_PREVIEW_MAJIN_FOREST_3], background: SPR_PREVIEW_MAJIN_FOREST_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_ANGEL_ISLAND_1, SPR_PREVIEW_ANGEL_ISLAND_2, SPR_PREVIEW_ANGEL_ISLAND_3], background: SPR_PREVIEW_ANGEL_ISLAND_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_TORTURE_CAVE_1, SPR_PREVIEW_TORTURE_CAVE_2, SPR_PREVIEW_TORTURE_CAVE_3], background: SPR_PREVIEW_TORTURE_CAVE_BACKGROUND, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_DARK_TOWER_1, SPR_PREVIEW_DARK_TOWER_2, SPR_PREVIEW_DARK_TOWER_3], background: BACKGROUND_DARKTOWER, vertical: &[false, false, false] },
    MapPictures { tiles: &[SPR_PREVIEW_HAUNTING_DREAM_1, SPR_PREVIEW_HAUNTING_DREAM_2, SPR_PREVIEW_HAUNTING_DREAM_3], background: SPR_PREVIEW_HAUNTING_DREAM_BACKGROUND, vertical: &[false, false, false] },
    // The original had no pictures for Fart Zone, only bg_test behind them (the main menu's).
    MapPictures { tiles: &[SPR_PREVIEW_FART_ZONE_1, SPR_PREVIEW_FART_ZONE_2, SPR_PREVIEW_FART_ZONE_3], background: SPR_PREVIEW_FART_ZONE_BACKGROUND, vertical: &[false, true, false] },
    MapPictures { tiles: &[SPR_PREVIEW_WEED_ZONE_1, SPR_PREVIEW_WEED_ZONE_2, SPR_PREVIEW_WEED_ZONE_3], background: SPR_PREVIEW_WEED_ZONE_BACKGROUND, vertical: &[false, false, true] },
    MapPictures { tiles: &[SPR_PREVIEW_MARIJUNA_1, SPR_PREVIEW_MARIJUNA_2, SPR_PREVIEW_MARIJUNA_3], background: SPR_PREVIEW_MARIJUNA_BACKGROUND, vertical: &[false, false, false] },
];
/// Kinda Fair scrolls slower.
const SLOW_SCROLL_MAP: usize = 8;
const SCROLL_PER_STEP: f64 = 0.64 + 0.16;
const SLOW_MAP_SCROLL_REDUCTION: f64 = 0.38;
/// Fading in over the first pixels of a picture and out before its end.
const FADE_IN_UNTIL: f64 = 56.0;
const FADE_DISTANCE: f64 = 32.0;
const FADE_OUT_MARGIN: f64 = 48.0;
const VIEW_WIDTH: f64 = 480.0;
const VIEW_HEIGHT: f64 = 270.0;
/// The background moves ten times slower than the picture and is less opaque.
const BACKGROUND_PARALLAX: f64 = 10.0;
const BACKGROUND_ALPHA_BELOW_PICTURE: f64 = 0.5;
const BACKGROUND_FRAME_MILLISECONDS: f64 = 100.0;
/// The map shown before the server names one.
const FIRST_MAP: usize = 7;

pub struct MapPreview {
    pub visible: bool,
    pub blend: u32,
    /// blackwhite: drawn without colors (the results of a round EXE won).
    pub black_white: bool,
    alpha: f64,
    map: usize,
    picture: usize,
    scroll: f64,
    vertical: bool,
}

impl MapPreview {
    pub fn new() -> MapPreview {
        // The room's creation code hides it until the vote.
        MapPreview { visible: false, blend: C_WHITE, black_white: false, alpha: 1.0, map: FIRST_MAP, picture: 0, scroll: 0.0, vertical: false }
    }

    pub fn map(&self) -> usize {
        self.map
    }

    /// setZone: a random picture of `map` from its start.
    pub fn set_zone(&mut self, map: usize) {
        let Some(pictures) = MAP_PICTURES.get(map) else { return };
        self.picture = rand::gen_range(0, pictures.tiles.len());
        self.scroll = 0.0;
        self.map = map;
        self.vertical = pictures.vertical[self.picture];
    }

    /// Draw_0: scrolls, fades and draws.
    pub fn draw(&mut self, canvas: &mut Canvas, current_time_ms: f64) {
        if !self.visible {
            return;
        }
        if self.black_white {
            canvas.set_black_white();
        }
        let pictures = &MAP_PICTURES[self.map];
        let tile = pictures.tiles[self.picture];
        let tile_meta = canvas.sprites.get(tile);
        let length = if self.vertical { tile_meta.height } else { tile_meta.width } as f64;
        self.scroll_one_step(length, pictures);

        let background_meta = canvas.sprites.get(pictures.background);
        let (background_width, background_height) = (background_meta.width as f64, background_meta.height as f64);
        let background_frame = current_time_ms / BACKGROUND_FRAME_MILLISECONDS;
        let background_offset = -self.scroll / BACKGROUND_PARALLAX;
        let mut column = 0.0;
        while column < VIEW_WIDTH / background_width + 2.0 {
            let mut row = 0.0;
            while row < VIEW_HEIGHT / background_height + 2.0 {
                let (x, y) = if self.vertical {
                    (column * background_width, background_offset + row * background_height)
                } else {
                    (background_offset + column * background_width, row * background_height)
                };
                canvas.draw_sprite_ext(pictures.background, background_frame, x, y, 1.0, 1.0, 0.0, self.blend, self.alpha - BACKGROUND_ALPHA_BELOW_PICTURE);
                row += 1.0;
            }
            column += 1.0;
        }

        let (x, y) = if self.vertical { (0.0, -self.scroll) } else { (-self.scroll, 0.0) };
        canvas.draw_sprite_ext(tile, 0.0, x, y, 1.0, 1.0, 0.0, self.blend, self.alpha);
        if self.black_white {
            canvas.reset_shader();
        }
    }

    fn scroll_one_step(&mut self, length: f64, pictures: &MapPictures) {
        self.scroll += SCROLL_PER_STEP * step();
        if self.map == SLOW_SCROLL_MAP {
            self.scroll -= SLOW_MAP_SCROLL_REDUCTION * step();
        }
        if self.scroll <= FADE_IN_UNTIL {
            self.alpha = self.scroll / FADE_DISTANCE;
        }
        let view_length = if self.vertical { VIEW_HEIGHT } else { VIEW_WIDTH };
        let fade_out_start = length - (view_length + FADE_OUT_MARGIN);
        if self.scroll >= fade_out_start {
            self.alpha = 1.5 - (self.scroll - fade_out_start) / FADE_DISTANCE;
        }
        if self.scroll > length - view_length {
            self.scroll = 0.0;
            self.picture = (self.picture + 1) % pictures.tiles.len();
            self.vertical = pictures.vertical[self.picture];
        }
    }
}
