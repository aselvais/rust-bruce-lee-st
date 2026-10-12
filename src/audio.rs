//! Original music and effects. Synthesized at startup, not taken from the 1984 game.
//!
//! Playback on the desktop goes through `rodio`. The browser build takes the same
//! samples from [`clips`] and plays them with the Web Audio API.

use crate::game::Sfx;
#[cfg(not(target_arch = "wasm32"))]
use crate::game::Mode;

#[cfg(not(target_arch = "wasm32"))]
use rodio::buffer::SamplesBuffer;
#[cfg(not(target_arch = "wasm32"))]
use rodio::source::Source;
#[cfg(not(target_arch = "wasm32"))]
use rodio::{nz, DeviceSinkBuilder, MixerDeviceSink, Player};
#[cfg(not(target_arch = "wasm32"))]
use std::num::NonZeroU32;

pub const SAMPLE_RATE: u32 = 22_050;
const SR: u32 = SAMPLE_RATE;

/// Effect index into [`ClipSet::fx`]. Desktop and web playback share this order.
pub fn sfx_index(cue: Sfx) -> usize {
    match cue {
        Sfx::Jump => 0,
        Sfx::Punch => 1,
        Sfx::Kick => 2,
        Sfx::Lantern => 3,
        Sfx::Hit => 4,
        Sfx::Hurt => 5,
        Sfx::Ko => 6,
        Sfx::Zap => 7,
        Sfx::Open => 8,
        Sfx::Throw => 9,
    }
}

pub struct ClipSet {
    pub title: Vec<f32>,
    pub level: Vec<f32>,
    pub clear: Vec<f32>,
    pub clear_bed: Vec<f32>,
    pub over: Vec<f32>,
    pub fx: [Vec<f32>; 10],
}

pub fn clips() -> ClipSet {
    ClipSet {
        title: tune(78.0, &TITLE, Wave::Tri),
        level: tune(108.0, &LEVEL, Wave::Pulse),
        clear: fanfare(),
        clear_bed: tune(72.0, &CLEAR_BED, Wave::Tri),
        over: sting(),
        fx: [jump(), punch(), kick(), lantern(), hit(), hurt(), ko(), zap(), open(), throw()],
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Track {
    Title,
    Level,
    Clear,
    Over,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Audio {
    sink: MixerDeviceSink,
    music: Player,
    track: Option<Track>,
    paused: bool,
    muted: bool,
    level: Vec<f32>,
    title: Vec<f32>,
    clear: Vec<f32>,
    clear_bed: Vec<f32>,
    over: Vec<f32>,
    fx: [Vec<f32>; 10],
}

#[cfg(not(target_arch = "wasm32"))]
impl Audio {
    pub fn open() -> Option<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink().ok()?;
        sink.log_on_drop(false);
        let music = Player::connect_new(sink.mixer());
        music.set_volume(0.85);
        let clips = clips();
        Some(Self {
            sink,
            music,
            track: None,
            paused: false,
            muted: false,
            level: clips.level,
            title: clips.title,
            clear: clips.clear,
            clear_bed: clips.clear_bed,
            over: clips.over,
            fx: clips.fx,
        })
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.music.set_volume(if self.muted { 0.0 } else { 0.85 });
    }

    pub fn sync(&mut self, mode: Mode, cues: Vec<Sfx>) {
        if !self.muted {
            for cue in cues {
                self.play(cue);
            }
        }
        let track = match mode {
            Mode::Title => Track::Title,
            Mode::Play | Mode::Pause => Track::Level,
            Mode::Clear => Track::Clear,
            Mode::Over => Track::Over,
        };
        if self.track != Some(track) {
            self.track = Some(track);
            // `clear` skips whatever is playing, including a loop, then pauses.
            self.music.clear();
            self.music.play();
            match track {
                Track::Title => self.music.append(looped(&self.title)),
                Track::Level => self.music.append(looped(&self.level)),
                Track::Clear => {
                    self.music.append(once(&self.clear));
                    self.music.append(looped(&self.clear_bed));
                }
                Track::Over => self.music.append(once(&self.over)),
            }
            self.paused = false;
        }
        let want_pause = mode == Mode::Pause;
        if want_pause && !self.paused {
            self.music.pause();
            self.paused = true;
        } else if !want_pause && self.paused {
            self.music.play();
            self.paused = false;
        }
    }

    fn play(&self, cue: Sfx) {
        self.sink.mixer().add(once(&self.fx[sfx_index(cue)]));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn once(data: &[f32]) -> SamplesBuffer {
    SamplesBuffer::new(nz!(1), NonZeroU32::new(SR).expect("sample rate"), data.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
fn looped(data: &[f32]) -> impl Source<Item = f32> + Send + 'static {
    once(data).repeat_infinite()
}

#[derive(Clone, Copy)]
enum Wave {
    Pulse,
    Tri,
    Square,
}

/// (lead midi, bass midi, hat). Zero midi is a rest. Each row is an eighth note.
const LEVEL: [(u8, u8, bool); 32] = [
    (76, 45, true), (0, 0, false), (72, 0, true), (0, 0, false),
    (69, 45, true), (72, 0, false), (76, 40, true), (79, 0, false),
    (76, 40, true), (0, 0, false), (74, 0, true), (0, 0, false),
    (72, 43, true), (69, 0, false), (67, 43, true), (64, 0, false),
    (69, 45, true), (72, 0, false), (76, 45, true), (79, 0, false),
    (81, 40, true), (79, 0, true), (76, 40, true), (72, 0, false),
    (74, 43, true), (76, 0, false), (79, 38, true), (76, 0, true),
    (74, 40, true), (72, 0, false), (69, 45, true), (0, 0, false),
];

const TITLE: [(u8, u8, bool); 32] = [
    (81, 33, false), (0, 0, false), (0, 0, false), (76, 0, false),
    (72, 33, false), (0, 0, false), (69, 0, false), (0, 0, false),
    (76, 28, false), (0, 0, false), (72, 0, false), (0, 0, false),
    (69, 31, false), (72, 0, false), (76, 0, false), (0, 0, false),
    (79, 33, false), (0, 0, false), (76, 0, false), (72, 33, false),
    (0, 0, false), (0, 0, false), (69, 28, false), (0, 0, false),
    (67, 31, false), (0, 0, false), (64, 0, false), (0, 0, false),
    (69, 33, false), (0, 0, false), (0, 0, false), (0, 0, false),
];

const CLEAR_BED: [(u8, u8, bool); 16] = [
    (76, 45, false), (0, 0, false), (72, 0, false), (0, 0, false),
    (69, 40, false), (0, 0, false), (72, 0, false), (0, 0, false),
    (76, 43, false), (0, 0, false), (79, 0, false), (0, 0, false),
    (76, 45, false), (0, 0, false), (72, 0, false), (0, 0, false),
];

fn tune(bpm: f32, steps: &[(u8, u8, bool)], lead: Wave) -> Vec<f32> {
    let step = 60.0 / bpm / 2.0;
    let mut buf = vec![0.0; ((steps.len() as f32 * step + 0.05) * SR as f32) as usize];
    let mut rng = 0xA341_316Cu32;
    for (i, &(lead_n, bass_n, hat)) in steps.iter().enumerate() {
        let start = (i as f32 * step * SR as f32) as usize;
        if lead_n > 0 {
            add_osc(&mut buf, start, (step * 0.9 * SR as f32) as usize, freq(lead_n), 0.15, lead);
        }
        if bass_n > 0 {
            add_osc(&mut buf, start, (step * 1.85 * SR as f32) as usize, freq(bass_n), 0.20, Wave::Tri);
        }
        if hat {
            add_noise(&mut buf, start, (0.04 * SR as f32) as usize, 0.07, &mut rng);
        }
    }
    echo(&mut buf, 0.13, 0.16);
    normalize(&mut buf, 0.62);
    buf
}

fn fanfare() -> Vec<f32> {
    let mut buf = vec![0.0; (1.6 * SR as f32) as usize];
    for (i, note) in [69, 72, 76, 81, 84].into_iter().enumerate() {
        let start = (i as f32 * 0.13 * SR as f32) as usize;
        add_osc(&mut buf, start, (0.22 * SR as f32) as usize, freq(note), 0.22, Wave::Pulse);
    }
    let chord = (0.7 * SR as f32) as usize;
    for note in [69, 76, 81] {
        add_osc(&mut buf, chord, (0.8 * SR as f32) as usize, freq(note), 0.10, Wave::Tri);
    }
    normalize(&mut buf, 0.7);
    buf
}

fn sting() -> Vec<f32> {
    let mut buf = vec![0.0; (1.1 * SR as f32) as usize];
    for (i, note) in [64, 62, 60, 57].into_iter().enumerate() {
        let start = (i as f32 * 0.18 * SR as f32) as usize;
        add_osc(&mut buf, start, (0.22 * SR as f32) as usize, freq(note), 0.22, Wave::Square);
    }
    normalize(&mut buf, 0.65);
    buf
}

fn jump() -> Vec<f32> {
    sweep(0.12, 240.0, 680.0, 0.30, Wave::Square)
}

fn punch() -> Vec<f32> {
    let mut buf = thump(0.07, 110.0, 0.40);
    let mut rng = 0x51u32;
    add_noise(&mut buf, 0, (0.045 * SR as f32) as usize, 0.28, &mut rng);
    normalize(&mut buf, 0.75);
    buf
}

fn kick() -> Vec<f32> {
    let mut buf = sweep(0.16, 200.0, 70.0, 0.28, Wave::Square);
    let mut rng = 0x99u32;
    add_noise(&mut buf, 0, (0.07 * SR as f32) as usize, 0.22, &mut rng);
    normalize(&mut buf, 0.75);
    buf
}

fn lantern() -> Vec<f32> {
    let mut buf = vec![0.0; (0.28 * SR as f32) as usize];
    add_osc(&mut buf, 0, (0.12 * SR as f32) as usize, freq(88), 0.22, Wave::Tri);
    add_osc(&mut buf, (0.08 * SR as f32) as usize, (0.16 * SR as f32) as usize, freq(93), 0.22, Wave::Tri);
    normalize(&mut buf, 0.7);
    buf
}

fn hit() -> Vec<f32> {
    let mut buf = thump(0.06, 160.0, 0.35);
    let mut rng = 7u32;
    add_noise(&mut buf, 0, (0.03 * SR as f32) as usize, 0.35, &mut rng);
    normalize(&mut buf, 0.8);
    buf
}

fn hurt() -> Vec<f32> {
    sweep(0.22, 460.0, 120.0, 0.28, Wave::Square)
}

fn ko() -> Vec<f32> {
    let mut buf = sweep(0.32, 300.0, 50.0, 0.30, Wave::Square);
    let mut rng = 3u32;
    add_noise(&mut buf, 0, (0.12 * SR as f32) as usize, 0.25, &mut rng);
    normalize(&mut buf, 0.75);
    buf
}

fn zap() -> Vec<f32> {
    let mut buf = vec![0.0; (0.24 * SR as f32) as usize];
    let n = buf.len();
    let mut rng = 0xBEEFu32;
    add_noise(&mut buf, 0, n, 0.22, &mut rng);
    add_osc(&mut buf, 0, n, 58.0, 0.28, Wave::Square);
    // Chop the buzz so it crackles instead of holding a tone.
    for (i, s) in buf.iter_mut().enumerate() {
        if (i / 80) % 2 == 0 {
            *s *= 0.25;
        }
    }
    normalize(&mut buf, 0.7);
    buf
}

fn open() -> Vec<f32> {
    let mut buf = vec![0.0; (0.42 * SR as f32) as usize];
    add_osc(&mut buf, 0, (0.18 * SR as f32) as usize, freq(69), 0.20, Wave::Pulse);
    add_osc(&mut buf, (0.12 * SR as f32) as usize, (0.26 * SR as f32) as usize, freq(76), 0.20, Wave::Pulse);
    normalize(&mut buf, 0.7);
    buf
}

fn throw() -> Vec<f32> {
    sweep(0.09, 900.0, 1400.0, 0.18, Wave::Square)
}

fn thump(dur: f32, freq_hz: f32, vol: f32) -> Vec<f32> {
    let mut buf = vec![0.0; (dur * SR as f32) as usize + 8];
    let n = buf.len();
    add_osc(&mut buf, 0, n, freq_hz, vol, Wave::Tri);
    buf
}

fn sweep(dur: f32, from: f32, to: f32, vol: f32, wave: Wave) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut buf = vec![0.0; n];
    let mut phase = 0.0;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = from + (to - from) * t;
        phase = (phase + f / SR as f32) % 1.0;
        let env = (1.0 - t).powf(1.4) * t.min(0.08) / 0.08;
        buf[i] += osc(phase, wave) * env * vol;
    }
    normalize(&mut buf, 0.75);
    buf
}

fn add_osc(buf: &mut [f32], start: usize, len: usize, freq_hz: f32, vol: f32, wave: Wave) {
    let mut phase = 0.0;
    let inc = freq_hz / SR as f32;
    for i in 0..len {
        let at = start + i;
        if at >= buf.len() {
            break;
        }
        let t = i as f32 / len as f32;
        let attack = (t / 0.02).min(1.0);
        let release = ((1.0 - t) / 0.12).min(1.0);
        phase = (phase + inc) % 1.0;
        buf[at] += osc(phase, wave) * attack * release * vol;
    }
}

fn add_noise(buf: &mut [f32], start: usize, len: usize, vol: f32, rng: &mut u32) {
    for i in 0..len {
        let at = start + i;
        if at >= buf.len() {
            break;
        }
        let t = i as f32 / len as f32;
        buf[at] += noise(rng) * (1.0 - t).powi(2) * vol;
    }
}

fn osc(phase: f32, wave: Wave) -> f32 {
    match wave {
        Wave::Pulse => {
            if phase < 0.22 {
                0.65
            } else {
                -0.65
            }
        }
        Wave::Square => {
            if phase < 0.5 {
                0.55
            } else {
                -0.55
            }
        }
        Wave::Tri => {
            if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                1.0 - (phase - 0.5) * 4.0
            }
        }
    }
}

fn noise(state: &mut u32) -> f32 {
    *state ^= state.wrapping_shl(13);
    *state ^= state.wrapping_shr(17);
    *state ^= state.wrapping_shl(5);
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn freq(midi: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn echo(buf: &mut [f32], delay_s: f32, mix: f32) {
    let delay = (delay_s * SR as f32) as usize;
    for i in delay..buf.len() {
        buf[i] += buf[i - delay] * mix;
    }
}

fn normalize(buf: &mut [f32], peak: f32) {
    let max = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max > 0.001 {
        let g = peak / max;
        for s in buf {
            *s *= g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_and_effects_are_audible_and_in_range() {
        for buf in [tune(108.0, &LEVEL, Wave::Pulse), tune(78.0, &TITLE, Wave::Tri), fanfare(), sting(), jump(), zap()] {
            assert!(buf.len() > SR as usize / 20, "clip too short");
            let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            assert!(peak > 0.2 && peak <= 1.0, "peak {peak}");
            assert!(buf.iter().any(|s| s.abs() > 0.05));
        }
    }

    #[test]
    fn level_loop_is_a_few_seconds() {
        let buf = tune(108.0, &LEVEL, Wave::Pulse);
        let secs = buf.len() as f32 / SR as f32;
        assert!(secs > 3.0 && secs < 12.0, "{secs}");
    }

    #[test]
    fn clips_match_the_shared_effect_order() {
        let set = clips();
        assert_eq!(sfx_index(Sfx::Throw), 9);
        assert_eq!(set.fx.len(), 10);
        for buf in [&set.title, &set.level, &set.clear, &set.clear_bed, &set.over] {
            let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            assert!(peak > 0.2 && peak <= 1.0, "peak {peak}");
        }
    }
}
