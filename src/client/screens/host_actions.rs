//! What the server's ?gui window (states::host_gui) does to a player: a small list of
//! actions over the screen, up and down to choose, Z to do it, X to close. The window
//! is the server's own and needs no rights from it: it prints the terminal command
//! that does the action, and the server's terminal, which reads what the window
//! prints, runs it as if the host typed it (terminal commands.rs open_gui).

use super::menu::SELECTED_BLEND;
use crate::client::canvas::{Canvas, VIEW_HEIGHT, VIEW_WIDTH};
use crate::client::Context;
use crate::core::resources::names::sound;
use crate::core::world::{C_BLACK, C_WHITE};

/// The label of each action and the ?select-player action it runs; "cancel" runs none.
/// The list opens on "cancel", so the press that picked the player, pressed once more
/// by a double click, does nothing to them.
const CANCEL: usize = ACTIONS.len() - 1;
const ACTIONS: [(&str, &str); 6] = [
    ("kick", "kick"),
    ("ban", "ban"),
    ("make operator", "op"),
    ("take operator away", "deop"),
    ("add to whitelist", "whitelist-append"),
    ("cancel", ""),
];
const LINE_HEIGHT: f64 = 16.0;
const BOX_PADDING: f64 = 8.0;
const BOX_ALPHA: f64 = 0.75;

pub struct HostActions {
    player: u16,
    title: String,
    choice: usize,
}

impl HostActions {
    pub fn new(player: u16, nickname: &str) -> HostActions {
        HostActions { player, title: format!("{nickname} (id {player})"), choice: CANCEL }
    }

    /// One step of the open list. Returns false once it is closed.
    pub fn step(&mut self, context: &mut Context) -> bool {
        let keys = &context.options.keys;
        let input = &context.input;
        if input.pressed(keys.special1.0) {
            return false;
        }
        if input.pressed(keys.jump.0) {
            let (_, action) = ACTIONS[self.choice];
            if !action.is_empty() {
                // Nobody may be reading any more (the server stopped): then it is lost.
                use std::io::Write;
                let _ = writeln!(std::io::stdout(), "?select-player --pid {} --action {action}", self.player);
            }
            return false;
        }
        let step: i32 = if input.pressed(keys.up.0) {
            -1
        } else if input.pressed(keys.down.0) {
            1
        } else {
            return true;
        };
        self.choice = (self.choice as i32 + step).rem_euclid(ACTIONS.len() as i32) as usize;
        context.audio.play(sound::SND_MENU_SELECT, false);
        true
    }

    /// The list in the middle of the screen, the player it is about above it.
    pub fn draw(&self, canvas: &mut Canvas) {
        let lines = std::iter::once(self.title.as_str()).chain(ACTIONS.iter().map(|(label, _)| *label));
        let width = lines.map(|line| canvas.menu_text_width(line)).fold(0.0, f64::max) + 2.0 * BOX_PADDING;
        let height = (ACTIONS.len() + 1) as f64 * LINE_HEIGHT + 2.0 * BOX_PADDING;
        let (left, top) = (((VIEW_WIDTH - width) / 2.0).floor(), ((VIEW_HEIGHT - height) / 2.0).floor());
        canvas.fill_rectangle(left, top, width, height, C_BLACK, BOX_ALPHA);
        let text_left = left + BOX_PADDING;
        canvas.draw_menu_text(text_left, top + BOX_PADDING, &self.title, C_WHITE, 1.0);
        for (index, (label, _)) in ACTIONS.iter().enumerate() {
            let color = if index == self.choice { SELECTED_BLEND } else { C_WHITE };
            canvas.draw_menu_text(text_left, top + BOX_PADDING + (index + 1) as f64 * LINE_HEIGHT, label, color, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_opens_on_cancel() {
        let actions = HostActions::new(7, "fox");
        assert_eq!(ACTIONS[actions.choice], ("cancel", ""));
    }
}
