// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::wasm32::*;

pub type I16Vec = v128;
pub type I32Vec = v128;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    #[target_feature(enable = "simd128")]
    pub fn splat(n: i16) -> I16Vec {
        i16x8_splat(n)
    }

    #[target_feature(enable = "simd128")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { v128_load(ptr.cast()) }
    }

    #[target_feature(enable = "simd128")]
    pub unsafe fn store(ptr: *mut i16, v: I16Vec) {
        unsafe { v128_store(ptr.cast(), v) }
    }

    #[target_feature(enable = "simd128")]
    pub fn min(a: I16Vec, b: I16Vec) -> I16Vec {
        i16x8_min(a, b)
    }

    #[target_feature(enable = "simd128")]
    pub fn max(a: I16Vec, b: I16Vec) -> I16Vec {
        i16x8_max(a, b)
    }

    #[target_feature(enable = "simd128")]
    pub fn mul(a: I16Vec, b: I16Vec) -> I16Vec {
        i16x8_mul(a, b)
    }

    #[target_feature(enable = "simd128")]
    pub fn madd(a: I16Vec, b: I16Vec) -> I32Vec {
        let lo = i32x4_extmul_low_i16x8(a, b);
        let hi = i32x4_extmul_high_i16x8(a, b);
        i32x4_add(lo, hi)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    #[target_feature(enable = "simd128")]
    pub fn splat(n: i32) -> I32Vec {
        i32x4_splat(n)
    }

    #[target_feature(enable = "simd128")]
    pub fn add(a: I32Vec, b: I32Vec) -> I32Vec {
        i32x4_add(a, b)
    }

    #[target_feature(enable = "simd128")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        let v64 = i32x4_add(v, i32x4_shuffle::<2, 3, 0, 1>(v, v));
        let v32 = i32x4_add(v64, i32x4_shuffle::<1, 0, 3, 2>(v64, v64));
        i32x4_extract_lane::<0>(v32)
    }
}
