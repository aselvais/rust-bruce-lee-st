# Bruce Lee — Level 1

An unofficial pixel-art tribute to Datasoft's 1984 *Bruce Lee*. This build is Level 1 only: the three opening chambers of the fortress. The sprites, rooms, and code are original. It is not affiliated with the Bruce Lee estate or with Datasoft.

The game is a Rust binary. Art is compiled into the executable, so a release build is one file. There is no installer and no audio.

## Requirements

- [Rust](https://rustup.rs/) with `cargo`. This project was built with Rust 1.99 (edition 2021). Any recent stable toolchain is enough.
- A display. The game uses a normal desktop window (`minifb`).

Python is only needed if you regenerate the pixel art. It is not needed to play or to build a release.

## Play

From this folder:

```
cargo run
```

The first run compiles the game and its dependencies. Later runs start quickly.

The window is 1280×720: a 640×360 pixel-art screen, scaled 2× with nearest-neighbor pixels. On a smaller display:

```
cargo run -- --1
```

That opens a 640×360 window. Click the window once so the keys reach the game.

| Key | Action |
| --- | --- |
| Left / Right, or A / D | Run |
| Space | Jump |
| Up or W, on a ladder or vine | Climb |
| Up or W, away from a ladder or vine | Jump |
| Left / Right while climbing | Step off onto a nearby ledge |
| Down or S | Duck. The ninja's saber and shurikens pass overhead. |
| Z, X, J, or K | Punch while standing still. Flying kick while running or in the air. |
| P | Pause. The music pauses with the game. |
| M | Mute or unmute music and effects |
| Enter | Start, or play again after the clear screen or game over |
| Esc | Quit |
| F1 | Toggle a thin debug outline on the player |

## Sound

The music and effects are original and synthesized when the game starts. They are not recordings from the 1984 release, and there are no audio files to copy next to the program.

A looping theme plays on the title screen and another through the chambers. The clear screen and game over each get their own phrase. Effects cover the jump, punch, kick, shuriken, lantern, hit, defeat, electric floor, and a passage opening.

If the computer has no audio device, the game prints a line and keeps running with the sound off.

## Release builds

A release build turns on compiler optimizations. The result is a single program. You do not copy `assets/` next to it.

### Mac

On the Mac you want the game to run on:

```
cd /Users/aselvais/projects/games/rust-bruce-lee-st
cargo build --release
```

The program is:

```
target/release/bruce-lee
```

Run it:

```
./target/release/bruce-lee
```

The same flags work on the release binary:

```
./target/release/bruce-lee --1
./target/release/bruce-lee --shots
```

`--shots` writes the six stills in `shots/` and does not open a window. Run it from the project folder so `shots/` is created there.

The binary matches the Mac you compiled on. On this machine that is Apple silicon (`arm64`). An Intel Mac needs its own `cargo build --release` on that Mac. To ship one file that runs on both, build each target and join them:

```
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
mkdir -p dist
lipo -create \
  target/aarch64-apple-darwin/release/bruce-lee \
  target/x86_64-apple-darwin/release/bruce-lee \
  -output dist/bruce-lee
```

`dist/bruce-lee` is the universal Mac binary. A binary downloaded from the internet may be blocked by Gatekeeper until you right-click it and choose Open, or clear the quarantine flag with `xattr -d com.apple.quarantine dist/bruce-lee`. A binary you just compiled locally does not need that.

### PC (Windows)

Build the Windows release on Windows. Install Rust from <https://rustup.rs/> (the default MSVC toolchain), open a terminal in this folder, and run:

```
cargo build --release
```

The program is:

```
target\release\bruce-lee.exe
```

Double-click `bruce-lee.exe`, or from that folder:

```
.\target\release\bruce-lee.exe
.\target\release\bruce-lee.exe --1
```

Copy `bruce-lee.exe` anywhere. It does not need `assets\`, a DLL folder, or the Rust toolchain on the machine that only plays the game.

This repository is developed on macOS. The Windows executable is produced by the command above on a Windows PC. Both releases are the same source and the same embedded art.

## Level 1

Three single-screen chambers. Walk through an open door and the next room slides in.

1. **Gate Court.** Guardian statue, center vine, six lanterns. You start at the lower left. The east door opens when every lantern in this room is taken.
2. **Middle Hall.** Ladders on both sides, a center vine, six lanterns. The left door is open once the court is clear. The right door opens when this hall is clear. The floor hatch in the middle opens only after all 18 lanterns are taken. Drop through it to finish the level.
3. **East Gallery.** High bridge, side ladders, six lanterns, and a timed electric floor. There is no door past this room. Take the lanterns, return to the hall, and use the hatch.

The electric floor runs on a 2.45 second cycle. It is dark, then it flashes a warning, then it is live. Cross while it is dark, or stay on the ladders and the upper platforms. Standing on it while it is live costs one pip of chi.

The black ninja takes two hits. He slashes with a saber up close and throws shurikens from farther away. Duck and both go over you. Green Yamo, the sumo, takes three hits. Both return after a few seconds, and they can hit each other. A shuriken that hits Yamo hurts him. Touching either fighter costs chi. A fall does not.

You have four chi pips and five lives. When chi reaches zero you lose a life and return to the left side of the current room. At zero lives the run ends. Enter starts again.

### Score

| Event | Points |
| --- | --- |
| Lantern | 125 |
| Punch | 100 |
| Flying kick | 75 |
| Stomp (land on an enemy) | 50 |
| Ninja defeated | the hit, plus 200 |
| Green Yamo defeated | the hit, plus 450 |
| First visit to a new chamber | 2000 |
| Level clear | 2000 |

The HUD shows the score, the chamber name, lanterns taken in the current room, chi, and lives remaining.

## Project layout

```
Cargo.toml          package bruce-lee, edition 2021
Cargo.lock          locked dependency versions
src/main.rs         window loop, keys, screenshot writer
src/game.rs         rooms, combat, score, title and clear screens
src/level.rs        maps, physics, doors, hatch, electric floor
src/gfx.rs          framebuffer, sprite blit, pixel font
assets/             PNG sprites and the three room backgrounds
tools/gen_assets.py redraws the PNGs in assets/
shots/              stills from --shots
LICENSE             MIT, Copyright (c) 2026 Arnaud Selvais
```

Internal resolution is 640×360. The top 24 rows are the HUD. The playfield is 40×21 tiles of 16 pixels (336 pixels tall).

Sprites and backgrounds are loaded with `include_bytes!` in `src/gfx.rs`. Changing a PNG requires a rebuild before the game shows it.

To redraw the art (Python 3 and Pillow):

```
python3 tools/gen_assets.py
cargo build
```

## Tests

```
cargo test
```

The tests check that a fall lands on the court floor, that a jump clears a one-way ledge, and that every lantern can be reached from the left spawn.

## Dependencies

| Crate | Use |
| --- | --- |
| `minifb` 0.27 | Window, keyboard, 60 fps |
| `image` 0.25 | PNG decode for embedded art, and writing `--shots` |

Both are fetched by Cargo. No system libraries need a separate install on macOS or on Windows with the default Rustup toolchain.
