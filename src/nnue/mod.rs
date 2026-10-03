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

cfg_select! {
    target_feature = "avx512bw" => {
        #[path = "simd/avx512.rs"]
        pub mod simd;
    }
    target_feature = "avx2" => {
        #[path = "simd/avx2.rs"]
        pub mod simd;
    }
    target_feature = "neon" => {
        #[path = "simd/neon.rs"]
        pub mod simd;
    }
    _ => {
        compile_error!(
            "Unsupported platform! Only AVX2 or newer (on x86) and Neon (on ARM) are supported"
        );
    }
}

pub static NET: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/wakwak.nnue"))) };

pub const Q0: i16 = 255;
pub const _Q1: i16 = 128;
pub const Q: i32 = 64;
pub const EVAL_SCALE: i64 = 400;

pub const INPUT: usize = 768;
pub const L1: usize = 768;
pub const L2: usize = 16;
pub const L3: usize = 32;
pub const HM: bool = true;

#[inline]
pub fn should_mirror(sq: Square) -> bool {
    sq.file() > File::D
}

#[repr(C, align(64))]
pub struct Network {
    pub l0w: [[i16; L1]; INPUT],
    pub l0b: [i16; L1],
    pub l1w: [[i8; L2 * 4]; L1 / 4],
    pub l1b: [i32; L2],
    pub l2w: [[i32; L3]; L2 * 2],
    pub l2b: [i32; L3],
    pub l3w: [i32; L3],
    pub l3b: i32,
}
