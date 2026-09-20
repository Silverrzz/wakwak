use crate::nnue::{EVAL_SCALE, L1, NET, QA, QB};

#[inline]
pub fn feed_forward(stm: &[i16; L1], ntm: &[i16; L1]) -> i32 {
    let mut output = 0;
    for (&stm, &weight) in stm.iter().zip(&NET.out_weights[..L1]) {
        let stm_clamped = stm.clamp(0, QA);
        output += i32::from(stm_clamped * weight) * i32::from(stm_clamped);
    }
    for (&ntm, &weight) in ntm.iter().zip(&NET.out_weights[L1..]) {
        let ntm_clamped = ntm.clamp(0, QA);
        output += i32::from(ntm_clamped * weight) * i32::from(ntm_clamped);
    }

    output /= QA as i32;
    output += NET.out_bias as i32;
    output *= EVAL_SCALE;
    output /= QA as i32 * QB as i32;

    output
}
