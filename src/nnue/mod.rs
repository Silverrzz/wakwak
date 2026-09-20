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

pub static mut NET: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/wakwak.nnue"))) };

pub const EVAL_SCALE: i32 = 400;
pub const QA: i16 = 255;
pub const QB: i16 = 64;

pub const INPUT: usize = 768;
pub const L1: usize = 16;
pub const HM: bool = false;

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

pub(crate) unsafe fn load_network(path: &str) -> std::io::Result<()> {
    let mut bytes = std::fs::read(path)?;
    let data_size = (INPUT * L1 + L1 + L1 * 2 + 1) * size_of::<i16>();
    if bytes.len() != data_size && bytes.len() != size_of::<Network>() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid NNUE size",
        ));
    }
    bytes.resize(size_of::<Network>(), 0);
    unsafe { NET = bytes.as_ptr().cast::<Network>().read_unaligned() };
    Ok(())
}
