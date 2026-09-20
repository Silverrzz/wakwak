pub mod accumulator;
pub mod feature;
pub mod inference;
#[allow(clippy::module_inception)]
pub mod nnue;

pub use accumulator::*;
pub use feature::*;
pub use inference::*;
pub use nnue::*;

use crate::common::{File, Square};

pub static NET: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/wakwak.nnue"))) };

pub const EVAL_SCALE: i32 = 400;
pub const QA: i16 = 255;
pub const QB: i16 = 64;

pub const INPUT: usize = 768;
pub const L1: usize = 64;
pub const HM: bool = true;

#[inline]
pub fn should_mirror(sq: Square) -> bool {
    sq.file() > File::D
}

#[repr(C, align(64))]
pub struct Network {
    pub ft_weights: [[i16; L1]; INPUT],
    pub ft_bias: [i16; L1],
    pub out_weights: [i16; L1 * 2],
    pub out_bias: i16,
}
