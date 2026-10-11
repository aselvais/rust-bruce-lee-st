mod audio;
mod game;
mod gfx;
mod level;

use audio::Audio;
use game::{Game, Input};
use gfx::{Assets, Frame};
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let assets = Assets::load();
    if args.iter().any(|a| a == "--shots") {
        render_shots(&assets);
        return;
    }
    let scale = if args.iter().any(|a| a == "--1") { Scale::X1 } else { Scale::X2 };
    let mut window = Window::new(
        "Bruce Lee — Level 1",
        level::SCREEN_W as usize,
        level::SCREEN_H as usize,
        WindowOptions {
            resize: false,
            scale,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("could not open a window");
    window.set_target_fps(60);
    let mut game = Game::new();
    let mut frame = Frame::new(level::SCREEN_W, level::SCREEN_H);
    let mut audio = Audio::open();
    if audio.is_none() {
        eprintln!("audio output is unavailable; the game will run silent");
    }
    let mut prev = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - prev).as_secs_f32().clamp(1.0 / 120.0, 1.0 / 30.0);
        prev = now;
        if window.is_key_pressed(Key::F1, KeyRepeat::No) {
            game.show_hit = !game.show_hit;
        }
        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            if let Some(out) = audio.as_mut() {
                out.toggle_mute();
            }
        }
        game.update(&read_input(&window), dt);
        let cues = game.take_sounds();
        if let Some(out) = audio.as_mut() {
            out.sync(game.mode, cues);
        }
        frame.clear(0);
        game.draw(&assets, &mut frame);
        if window.update_with_buffer(&frame.px, level::SCREEN_W as usize, level::SCREEN_H as usize).is_err() {
            break;
        }
    }
}

fn read_input(window: &Window) -> Input {
    let held = |keys: &[Key]| keys.iter().any(|k| window.is_key_down(*k));
    let edge = |keys: &[Key]| keys.iter().any(|k| window.is_key_pressed(*k, KeyRepeat::No));
    Input {
        left: held(&[Key::Left, Key::A]),
        right: held(&[Key::Right, Key::D]),
        up: held(&[Key::Up, Key::W]),
        down: held(&[Key::Down, Key::S]),
        jump_edge: edge(&[Key::Space]),
        jump_held: held(&[Key::Space, Key::Up, Key::W]),
        up_edge: edge(&[Key::Up, Key::W]),
        attack_edge: edge(&[Key::Z, Key::X, Key::J, Key::K]),
        start_edge: edge(&[Key::Enter]),
        pause_edge: edge(&[Key::P]),
    }
}

fn render_shots(assets: &Assets) {
    std::fs::create_dir_all("shots").expect("shots dir");
    let mut frame = Frame::new(level::SCREEN_W, level::SCREEN_H);
    let scenes = ["title", "court", "fight", "hall", "gallery", "clear"];
    let names = [
        "01_title.png",
        "02_gate_court.png",
        "03_fight.png",
        "04_middle_hall.png",
        "05_east_gallery.png",
        "06_level_clear.png",
    ];
    for (scene, name) in scenes.iter().zip(names.iter()) {
        let mut game = Game::new();
        game.stage(scene);
        game.draw(assets, &mut frame);
        save_2x(&format!("shots/{name}"), &frame);
        println!("wrote shots/{name}");
    }
}

fn save_2x(path: &str, frame: &Frame) {
    let w = frame.w as u32;
    let h = frame.h as u32;
    let mut img = image::RgbaImage::new(w * 2, h * 2);
    for y in 0..h {
        for x in 0..w {
            let p = frame.px[(y * w + x) as usize];
            let px = image::Rgba([(p >> 16) as u8, (p >> 8) as u8, p as u8, 255]);
            img.put_pixel(x * 2, y * 2, px);
            img.put_pixel(x * 2 + 1, y * 2, px);
            img.put_pixel(x * 2, y * 2 + 1, px);
            img.put_pixel(x * 2 + 1, y * 2 + 1, px);
        }
    }
    img.save(path).expect("save png");
}
