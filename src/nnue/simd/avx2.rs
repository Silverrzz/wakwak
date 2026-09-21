// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;

pub type I16Vec = __m256i;
pub type I32Vec = __m256i;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    #[target_feature(enable = "avx2")]
    pub fn splat(n: i16) -> I16Vec {
        _mm256_set1_epi16(n)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { _mm256_loadu_si256(ptr.cast()) }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn store(ptr: *mut i16, v: I16Vec) {
        unsafe { _mm256_storeu_si256(ptr.cast(), v) }
    }

    #[target_feature(enable = "avx2")]
    pub fn min(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm256_min_epi16(a, b)
    }

    #[target_feature(enable = "avx2")]
    pub fn max(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm256_max_epi16(a, b)
    }

    #[target_feature(enable = "avx2")]
    pub fn mul(a: I16Vec, b: I16Vec) -> I16Vec {
        _mm256_mullo_epi16(a, b)
    }

    #[target_feature(enable = "avx2")]
    pub fn madd(a: I16Vec, b: I16Vec) -> I32Vec {
        _mm256_madd_epi16(a, b)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    #[target_feature(enable = "avx2")]
    pub fn splat(n: i32) -> I32Vec {
        _mm256_set1_epi32(n)
    }

    #[target_feature(enable = "avx2")]
    pub fn add(a: I32Vec, b: I32Vec) -> I32Vec {
        _mm256_add_epi32(a, b)
    }

    #[target_feature(enable = "avx2")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        let v128 = _mm_add_epi32(_mm256_castsi256_si128(v), _mm256_extracti128_si256(v, 1));
        let v64 = _mm_add_epi32(v128, _mm_shuffle_epi32(v128, 0b01_00_11_10));
        let v32 = _mm_add_epi32(v64, _mm_shuffle_epi32(v64, 0b10_11_00_01));
        _mm_cvtsi128_si32(v32)
    }
}
