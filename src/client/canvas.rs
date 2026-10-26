//! The 480x270 picture a room draws into, with GameMaker's sprite drawing
//! functions. At the end of a frame the picture is stretched over the whole
//! window without smoothing (the project's "full scale" and "no interpolation"
//! options).

use crate::client::font::{Font, Glyph};
use crate::client::palette::Colors;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams, UniformDesc, UniformType};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::resources::sprites::Sprites;
use crate::core::resources::SpriteId;

/// GameMaker color constants (0xBBGGRR) that the client draws with.
pub const C_RED: u32 = 0x0000FF;
pub const C_GREEN: u32 = 0x008000;
pub const C_DKGRAY: u32 = 0x404040;

pub use crate::core::{VIEW_HEIGHT, VIEW_WIDTH};

/// make_color_rgb: GameMaker keeps colors as 0xBBGGRR.
pub const fn make_color_rgb(red: u8, green: u8, blue: u8) -> u32 {
    (blue as u32) << 16 | (green as u32) << 8 | red as u32
}

/// Where the picture stands in the window: as big as fits whole, never stretched out of
/// its 16:9, with black bars on the sides or above and below (fullscreen on another
/// shape of monitor, or a window resized to one). Its left, top and scale.
fn picture_in_window() -> (f32, f32, f32) {
    let (width, height) = (screen_width(), screen_height());
    let scale = (width / VIEW_WIDTH as f32).min(height / VIEW_HEIGHT as f32);
    let left = ((width - VIEW_WIDTH as f32 * scale) / 2.0).floor();
    let top = ((height - VIEW_HEIGHT as f32 * scale) / 2.0).floor();
    (left, top, scale)
}

/// The palette swap shader takes 256 floats per list: 64 colors.
const MAX_PALETTE_COLORS: usize = 64;

const PALETTE_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

/// The palette swap shader: a pixel equal (within 0.01 per channel) to a pattern color is
/// drawn as that color's replacement.
const PALETTE_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform vec4 Pattern[64];
uniform vec4 Replacements[64];
uniform int ColorCount;
void main() {
    vec4 texel = texture2D(Texture, uv);
    for (int i = 0; i < 64; i++) {
        if (i >= ColorCount) {
            break;
        }
        if (all(lessThan(abs(texel - Pattern[i]), vec4(0.01)))) {
            texel = Replacements[i];
            break;
        }
    }
    gl_FragColor = color * texel;
}"#;

/// shd_blackwhite: every pixel as its brightness.
const BLACK_WHITE_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    vec4 source = color * texture2D(Texture, uv);
    float brightness = dot(vec3(0.2989, 0.5870, 0.1140), source.rgb);
    gl_FragColor = vec4(vec3(brightness), source.a);
}"#;

/// shd_inverse: inside a circle around a point the picture is inverted and swirled.
const INVERSE_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform vec2 Point;
uniform float Radius;
uniform float Scale;
void main() {
    vec2 view_size = vec2(480.0, 270.0);
    // The picture is stored upside down.
    vec2 pixel = vec2(uv.x, 1.0 - uv.y) * view_size;
    float len = length(pixel - Point) * Scale;
    float radius = Radius * Scale;
    if (len < radius) {
        float size = max(len - radius / 2.0, 0.0) / radius * 10.0 * Scale;
        vec2 offset = vec2(cos(len * 0.2), -sin(len * 0.2)) * size / (view_size * Scale);
        vec4 source = color * texture2D(Texture, uv + offset);
        gl_FragColor = vec4(1.0 - source.rgb, source.a);
    } else {
        gl_FragColor = color * texture2D(Texture, uv);
    }
}"#;

/// GameMaker's _filter_heathaze on one layer. Two samples of a small noise picture,
/// stretched by their own scale across and down and rolling upwards with time, say how
/// far each pixel of the layer is pushed (a pixel or two), and red, green and blue are
/// pushed a little differently. The noise is anchored to the room, so it does not
/// slide with the camera. The layer comes in drawn on its own picture, which render
/// targets keep upside down: `uv` counts from the bottom, hence the flips of y.
///
/// _filter_underwater is the same push, then the layer is colored: where the noise
/// pushes hardest a glint is added, then the whole layer is tinted and given a color on
/// top. The heat haze is drawn with no glint, a white tint and nothing added.
const HEAT_HAZE_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform sampler2D Noise;
uniform vec2 NoiseSize;
uniform vec2 ViewSize;
uniform vec2 View;
uniform float Time;
uniform vec4 Scales;
uniform vec2 Speeds;
uniform vec2 Amounts;
uniform float ChromaSpread;
uniform float CamOffsetScale;
uniform vec3 Glint;
uniform vec3 Tint;
uniform vec3 Add;

vec2 noise_push(vec2 pixel, vec2 scale, float speed) {
    vec2 coord = (pixel + View * CamOffsetScale) / (NoiseSize * scale);
    coord.y += speed * Time;
    return texture2D(Noise, fract(coord)).rg - 0.5;
}

void main() {
    vec2 pixel = vec2(uv.x, 1.0 - uv.y) * ViewSize;
    vec2 push = noise_push(pixel, Scales.xy, Speeds.x) * Amounts.x + noise_push(pixel, Scales.zw, Speeds.y) * Amounts.y;
    vec2 offset = vec2(push.x, -push.y) / ViewSize;
    vec4 red = texture2D(Texture, uv + offset * (1.0 + ChromaSpread));
    float green = texture2D(Texture, uv + offset * (1.0 + ChromaSpread * 0.5)).g;
    float blue = texture2D(Texture, uv + offset).b;
    // 1 where the push is as strong as the two amounts on average, and fading fast below.
    vec2 strength = push / ((Amounts.x + Amounts.y) * 0.5);
    float glint = pow(dot(strength, strength) * 0.70711, 4.0);
    vec3 colored = (vec3(red.r, green, blue) + glint * Glint) * Tint + Add;
    gl_FragColor = color * vec4(colored, red.a);
}"#;

/// What a map asks the heat haze for: the effect properties of a layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatHaze {
    /// Distort 1 and 2: how far the noise is stretched across and down.
    pub scales: [(f32, f32); 2],
    /// Distort 1 and 2: noise pictures a second the noise rolls up.
    pub speeds: [f32; 2],
    /// Distort 1 and 2: pixels the noise pushes at most each way, times two.
    pub amounts: [f32; 2],
    pub chroma_spread: f32,
    /// How much the noise follows the room rather than the screen.
    pub camera_offset_scale: f32,
    /// The picture of noise that pushes the pixels.
    pub noise: SpriteId,
    /// Underwater only: a color added where the push is strongest, one multiplied over
    /// the layer and one added over it.
    pub glint: u32,
    pub tint: u32,
    pub add: u32,
}

pub struct Canvas {
    pub sprites: Arc<Sprites>,
    palette_swap: Material,
    black_white: Material,
    inverse: Material,
    heat_haze: Material,
    /// A layer with a heat haze is drawn here first (start_heat_haze).
    scratch: RenderTarget,
    /// The heat haze option: off, layers with a heat haze are drawn as they are.
    pub heat_haze_on: bool,
    /// The inverted circle over the whole picture (view position and radius), if any.
    pub inverse_circle: Option<(f64, f64, f64)>,
    /// Uploaded frames, by sprite. A sprite is uploaded the first time it is drawn.
    // ponytail: textures stay until the game closes; drop them on room change once levels fill the GPU
    frames: HashMap<SpriteId, Vec<Texture2D>>,
    target: RenderTarget,
    camera: Camera2D,
    /// view_camera[0]: the room position shown at the picture's top-left corner.
    view: (f64, f64),
    /// _filter_tintfilter of the layer being drawn: every sprite's color times this.
    layer_tint: u32,
    /// The menu buttons' lettering, fnt_big, the chat lettering and the headings',
    /// and their pictures once uploaded.
    fonts: [Font; 4],
    font_textures: [Option<Texture2D>; 4],
}

/// The bitmap fonts a canvas draws with.
#[derive(Clone, Copy)]
pub enum TextFont {
    Menu = 0,
    /// fnt_big
    Big = 1,
    /// The letters the original drew chat, names and the version with (spr_letter1).
    Chat = 2,
    /// The lettering of headings such as the player list's (Fonts/Header.png).
    Header = 3,
}

impl Canvas {
    pub fn new(sprites: Arc<Sprites>, fonts: [Font; 4]) -> Canvas {
        let target = render_target(VIEW_WIDTH as u32, VIEW_HEIGHT as u32);
        target.texture.set_filter(FilterMode::Nearest);
        let camera = view_camera(&target, 0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT);
        Canvas {
            sprites,
            palette_swap: palette_swap_material(),
            black_white: black_white_material(),
            inverse: inverse_material(),
            heat_haze: heat_haze_material(),
            scratch: {
                let scratch = render_target(VIEW_WIDTH as u32, VIEW_HEIGHT as u32);
                scratch.texture.set_filter(FilterMode::Nearest);
                scratch
            },
            heat_haze_on: true,
            inverse_circle: None,
            frames: HashMap::new(),
            target,
            camera,
            view: (0.0, 0.0),
            layer_tint: crate::core::world::C_WHITE,
            fonts,
            font_textures: [None, None, None, None],
        }
    }

    /// Draws one line of `text` in the menu font with its top-left corner at (x, y).
    pub fn draw_menu_text(&mut self, x: f64, y: f64, text: &str, color: u32, alpha: f64) {
        self.draw_font_text(TextFont::Menu, x, y, text, color, alpha, 1.0);
    }

    /// Draws one line of `text` with its top-left corner at (x, y), `scale` times as big.
    pub fn draw_font_text(&mut self, font: TextFont, x: f64, y: f64, text: &str, color: u32, alpha: f64, scale: f64) {
        let (font, texture) = (&self.fonts[font as usize], &mut self.font_textures[font as usize]);
        let texture = texture.get_or_insert_with(|| {
            let texture = Texture2D::from_rgba8(font.image.width as u16, font.image.height as u16, &font.image.frames[0]);
            texture.set_filter(FilterMode::Nearest);
            texture
        });
        let (texture, size) = (texture.clone(), (font.image.width as f64, font.image.height as f64));
        let placed: Vec<(f64, Glyph)> = font.layout(text).map(|(start, glyph)| (start, *glyph)).collect();
        for (start, glyph) in placed {
            draw_glyph(&texture, size, &glyph, x + start * scale, y, scale, color, alpha);
        }
    }

    /// One glyph of `font` with its top-left corner at (x, y), for text laid out by
    /// the caller (the chat lettering colors every letter on its own).
    pub fn draw_font_glyph(&mut self, font: TextFont, glyph: &Glyph, x: f64, y: f64, color: u32, alpha: f64) {
        let (font, texture) = (&self.fonts[font as usize], &mut self.font_textures[font as usize]);
        let texture = texture.get_or_insert_with(|| {
            let texture = Texture2D::from_rgba8(font.image.width as u16, font.image.height as u16, &font.image.frames[0]);
            texture.set_filter(FilterMode::Nearest);
            texture
        });
        let size = (font.image.width as f64, font.image.height as f64);
        draw_glyph(&texture.clone(), size, glyph, x, y, 1.0, color, alpha);
    }

    /// The font itself, for code that lays text out letter by letter.
    pub fn font(&self, font: TextFont) -> &Font {
        &self.fonts[font as usize]
    }

    pub fn menu_text_width(&self, text: &str) -> f64 {
        self.fonts[TextFont::Menu as usize].width(text)
    }

    /// The width and line height of `text` in `font`.
    pub fn font_text_size(&self, font: TextFont, text: &str) -> (f64, f64) {
        let font = &self.fonts[font as usize];
        (font.width(text), font.line_height())
    }

    /// scr_palette_swap: sprites drawn until `reset_shader` get their colors replaced.
    pub fn set_palette_swap(&self, from: &Colors, to: &Colors) {
        let as_uniform = |colors: &Colors| {
            let mut padded = [Vec4::ZERO; MAX_PALETTE_COLORS];
            for (slot, color) in padded.iter_mut().zip(colors) {
                let [red, green, blue] = color.map(|channel| crate::client::palette::fraction(channel) as f32);
                *slot = Vec4::new(red, green, blue, 1.0);
            }
            padded
        };
        let count = from.len().min(to.len()).min(MAX_PALETTE_COLORS) as i32;
        gl_use_material(&self.palette_swap);
        self.palette_swap.set_uniform_array("Pattern", &as_uniform(from));
        self.palette_swap.set_uniform_array("Replacements", &as_uniform(to));
        self.palette_swap.set_uniform("ColorCount", count);
    }

    /// A layer's _filter_tintfilter while it is drawn; C_WHITE when it is done.
    pub fn set_layer_tint(&mut self, tint: u32) {
        self.layer_tint = tint;
    }

    /// shader_set(shd_blackwhite), and _filter_greyscale at full intensity
    pub fn set_black_white(&self) {
        gl_use_material(&self.black_white);
    }

    /// shader_reset
    pub fn reset_shader(&self) {
        gl_use_default_material();
    }

    /// Starts a new picture; everything drawn until the next `begin` goes into it.
    /// Drawing starts in the picture's own coordinates, as in a room without a view.
    pub fn begin(&mut self) {
        self.set_view(0.0, 0.0);
        clear_background(BLACK);
    }

    /// camera_set_view_pos: what follows is drawn in room coordinates, with this room
    /// position at the picture's top-left corner. Draw GUI is the view at (0, 0).
    pub fn set_view(&mut self, x: f64, y: f64) {
        self.set_view_sized(x, y, VIEW_WIDTH, VIEW_HEIGHT);
    }

    /// camera_set_view_pos and camera_set_view_size: a view larger than the picture
    /// shows more of the room, squeezed into it (Dark Tower's jumpscare).
    pub fn set_view_sized(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.view = (x, y);
        self.camera = view_camera(&self.target, x, y, width, height);
        set_camera(&self.camera);
    }

    /// camera_get_view_x, camera_get_view_y
    pub fn view(&self) -> (f64, f64) {
        self.view
    }

    /// GameMaker's filter on a single layer, begun: when `haze` is set and the option
    /// is on, what is drawn until finish_heat_haze lands on a picture of its own.
    /// Returns what finish_heat_haze needs. Only that layer shimmers, as GameMaker
    /// applies a layer's own filter to that layer alone.
    pub fn start_heat_haze(&mut self, haze: Option<HeatHaze>) -> Option<HeatHaze> {
        let haze = haze.filter(|_| self.heat_haze_on)?;
        std::mem::swap(&mut self.target, &mut self.scratch);
        self.set_view(self.view.0, self.view.1);
        clear_background(BLANK);
        Some(haze)
    }

    /// The layer begun by start_heat_haze comes onto the picture being built, pushed
    /// around by the heat haze.
    pub fn finish_heat_haze(&mut self, haze: Option<HeatHaze>) {
        let Some(haze) = haze else { return };
        std::mem::swap(&mut self.target, &mut self.scratch);
        self.set_view(self.view.0, self.view.1);
        let Some(noise) = self.frame_texture(haze.noise, 0.0) else { return };
        noise.set_filter(FilterMode::Linear);
        let material = &self.heat_haze;
        material.set_texture("Noise", noise.clone());
        material.set_uniform("NoiseSize", (noise.width(), noise.height()));
        material.set_uniform("ViewSize", (VIEW_WIDTH as f32, VIEW_HEIGHT as f32));
        material.set_uniform("View", (self.view.0 as f32, self.view.1 as f32));
        material.set_uniform("Time", get_time() as f32);
        let [(scale1_x, scale1_y), (scale2_x, scale2_y)] = haze.scales;
        material.set_uniform("Scales", (scale1_x, scale1_y, scale2_x, scale2_y));
        material.set_uniform("Speeds", (haze.speeds[0], haze.speeds[1]));
        material.set_uniform("Amounts", (haze.amounts[0], haze.amounts[1]));
        material.set_uniform("ChromaSpread", haze.chroma_spread);
        material.set_uniform("CamOffsetScale", haze.camera_offset_scale);
        let rgb = |color| { let color = gm_color(color, 1.0); (color.r, color.g, color.b) };
        material.set_uniform("Glint", rgb(haze.glint));
        material.set_uniform("Tint", rgb(haze.tint));
        material.set_uniform("Add", rgb(haze.add));
        gl_use_material(material);
        let whole_view = DrawTextureParams { dest_size: Some(vec2(VIEW_WIDTH as f32, VIEW_HEIGHT as f32)), flip_y: true, ..Default::default() };
        draw_texture_ex(&self.scratch.texture, self.view.0 as f32, self.view.1 as f32, WHITE, whole_view);
        gl_use_default_material();
    }

    /// Shows the latest picture in the window.
    pub fn present(&self) {
        set_default_camera();
        clear_background(BLACK);
        let (left, top, scale) = picture_in_window();
        let picture = DrawTextureParams {
            dest_size: Some(vec2(VIEW_WIDTH as f32 * scale, VIEW_HEIGHT as f32 * scale)),
            // Render targets are stored upside down.
            flip_y: true,
            ..Default::default()
        };
        if let Some((x, y, radius)) = self.inverse_circle.filter(|&(_, _, radius)| radius > 0.0) {
            self.inverse.set_uniform("Point", (x as f32, y as f32));
            self.inverse.set_uniform("Radius", radius as f32);
            self.inverse.set_uniform("Scale", scale);
            gl_use_material(&self.inverse);
            draw_texture_ex(&self.target.texture, left, top, WHITE, picture);
            gl_use_default_material();
        } else {
            draw_texture_ex(&self.target.texture, left, top, WHITE, picture);
        }
    }

    /// A window pixel as a position in the picture (mouse_x, mouse_y).
    pub fn window_to_view(&self, x: f32, y: f32) -> (f64, f64) {
        let (left, top, scale) = picture_in_window();
        (((x - left) / scale) as f64, ((y - top) / scale) as f64)
    }

    pub fn draw_sprite(&mut self, sprite: SpriteId, frame: f64, x: f64, y: f64) {
        self.draw_sprite_ext(sprite, frame, x, y, 1.0, 1.0, 0.0, crate::core::world::C_WHITE, 1.0);
    }

    /// draw_sprite_ext: the sprite origin lands on (x, y); scale and rotation
    /// (degrees, counter-clockwise) happen around it. `blend` tints, `alpha` fades.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite_ext(&mut self, sprite: SpriteId, frame: f64, x: f64, y: f64, xscale: f64, yscale: f64, angle: f64, blend: u32, alpha: f64) {
        let (x, y) = snap(x, y);
        let meta = self.sprites.get(sprite);
        let (width, height, origin_x, origin_y) = (meta.width as f64, meta.height as f64, meta.origin_x, meta.origin_y);
        let Some(texture) = self.frame_texture(sprite, frame) else { return };

        let left = -origin_x * xscale;
        let right = (width - origin_x) * xscale;
        let top = -origin_y * yscale;
        let bottom = (height - origin_y) * yscale;
        let corners = [(left, top), (right, top), (right, bottom), (left, bottom)].map(|(dx, dy)| {
            let (turned_x, turned_y) = rotate(dx, dy, angle);
            (x + turned_x, y + turned_y)
        });
        draw_textured_quad(texture, corners, [0.0, 0.0, 1.0, 1.0], gm_color(multiply_colors(blend, self.layer_tint), alpha));
    }

    /// draw_sprite_part_ext: a rectangle cut out of the sprite image, its top-left
    /// corner at (x, y). The sprite origin is not used. Parts outside the image are cut off.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite_part_ext(&mut self, sprite: SpriteId, frame: f64, part: [f64; 4], x: f64, y: f64, xscale: f64, yscale: f64, blend: u32, alpha: f64) {
        let (x, y) = snap(x, y);
        let meta = self.sprites.get(sprite);
        let (width, height) = (meta.width as f64, meta.height as f64);
        let Some(texture) = self.frame_texture(sprite, frame) else { return };

        let [part_left, part_top, part_width, part_height] = part;
        let (clipped_left, clipped_top) = (part_left.max(0.0), part_top.max(0.0));
        let clipped_right = (part_left + part_width).min(width);
        let clipped_bottom = (part_top + part_height).min(height);
        if clipped_right <= clipped_left || clipped_bottom <= clipped_top {
            return;
        }
        let screen_left = x + (clipped_left - part_left) * xscale;
        let screen_top = y + (clipped_top - part_top) * yscale;
        let screen_right = screen_left + (clipped_right - clipped_left) * xscale;
        let screen_bottom = screen_top + (clipped_bottom - clipped_top) * yscale;
        let corners = [(screen_left, screen_top), (screen_right, screen_top), (screen_right, screen_bottom), (screen_left, screen_bottom)];
        let uv = [clipped_left / width, clipped_top / height, clipped_right / width, clipped_bottom / height].map(|value| value as f32);
        draw_textured_quad(texture, corners, uv, gm_color(multiply_colors(blend, self.layer_tint), alpha));
    }

    /// draw_rectangle filled with one color (backgrounds without a sprite).
    /// The whole view dimmed (or tinted) by one color: a plain quad, no picture.
    pub fn fill_view(&mut self, color: u32, alpha: f64) {
        self.fill_rectangle(0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT, color, alpha);
    }

    pub fn fill_rectangle(&mut self, left: f64, top: f64, width: f64, height: f64, color: u32, alpha: f64) {
        draw_rectangle(left as f32, top as f32, width as f32, height as f32, gm_color(multiply_colors(color, self.layer_tint), alpha));
    }

    /// `frame` counts on past the last frame and wraps around, like image_index.
    /// Uploads a sprite's pictures now rather than the first time it is drawn.
    pub fn preload(&mut self, sprite: SpriteId) {
        let _ = self.frame_texture(sprite, 0.0);
    }

    fn frame_texture(&mut self, sprite: SpriteId, frame: f64) -> Option<Texture2D> {
        let frames = self.frames.entry(sprite).or_insert_with(|| upload_frames(&self.sprites, sprite));
        if frames.is_empty() {
            return None;
        }
        let index = (frame.floor() as i64).rem_euclid(frames.len() as i64) as usize;
        Some(frames[index].clone())
    }
}

/// Sprites land on whole pixels. Drawn half a pixel off (a centered word, a resource placed
/// at x.5 in a room), each texel sits on a pixel edge and rounds its own way, so letters
/// drift apart; the menu font's glyphs are floored the same way.
/// Puts one glyph on the canvas: its cell of the font picture, `scale` times as big,
/// snapped to whole pixels so letters never land between them.
fn draw_glyph(texture: &Texture2D, size: (f64, f64), glyph: &Glyph, x: f64, y: f64, scale: f64, color: u32, alpha: f64) {
    let (left, top) = ((x + glyph.offset as f64 * scale).floor(), y.floor());
    let (gw, gh) = (glyph.w as f64, glyph.h as f64);
    let (w, h) = (gw * scale, gh * scale);
    let corners = [(left, top), (left + w, top), (left + w, top + h), (left, top + h)];
    let uv = [glyph.x as f64 / size.0, glyph.y as f64 / size.1, (glyph.x as f64 + gw) / size.0, (glyph.y as f64 + gh) / size.1].map(|value| value as f32);
    draw_textured_quad(texture.clone(), corners, uv, gm_color(color, alpha));
}

fn snap(x: f64, y: f64) -> (f64, f64) {
    (x.floor(), y.floor())
}

fn view_camera(target: &RenderTarget, x: f64, y: f64, width: f64, height: f64) -> Camera2D {
    let mut camera = Camera2D::from_display_rect(Rect::new(x as f32, y as f32, width as f32, height as f32));
    camera.render_target = Some(target.clone());
    camera
}

fn black_white_material() -> Material {
    let shader = ShaderSource::Glsl { vertex: PALETTE_VERTEX_SHADER, fragment: BLACK_WHITE_FRAGMENT_SHADER };
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        ..Default::default()
    };
    load_material(shader, MaterialParams { pipeline_params, ..Default::default() }).expect("the black and white shader compiles")
}

fn heat_haze_material() -> Material {
    let shader = ShaderSource::Glsl { vertex: PALETTE_VERTEX_SHADER, fragment: HEAT_HAZE_FRAGMENT_SHADER };
    let uniforms = vec![
        UniformDesc::new("NoiseSize", UniformType::Float2),
        UniformDesc::new("ViewSize", UniformType::Float2),
        UniformDesc::new("View", UniformType::Float2),
        UniformDesc::new("Time", UniformType::Float1),
        UniformDesc::new("Scales", UniformType::Float4),
        UniformDesc::new("Speeds", UniformType::Float2),
        UniformDesc::new("Amounts", UniformType::Float2),
        UniformDesc::new("ChromaSpread", UniformType::Float1),
        UniformDesc::new("CamOffsetScale", UniformType::Float1),
        UniformDesc::new("Glint", UniformType::Float3),
        UniformDesc::new("Tint", UniformType::Float3),
        UniformDesc::new("Add", UniformType::Float3),
    ];
    // The layer lies over what is drawn already, its empty parts letting that through.
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        ..Default::default()
    };
    let params = MaterialParams { uniforms, textures: vec!["Noise".to_string()], pipeline_params };
    load_material(shader, params).expect("the heat haze shader compiles")
}

fn inverse_material() -> Material {
    let shader = ShaderSource::Glsl { vertex: PALETTE_VERTEX_SHADER, fragment: INVERSE_FRAGMENT_SHADER };
    let uniforms = vec![UniformDesc::new("Point", UniformType::Float2), UniformDesc::new("Radius", UniformType::Float1), UniformDesc::new("Scale", UniformType::Float1)];
    load_material(shader, MaterialParams { uniforms, ..Default::default() }).expect("the inverse shader compiles")
}

fn palette_swap_material() -> Material {
    let shader = ShaderSource::Glsl { vertex: PALETTE_VERTEX_SHADER, fragment: PALETTE_FRAGMENT_SHADER };
    let uniforms = vec![
        UniformDesc::new("Pattern", UniformType::Float4).array(MAX_PALETTE_COLORS),
        UniformDesc::new("Replacements", UniformType::Float4).array(MAX_PALETTE_COLORS),
        UniformDesc::new("ColorCount", UniformType::Int1),
    ];
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        ..Default::default()
    };
    load_material(shader, MaterialParams { pipeline_params, uniforms, ..Default::default() }).expect("the palette shader compiles")
}

fn upload_frames(sprites: &Sprites, sprite: SpriteId) -> Vec<Texture2D> {
    // Same policy as the sprite metadata: a sprite the code uses must exist.
    let image = sprites.read_image(sprite).unwrap_or_else(|error| panic!("sprite {sprite:?}: {error:#}"));
    image
        .frames
        .iter()
        .map(|pixels| {
            let texture = Texture2D::from_rgba8(image.width as u16, image.height as u16, pixels);
            texture.set_filter(FilterMode::Nearest);
            texture
        })
        .collect()
}

/// GameMaker turns counter-clockwise as seen on screen, where y grows downwards.
fn rotate(dx: f64, dy: f64, angle_degrees: f64) -> (f64, f64) {
    let (sin, cos) = angle_degrees.to_radians().sin_cos();
    (dx * cos + dy * sin, -dx * sin + dy * cos)
}

/// Two colors multiplied channel by channel, as a tint does to what it covers.
fn multiply_colors(a: u32, b: u32) -> u32 {
    let (a, b) = (a.to_le_bytes(), b.to_le_bytes());
    let channel = |i: usize| (u32::from(a[i]) * u32::from(b[i]) + 127) / 255;
    channel(0) | channel(1) << 8 | channel(2) << 16
}

fn gm_color(color: u32, alpha: f64) -> Color {
    let [red, green, blue, _] = color.to_le_bytes();
    Color::from_rgba(red, green, blue, (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// `corners` go clockwise from the top-left of the image; `uv` is left, top, right, bottom.
fn draw_textured_quad(texture: Texture2D, corners: [(f64, f64); 4], uv: [f32; 4], color: Color) {
    let [u_left, v_top, u_right, v_bottom] = uv;
    let uv_corners = [(u_left, v_top), (u_right, v_top), (u_right, v_bottom), (u_left, v_bottom)];
    let vertices = corners.iter().zip(uv_corners).map(|(&(x, y), (u, v))| Vertex::new(x as f32, y as f32, 0.0, u, v, color)).collect();
    draw_mesh(&Mesh { vertices, indices: vec![0, 1, 2, 0, 2, 3], texture: Some(texture) });
}
