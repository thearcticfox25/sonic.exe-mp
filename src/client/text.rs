//! scr_text_spr: text drawn with the chat lettering (Fonts/letter.png, the original's
//! spr_letter1). Some characters are not drawn but switch the color of the letters
//! after them, so the text is laid out letter by letter here instead of by the font.
//! Where a player types (the chat box, the menu's text fields), each place says when
//! it is made whether the markers color what is typed or are drawn as themselves.

use crate::client::canvas::{make_color_rgb, Canvas, TextFont};
use crate::client::font::{Font, Glyph};
use crate::core::resources::names::sprite;
use crate::core::world::C_WHITE;

const LINE_HEIGHT: f64 = 8.0;
/// Draws `text` with its top-left corner at (x, y) and returns the width of its
/// widest line. A `color` other than white overrides the color codes.
pub fn draw_text(canvas: &mut Canvas, x: f64, y: f64, text: &str, color: u32, alpha: f64) -> f64 {
    let (letters, width) = layout(canvas.font(TextFont::Chat), x, y, text, color, Markers::Color);
    draw_letters(canvas, letters, alpha);
    width
}

/// Draws typed text with its top-left corner at (x, y): `colored`, the color markers
/// color the letters after them as in `draw_text`; otherwise they show as themselves.
pub fn draw_typed_text(canvas: &mut Canvas, x: f64, y: f64, text: &str, color: u32, colored: bool) {
    let markers = if colored { Markers::Color } else { Markers::Show };
    let (letters, _) = layout(canvas.font(TextFont::Chat), x, y, text, color, markers);
    draw_letters(canvas, letters, 1.0);
}

fn draw_letters(canvas: &mut Canvas, letters: Vec<Letter>, alpha: f64) {
    for letter in letters {
        canvas.draw_font_glyph(TextFont::Chat, &letter.glyph, letter.x, letter.y, letter.color, alpha);
    }
}

/// The width `draw_text` would return (the original measured by drawing off screen).
pub fn text_width(canvas: &Canvas, text: &str) -> f64 {
    layout(canvas.font(TextFont::Chat), 0.0, 0.0, text, C_WHITE, Markers::Color).1
}

/// scr_counter_draw: one spr_counter digit frame.
pub fn draw_counter(canvas: &mut Canvas, number: i32, x: f64, y: f64) {
    canvas.draw_sprite(sprite::SPR_COUNTER, number as f64, x, y);
}

/// scr_counter_draw_m: the digits of `number`, padded with leading zeros to `width` digits.
pub fn draw_counter_padded(canvas: &mut Canvas, number: i32, x: f64, y: f64, width: usize) {
    draw_counter_digits(canvas, &format!("{number:0width$}"), x, y, 1.0, C_WHITE);
}

/// spr_counter3 and spr_countdown, which had a frame for every whole number: the
/// digits of `number`, centered on (x, y) by what they show, at `scale`.
pub fn draw_counter_centered(canvas: &mut Canvas, number: i32, x: f64, y: f64, scale: f64, color: u32) {
    let digits = number.to_string();
    // The last column of a digit's frame is the gap before the next one.
    let ink_width = COUNTER_DIGIT_WIDTH * digits.len() as f64 - 1.0;
    let (left, top) = (x - (ink_width / 2.0).floor() * scale, y - COUNTER_MIDDLE_Y * scale);
    draw_counter_digits(canvas, &digits, left, top, scale, color);
}

/// spr_counter digits from the top-left corner (x, y), one every frame width.
fn draw_counter_digits(canvas: &mut Canvas, digits: &str, x: f64, y: f64, scale: f64, color: u32) {
    for (index, digit) in digits.chars().enumerate() {
        let frame = digit.to_digit(10).unwrap_or(0) as f64;
        let left = x + COUNTER_DIGIT_WIDTH * index as f64 * scale;
        canvas.draw_sprite_ext(sprite::SPR_COUNTER, frame, left, y, scale, scale, 0.0, color, 1.0);
    }
}

const COUNTER_DIGIT_WIDTH: f64 = 11.0;
/// Half the height of a spr_counter digit, where spr_counter3 and spr_countdown had their origin.
const COUNTER_MIDDLE_Y: f64 = 7.0;

/// What the color markers of crate::colors do in a text.
#[derive(Clone, Copy, PartialEq)]
enum Markers {
    /// Switch the color of the letters after them, drawn as nothing.
    Color,
    /// Drawn as the characters they are.
    Show,
}

struct Letter {
    glyph: Glyph,
    x: f64,
    y: f64,
    color: u32,
}

fn layout(font: &Font, x: f64, y: f64, text: &str, color: u32, markers: Markers) -> (Vec<Letter>, f64) {
    let mut letters = Vec::new();
    let (mut pen_x, mut pen_y) = (x, y);
    let mut code_color = color;
    let mut widest_line = 0.0;

    for character in text.chars() {
        if character == '\n' {
            pen_x = x;
            pen_y += LINE_HEIGHT;
            continue;
        }
        // Spaces and color codes do not count towards the width.
        if character == ' ' {
            pen_x += font.glyph(' ').map_or(0.0, |space| space.shift as f64);
            continue;
        }
        if let Some(switched) = color_code(character, color).filter(|_| markers == Markers::Color) {
            code_color = switched;
            continue;
        }

        // A letter the lettering has no sign of is written as its question mark (font.rs).
        let Some(glyph) = font.drawn_glyph(character).copied() else { continue };
        let letter_color = if color != C_WHITE { color } else { code_color };
        letters.push(Letter { glyph, x: pen_x, y: pen_y, color: letter_color });
        pen_x += glyph.shift as f64;
        widest_line = f64::max(widest_line, pen_x - x);
    }
    (letters, widest_line)
}

/// `~` goes back to the color the caller asked for; the others are in crate::colors.
fn color_code(character: char, caller_color: u32) -> Option<u32> {
    if crate::colors::is_reset(character) {
        return Some(caller_color);
    }
    crate::colors::marker_color(character).map(|(red, green, blue)| make_color_rgb(red, green, blue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn lettering() -> Font {
        Font::load_laid_out(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Fonts"), "letter").unwrap()
    }

    /// The widths the sprite version had: 6 px a letter, 8 for the wide ones, 5 for a space.
    #[test]
    fn the_lettering_keeps_the_widths_the_frames_were_drawn_with() {
        let font = lettering();
        assert_eq!(font.glyph('a').unwrap().shift, 6);
        assert_eq!(font.glyph('m').unwrap().shift, 8);
        assert_eq!(font.glyph('ж').unwrap().shift, 8);
        assert_eq!(font.glyph(' ').unwrap().shift, 5);
        // Upper case is the same lettering, and the version's alpha is in there now.
        assert_eq!(font.glyph('A').map(|glyph| glyph.x), font.glyph('a').map(|glyph| glyph.x));
        assert!(font.glyph('α').is_some());
    }

    /// A letter the sheet draws no sign of is written as the question mark, and a
    /// letter it does draw is written as itself whichever case it is typed in.
    #[test]
    fn a_letter_the_lettering_lacks_is_written_as_a_question_mark() {
        let font = lettering();
        assert!(font.glyph('\u{4e2d}').is_none(), "the sheet has no chinese");
        let (letters, _) = layout(&font, 0.0, 0.0, "\u{4e2d}", C_WHITE, Markers::Color);
        assert_eq!(letters[0].glyph.x, font.glyph('?').unwrap().x, "written as a question mark");

        let (upper, _) = layout(&font, 0.0, 0.0, "ЩУКА", C_WHITE, Markers::Color);
        let (lower, _) = layout(&font, 0.0, 0.0, "щука", C_WHITE, Markers::Color);
        let boxes: Vec<i32> = upper.iter().map(|letter| letter.glyph.x).collect();
        assert_eq!(boxes, lower.iter().map(|letter| letter.glyph.x).collect::<Vec<i32>>(), "the same letters either way");
        assert!(!boxes.contains(&font.glyph('?').unwrap().x), "and none of them a question mark: {boxes:?}");
    }

    #[test]
    fn width_skips_spaces_and_color_codes_and_counts_wide_letters() {
        let font = lettering();
        let width = |text: &str| layout(&font, 0.0, 0.0, text, C_WHITE, Markers::Color).1;
        assert_eq!(width("ab"), 12.0);
        assert_eq!(width("@a b~"), 17.0);
        assert_eq!(width("mw"), 16.0);
        assert_eq!(width("abc\nd"), 18.0);
        // A trailing space moves the pen but not the measured width.
        assert_eq!(width("a "), 6.0);
    }

    #[test]
    fn caller_color_overrides_codes() {
        let font = lettering();
        let red = make_color_rgb(255, 0, 0);
        let (letters, _) = layout(&font, 0.0, 0.0, "@a", red, Markers::Color);
        assert_eq!(letters[0].color, red);
        let (letters, _) = layout(&font, 0.0, 0.0, "@a~b", C_WHITE, Markers::Color);
        assert_eq!(letters[0].color, make_color_rgb(0x0F, 0xFF, 0x39));
        assert_eq!(letters[1].color, C_WHITE);
    }

    /// What a player types shows every character: "&" and "/" are drawn, not obeyed.
    #[test]
    fn typed_text_draws_the_markers_as_letters() {
        let font = lettering();
        let (colored, _) = layout(&font, 0.0, 0.0, "&a/b", C_WHITE, Markers::Color);
        let (typed, _) = layout(&font, 0.0, 0.0, "&a/b", C_WHITE, Markers::Show);
        assert_eq!(colored.len(), 2, "only a and b are letters in a colored text");
        assert_eq!(typed.len(), 4, "every character is a letter in typed text");
        assert!(typed.iter().all(|letter| letter.color == C_WHITE));
    }
}
