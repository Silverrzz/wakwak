// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::aarch64::*;

pub type I16Vec = int16x8_t;
pub type I32Vec = int32x4_t;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    pub use std::arch::aarch64::{
        vdupq_n_s16 as splat, vld1q_s16 as load, vmaxq_s16 as max, vminq_s16 as min,
        vmulq_s16 as mul, vst1q_s16 as store,
    };

    #[target_feature(enable = "neon")]
    pub fn madd(a: I16Vec, b: I16Vec) -> I32Vec {
        let lo = vmull_s16(vget_low_s16(a), vget_low_s16(b));
        let hi = vmull_high_s16(a, b);
        vpaddq_s32(lo, hi)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    pub use std::arch::aarch64::{
        vaddq_s32 as add, vaddvq_s32 as reduce_add, vdupq_n_s32 as splat,
    };
}
