use crate::engine::Engine;

pub mod board;
pub mod common;
pub mod engine;
pub mod position;
pub mod uci;
pub mod util;

fn main() {
    Engine::new().run()
}
