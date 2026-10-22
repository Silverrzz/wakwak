// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;

pub type I8Vec = __m512i;
pub type I16Vec = __m512i;
pub type I32Vec = __m512i;

pub mod i8s {
    use super::*;

    pub const LANES: usize = size_of::<I8Vec>() / size_of::<i8>();

    pub use std::arch::x86_64::{_mm512_loadu_epi8 as load, _mm512_storeu_epi8 as store};

    #[target_feature(enable = "avx512bw")]
    pub fn dpbusd(acc: I32Vec, l: I8Vec, r: I8Vec) -> I32Vec {
        cfg_select! {
            target_feature = "avx512vnni" => unsafe { _mm512_dpbusd_epi32(acc, l, r) },
            _ => _mm512_add_epi32(
                acc,
                _mm512_madd_epi16(_mm512_maddubs_epi16(l, r), _mm512_set1_epi16(1)),
            ),
        }
    }
}

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    pub use std::arch::x86_64::{
        _mm512_loadu_epi16 as load, _mm512_max_epi16 as max, _mm512_min_epi16 as min,
        _mm512_mullo_epi16 as mul, _mm512_set1_epi16 as splat, _mm512_storeu_epi16 as store,
    };

    #[target_feature(enable = "avx512bw")]
    pub fn mulhi_shl7(l: I16Vec, r: I16Vec) -> I16Vec {
        _mm512_mulhi_epi16(l, _mm512_slli_epi16(r, 7))
    }

    #[target_feature(enable = "avx512bw")]
    pub fn packus(l: I16Vec, r: I16Vec) -> I8Vec {
        let lo = _mm512_shuffle_i64x2(l, r, 136);
        let hi = _mm512_shuffle_i64x2(l, r, 221);
        _mm512_packus_epi16(lo, hi)
    }
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    pub use std::{
        arch::x86_64::{
            _mm512_add_epi32 as add, _mm512_loadu_epi32 as load, _mm512_max_epi32 as max,
            _mm512_min_epi32 as min, _mm512_mullo_epi32 as mul,
            _mm512_reduce_add_epi32 as reduce_add, _mm512_set1_epi32 as splat,
            _mm512_srai_epi32 as shr_const, _mm512_storeu_epi32 as store,
        },
        convert::identity as reinterpret_i8,
    };
}
