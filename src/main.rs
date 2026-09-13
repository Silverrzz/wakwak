use crate::engine::Engine;

pub mod bench;
pub mod board;
pub mod common;
pub mod engine;
pub mod position;
pub mod score;
pub mod search;
pub mod uci;
pub mod util;

fn main() {
    Engine::new().run()
}
