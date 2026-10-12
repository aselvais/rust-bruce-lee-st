//! Browser shell. The page owns the canvas, the keys, and Web Audio.
//! This module steps the game and returns pixels plus the synthesized clips.

use wasm_bindgen::prelude::*;

use crate::audio::{self, ClipSet};
use crate::game::{Game, Input, Mode};
use crate::gfx::{Assets, Frame};
use crate::level;

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    assets: Assets,
    frame: Frame,
    rgba: Vec<u8>,
    clips: ClipSet,
    sfx: Vec<u8>,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebGame {
        let w = level::SCREEN_W;
        let h = level::SCREEN_H;
        let mut shell = WebGame {
            game: Game::new(),
            assets: Assets::load(),
            frame: Frame::new(w, h),
            rgba: vec![0; (w * h * 4) as usize],
            clips: audio::clips(),
            sfx: Vec::new(),
        };
        shell.redraw();
        shell
    }

    pub fn width(&self) -> u32 {
        self.frame.w as u32
    }

    pub fn height(&self) -> u32 {
        self.frame.h as u32
    }

    pub fn sample_rate(&self) -> u32 {
        audio::SAMPLE_RATE
    }

    /// 0 title, 1 level, 2 clear fanfare, 3 clear bed, 4 game over, then 5..=14 effects.
    pub fn clip_samples(&self, id: u32) -> Vec<f32> {
        let clips = &self.clips;
        match id {
            0 => clips.title.clone(),
            1 => clips.level.clone(),
            2 => clips.clear.clone(),
            3 => clips.clear_bed.clone(),
            4 => clips.over.clone(),
            n if (5..15).contains(&n) => clips.fx[(n - 5) as usize].clone(),
            _ => Vec::new(),
        }
    }

    /// Advance one frame.
    ///
    /// `bits` uses the same edges as the desktop window:
    /// 0 left, 1 right, 2 up, 3 down, 4 jump edge (Space), 5 jump held,
    /// 6 up edge, 7 attack edge, 8 start edge (Enter), 9 pause edge,
    /// 10 debug outline (F1).
    /// `dt` is seconds and is clamped to the desktop range.
    /// The return value is the mode: 0 title, 1 play, 2 pause, 3 clear, 4 game over.
    pub fn tick(&mut self, bits: u32, dt: f32) -> u32 {
        if bit(bits, 10) {
            self.game.show_hit = !self.game.show_hit;
        }
        let input = Input {
            left: bit(bits, 0),
            right: bit(bits, 1),
            up: bit(bits, 2),
            down: bit(bits, 3),
            jump_edge: bit(bits, 4),
            jump_held: bit(bits, 5),
            up_edge: bit(bits, 6),
            attack_edge: bit(bits, 7),
            start_edge: bit(bits, 8),
            pause_edge: bit(bits, 9),
        };
        let dt = dt.clamp(1.0 / 120.0, 1.0 / 30.0);
        self.game.update(&input, dt);
        self.sfx.clear();
        for cue in self.game.take_sounds() {
            self.sfx.push(audio::sfx_index(cue) as u8);
        }
        self.redraw();
        mode_id(self.game.mode)
    }

    pub fn take_sfx(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.sfx)
    }

    /// Byte offset of the RGBA framebuffer in wasm memory. Stable for the life of this game.
    pub fn pixels(&self) -> u32 {
        self.rgba.as_ptr() as u32
    }

    pub fn pixel_len(&self) -> u32 {
        self.rgba.len() as u32
    }
}

impl WebGame {
    fn redraw(&mut self) {
        self.frame.clear(0);
        self.game.draw(&self.assets, &mut self.frame);
        let px = &self.frame.px;
        let rgba = &mut self.rgba;
        for (i, p) in px.iter().enumerate() {
            let o = i * 4;
            rgba[o] = (*p >> 16) as u8;
            rgba[o + 1] = (*p >> 8) as u8;
            rgba[o + 2] = *p as u8;
            rgba[o + 3] = 255;
        }
    }
}

fn bit(bits: u32, n: u32) -> bool {
    bits & (1 << n) != 0
}

fn mode_id(mode: Mode) -> u32 {
    match mode {
        Mode::Title => 0,
        Mode::Play => 1,
        Mode::Pause => 2,
        Mode::Clear => 3,
        Mode::Over => 4,
    }
}
