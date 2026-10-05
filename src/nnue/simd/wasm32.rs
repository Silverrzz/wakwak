// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::wasm32::*;

pub type I8Vec = v128;
pub type I16Vec = v128;
pub type I32Vec = v128;

pub mod i8s {
    use super::*;

    pub const LANES: usize = size_of::<I8Vec>() / size_of::<i8>();

    #[target_feature(enable = "simd128")]
    pub unsafe fn load(ptr: *const i8) -> I8Vec {
        unsafe { v128_load(ptr.cast()) }
    }

    #[target_feature(enable = "simd128")]
    pub unsafe fn store(ptr: *mut i8, v: I8Vec) {
        unsafe { v128_store(ptr.cast(), v) }
    }

    #[target_feature(enable = "relaxed-simd")]
    pub fn dpbusd(acc: I32Vec, l: I8Vec, r: I8Vec) -> I32Vec {
        i32x4_relaxed_dot_i8x16_i7x16_add(r, l, acc)
    }
}

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    pub use std::arch::wasm32::{
        i16x8_max as max, i16x8_min as min, i16x8_mul as mul, i16x8_splat as splat,
        u8x16_narrow_i16x8 as packus,
    };

    #[target_feature(enable = "simd128")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { v128_load(ptr.cast()) }
    }

    #[target_feature(enable = "simd128")]
    pub unsafe fn store(ptr: *mut i16, v: I16Vec) {
        unsafe { v128_store(ptr.cast(), v) }
    }

    #[target_feature(enable = "simd128")]
    pub fn mulhi_shl7(l: I16Vec, r: I16Vec) -> I16Vec {
        let r = i16x8_shl(r, 7);
        let lo = u32x4_shr(i32x4_extmul_low_i16x8(l, r), 16);
        let hi = u32x4_shr(i32x4_extmul_high_i16x8(l, r), 16);
        u16x8_narrow_i32x4(lo, hi)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    pub use std::{
        arch::wasm32::{
            i32x4_add as add, i32x4_max as max, i32x4_min as min, i32x4_mul as mul,
            i32x4_splat as splat,
        },
        convert::identity as reinterpret_i8,
    };

    #[target_feature(enable = "simd128")]
    pub unsafe fn load(ptr: *const i32) -> I32Vec {
        unsafe { v128_load(ptr.cast()) }
    }

    #[target_feature(enable = "simd128")]
    pub unsafe fn store(ptr: *mut i32, v: I32Vec) {
        unsafe { v128_store(ptr.cast(), v) }
    }

    #[target_feature(enable = "simd128")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        let v64 = i32x4_add(v, i32x4_shuffle::<2, 3, 0, 1>(v, v));
        let v32 = i32x4_add(v64, i32x4_shuffle::<1, 0, 3, 2>(v64, v64));
        i32x4_extract_lane::<0>(v32)
    }

    #[target_feature(enable = "simd128")]
    pub fn shr_const<const SHIFT: u32>(v: I32Vec) -> I32Vec {
        i32x4_shr(v, SHIFT)
    }
}
