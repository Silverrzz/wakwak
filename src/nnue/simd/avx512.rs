// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;

pub type I16Vec = __m512i;
pub type I32Vec = __m512i;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    #[target_feature(enable = "avx512f")]
    pub fn splat(n: i16) -> I16Vec {
        _mm512_set1_epi16(n)
    }

    #[target_feature(enable = "avx512f")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { _mm512_loadu_epi16(ptr.cast()) }
    }

    #[target_feature(enable = "avx512f")]
    pub unsafe fn store(ptr: *mut i16, v: I16Vec) {
        unsafe { _mm512_storeu_epi16(ptr.cast(), v) }
    }

    #[target_feature(enable = "avx512bw")]
    pub fn min(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm512_min_epi16(a, b)
    }

    #[target_feature(enable = "avx512bw")]
    pub fn max(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm512_max_epi16(a, b)
    }

    #[target_feature(enable = "avx512bw")]
    pub fn mul(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm512_mullo_epi16(a, b)
    }

    #[target_feature(enable = "avx512bw")]
    pub fn madd(a: I16Vec, b: I16Vec) -> I32Vec {
        _mm512_madd_epi16(a, b)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    #[target_feature(enable = "avx512f")]
    pub fn splat(n: i32) -> I32Vec {
        _mm512_set1_epi32(n)
    }

    #[target_feature(enable = "avx512f")]
    pub fn add(a: I32Vec, b: I32Vec) -> I32Vec {
        _mm512_add_epi32(a, b)
    }

    #[target_feature(enable = "avx512f")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        _mm512_reduce_add_epi32(v)
    }
}
