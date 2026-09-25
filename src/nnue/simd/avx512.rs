// clippy complains about every #[target_feature] function without safety comment, so we make it shut up
#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;

pub type I16Vec = __m512i;
pub type I32Vec = __m512i;

pub mod i16s {
    use super::*;

    pub const LANES: usize = size_of::<I16Vec>() / size_of::<i16>();

    pub use std::arch::x86_64::{
        _mm512_loadu_epi16 as load, _mm512_madd_epi16 as madd, _mm512_max_epi16 as max,
        _mm512_min_epi16 as min, _mm512_mullo_epi16 as mul, _mm512_set1_epi16 as splat,
        _mm512_storeu_epi16 as store,
    };
}

pub mod i32s {
    use super::*;

    pub const LANES: usize = size_of::<I32Vec>() / size_of::<i32>();

    pub use std::arch::x86_64::{
        _mm512_add_epi32 as add, _mm512_reduce_add_epi32 as reduce_add, _mm512_set1_epi32 as splat,
    };
}
