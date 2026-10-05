#[cfg(target_os = "emscripten")]
extern crate atomic_wait_wasm as atomic_wait;

#[cfg(not(target_os = "emscripten"))]
use crate::engine::Engine;

#[cfg(target_os = "emscripten")]
mod browser;

pub mod bench;
pub mod board;
pub mod common;
pub mod engine;
pub mod genfens;
pub mod nnue;
pub mod position;
pub mod score;
pub mod search;
pub mod tools;
pub mod uci;
pub mod util;

#[cfg(not(target_os = "emscripten"))]
fn main() -> std::io::Result<()> {
    Engine::new().run();
    Ok(())
}

#[cfg(target_os = "emscripten")]
fn main() {
    browser::run()
}
