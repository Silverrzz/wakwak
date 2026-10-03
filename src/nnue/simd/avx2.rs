// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;

pub type I8Vec = __m256i;
pub type I16Vec = __m256i;
pub type I32Vec = __m256i;

pub mod i8s {
    use super::*;

    pub const LANES: usize = size_of::<I8Vec>() / size_of::<i8>();

    #[target_feature(enable = "avx2")]
    pub unsafe fn load(ptr: *const i8) -> I8Vec {
        unsafe { _mm256_loadu_si256(ptr.cast()) }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn store(ptr: *mut i8, v: I8Vec) {
        unsafe { _mm256_storeu_si256(ptr.cast(), v) }
    }

    #[target_feature(enable = "avx2")]
    pub fn dpbusd(acc: I32Vec, l: I8Vec, r: I8Vec) -> I32Vec {
        cfg_select! {
            target_feature = "avxvnni" => unsafe { _mm256_dpbusd_avx_epi32(acc, l, r) },
            _ => _mm256_add_epi32(
                acc,
                _mm256_madd_epi16(_mm256_maddubs_epi16(l, r), _mm256_set1_epi16(1)),
            ),
        }
    }
}

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();
    pub use std::arch::x86_64::{
        _mm256_max_epi16 as max, _mm256_min_epi16 as min, _mm256_mullo_epi16 as mul,
        _mm256_set1_epi16 as splat,
    };

    #[target_feature(enable = "avx2")]
    pub unsafe fn load(ptr: *const i16) -> I16Vec {
        unsafe { _mm256_loadu_si256(ptr.cast()) }
    }

    #[target_feature(enable = "avx2")]
    pub fn mulhi_shl7(l: I16Vec, r: I16Vec) -> I16Vec {
        _mm256_mulhi_epi16(l, _mm256_slli_epi16(r, 7))
    }
    #[target_feature(enable = "avx2")]
    pub fn packus(l: I16Vec, r: I16Vec) -> I8Vec {
        _mm256_permute4x64_epi64(_mm256_packs_epi16(l, r), 0xd8)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    pub use std::{
        arch::x86_64::{
            _mm256_add_epi32 as add, _mm256_max_epi32 as max, _mm256_min_epi32 as min,
            _mm256_mullo_epi32 as mul, _mm256_set1_epi32 as splat, _mm256_srai_epi32 as shr_const,
        },
        convert::identity as reinterpret_i8,
    };

    #[target_feature(enable = "avx2")]
    pub fn reduce_add(v: I32Vec) -> i32 {
        let v128 = _mm_add_epi32(_mm256_castsi256_si128(v), _mm256_extracti128_si256(v, 1));
        let v64 = _mm_add_epi32(v128, _mm_shuffle_epi32(v128, 0b01_00_11_10));
        let v32 = _mm_add_epi32(v64, _mm_shuffle_epi32(v64, 0b10_11_00_01));
        _mm_cvtsi128_si32(v32)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn load(ptr: *const i32) -> I32Vec {
        unsafe { _mm256_loadu_si256(ptr.cast()) }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn store(ptr: *mut i32, v: I32Vec) {
        unsafe { _mm256_storeu_si256(ptr.cast(), v) }
    }
}
