// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::aarch64::*;

pub type I16Vec = int16x8_t;
pub type I32Vec = int32x4_t;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    #[target_feature(enable = "neon")]
    pub fn splat(n: i16) -> I16Vec {
        vdupq_n_s16(n)
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { vld1q_s16(ptr) }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn store(ptr: *mut i16, v: I16Vec) {
        unsafe {
            vst1q_s16(ptr, v);
        }
    }

    #[target_feature(enable = "neon")]
    pub fn min(a: I16Vec, b: I16Vec) -> I16Vec {
        vminq_s16(a, b)
    }

    #[target_feature(enable = "neon")]
    pub fn max(a: I16Vec, b: I16Vec) -> I16Vec {
        vmaxq_s16(a, b)
    }

    #[target_feature(enable = "neon")]
    pub fn mul(a: I16Vec, b: I16Vec) -> I16Vec {
        vmulq_s16(a, b)
    }

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

    #[target_feature(enable = "neon")]
    pub fn splat(n: i32) -> I32Vec {
        vdupq_n_s32(n)
    }

    #[target_feature(enable = "neon")]
    pub fn add(a: I32Vec, b: I32Vec) -> I32Vec {
        vaddq_s32(a, b)
    }

    #[target_feature(enable = "neon")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        vaddvq_s32(v)
    }
}
