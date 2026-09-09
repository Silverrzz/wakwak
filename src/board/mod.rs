#[allow(clippy::module_inception)]
pub mod board;
pub mod castling;
pub mod display;
pub mod en_passant;
pub mod fen;
pub mod make_move;
pub mod movegen;
pub mod perft;
pub mod sliders;
pub mod startpos;
pub mod zobrist;
pub mod search;

pub use board::*;
pub use castling::*;
pub use en_passant::*;
pub use zobrist::*;
