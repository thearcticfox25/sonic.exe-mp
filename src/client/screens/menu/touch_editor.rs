//! The touch controls page (SUBMENU_OPTIONSCONTROLSMOBILE): obj_menu_mobiledpad,
//! obj_menu_mobilebutton, obj_menu_mobilebutton2 and obj_menu_slider. A tap chooses a
//! control, a second tap soon after makes it bigger (and the biggest small again), a
//! finger held on it drags it, and the slider sets the chosen control's opacity.
//! Every change is saved at once, as obj_controls.buttonSetProp and buttonSetPos did.

use super::widgets::Builtins;
use crate::client::canvas::Canvas;
use crate::client::input::Input;
use crate::client::options::Options;
use crate::client::touch::{Control, CONTROLS};
use crate::core::collision::sprite_bbox;
use crate::core::config::original_ticks;

/// dragTime >= 20: a finger held this long on a control starts dragging it.
const DRAG_AFTER_STEPS: i32 = 20;
/// tapTimer = 10: the second tap of a double tap comes within this many steps.
const DOUBLE_TAP_STEPS: i32 = 10;
/// A double tap makes a control this much bigger, from the smallest back after the biggest.
const SCALE_STEP: f64 = 0.5;
const SMALLEST_SCALE: f64 = 0.5;
const BIGGEST_SCALE: f64 = 1.5;
/// The slider's opacity comes in fifths, one frame of spr_menu_slider each.
const SLIDER_STEPS: f64 = 5.0;

pub struct TouchEditor {
    selected: Control,
    /// dragTime of each control, in CONTROLS order.
    held_ticks: [i32; CONTROLS.len()],
    /// tapCount and tapTimer of each control.
    taps: [(u32, i32); CONTROLS.len()],
    /// obj_menu_slider.value. It starts at the chosen control's opacity; the original
    /// started it at 0.5, which set the joystick to 0.5 whenever the page opened.
    slider: f64,
}

impl TouchEditor {
    pub fn new(options: &Options) -> TouchEditor {
        TouchEditor {
            selected: Control::Joystick,
            held_ticks: Default::default(),
            taps: Default::default(),
            slider: options.touch_layout.get(Control::Joystick).opacity,
        }
    }

    /// The control an editor object on menu.tmj stands for: the joystick has no bid,
    /// the buttons are numbered in CONTROLS order.
    pub fn control_of(bid: i32) -> Control {
        CONTROLS.get(bid.max(0) as usize).copied().unwrap_or(Control::Joystick)
    }

    /// Draw of one control's editor object: it shows the control where the layout has
    /// it, and takes the taps and drags on it. `over`: the mouse (a finger) is on it.
    pub fn step_control(&mut self, control: Control, widget: &mut Builtins, over: bool, input: &Input, options: &mut Options) {
        let index = CONTROLS.iter().position(|&known| known == control).expect("every control is listed");
        let before = options.touch_layout.get(control);
        let placement = options.touch_layout.get_mut(control);

        if input.mouse_any_held() {
            if over && self.held_ticks[index] < original_ticks(DRAG_AFTER_STEPS) {
                self.held_ticks[index] += 1;
            }
        } else {
            self.held_ticks[index] = 0;
        }
        let taps = &mut self.taps[index];
        if input.mouse_any_released() && over {
            self.selected = control;
            self.slider = placement.opacity;
            taps.0 += 1;
            if taps.0 >= 2 {
                placement.scale = if placement.scale + SCALE_STEP > BIGGEST_SCALE { SMALLEST_SCALE } else { placement.scale + SCALE_STEP };
                taps.0 = 0;
            }
            taps.1 = original_ticks(DOUBLE_TAP_STEPS);
        }
        if taps.1 > 0 {
            taps.1 -= 1;
        } else {
            taps.0 = 0;
        }
        if self.held_ticks[index] >= original_ticks(DRAG_AFTER_STEPS) {
            (placement.x, placement.y) = (input.mouse_x, input.mouse_y);
            // One control follows the finger at a time.
            for (other, ticks) in self.held_ticks.iter_mut().enumerate() {
                if other != index {
                    *ticks = 0;
                }
            }
        }

        let placement = *placement;
        (widget.x, widget.y) = (placement.x, placement.y);
        (widget.xscale, widget.yscale) = (placement.scale, placement.scale);
        widget.image_alpha = placement.opacity;
        if placement != before {
            options.save();
        }
    }

    /// Draw of obj_menu_slider: dragging along it picks the opacity, which the chosen
    /// control takes (obj_menu.sliderChanged).
    pub fn step_slider(&mut self, widget: &mut Builtins, over: bool, canvas: &Canvas, input: &Input, options: &mut Options) {
        if over && input.mouse_any_held() {
            if let Some(sprite) = widget.sprite {
                let bbox = sprite_bbox(canvas.sprites.get(sprite), widget.x, widget.y, widget.xscale, widget.yscale, widget.angle);
                let along = (input.mouse_x - bbox.left) / (bbox.right - bbox.left);
                self.slider = ((along * SLIDER_STEPS).floor() / SLIDER_STEPS).clamp(0.0, 1.0);
            }
        }
        widget.image_index = self.slider * SLIDER_STEPS;
        let opacity = &mut options.touch_layout.get_mut(self.selected).opacity;
        if *opacity != self.slider {
            *opacity = self.slider;
            options.save();
        }
    }
}
