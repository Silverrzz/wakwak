use crate::nnue::{EVAL_SCALE, L1, NET, QA, QB};

#[inline]
pub fn feed_forward(stm: &[i16; L1], ntm: &[i16; L1]) -> i32 {
    let mut output;

    unsafe {
        use super::simd::{i16s, i32s};

        let mut sums = i32s::splat(0);

        let zero = i16s::splat(0);
        let qa = i16s::splat(QA);

        for i in (0..L1).step_by(i16s::LANES) {
            let us = i16s::load(stm.as_ptr().add(i));
            let them = i16s::load(ntm.as_ptr().add(i));

            let us_weights = i16s::load(NET.out_weights.as_ptr().add(i));
            let them_weights = i16s::load(NET.out_weights.as_ptr().add(i + L1));

            let us_clamped = i16s::min(i16s::max(us, zero), qa);
            let them_clamped = i16s::min(i16s::max(them, zero), qa);

            let us_results = i16s::madd(i16s::mul(us_weights, us_clamped), us_clamped);
            let them_results = i16s::madd(i16s::mul(them_weights, them_clamped), them_clamped);

            sums = i32s::add(sums, us_results);
            sums = i32s::add(sums, them_results);
        }

        output = i32s::reduce_add(sums);
    }

    output /= QA as i32;
    output += NET.out_bias as i32;
    output *= EVAL_SCALE;
    output /= QA as i32 * QB as i32;

    output
}
