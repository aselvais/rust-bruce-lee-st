//! Desktop play is the `bruce-lee` binary. The browser build compiles this
//! crate to WebAssembly and draws it from `web/`.

pub mod audio;
pub mod game;
pub mod gfx;
pub mod level;

#[cfg(target_arch = "wasm32")]
mod web;
