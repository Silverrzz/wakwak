use crate::board::Board;
use crate::common::{Bitboard, Color, Square};
use crate::nnue::{FeatureUpdates, INPUT, L1, NET, PieceFeature};
use arrayvec::ArrayVec;
use enum_map::{EnumMap, enum_map};

#[derive(Debug, Clone)]
pub struct Accumulator {
    pub values: EnumMap<Color, [i16; L1]>,
    pub needs_refresh: EnumMap<Color, bool>,
    pub dirty: EnumMap<Color, bool>,
    pub updates: FeatureUpdates,
}

impl Accumulator {
    #[inline]
    pub fn update(&mut self, prev: &Accumulator, king: Square, perspective: Color) {
        let values = &mut self.values[perspective];
        let weights = &NET.ft_weights;
        let (adds, subs) = prev.updates.to_indices(king, perspective);

        *values = prev.values[perspective];
        for i in adds {
            acc_add(values, weights, i);
        }

        for i in subs {
            acc_sub(values, weights, i);
        }

        self.dirty[perspective] = false;
    }

    #[inline]
    pub fn reset(&mut self, board: &Board, perspective: Color) {
        let king = board.king(perspective);
        let mut adds: ArrayVec<usize, 33> = ArrayVec::new();

        for sq in board.occupied() ^ board.duck().map_or(Bitboard::EMPTY, Square::bitboard) {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            adds.push(PieceFeature::new(piece, color, sq).to_index(king, perspective));
        }

        /*if let Some(sq) = board.duck() {
            adds.push(DuckFeature(sq).to_index(king, perspective));
        }*/

        let weights = &NET.ft_weights;
        let (chunks, rem) = adds.as_chunks();
        let values = &mut self.values[perspective];
        *values = NET.ft_bias;

        for &[add1, add2, add3, add4] in chunks {
            acc_add4(values, weights, add1, add2, add3, add4);
        }

        for &add in rem {
            acc_add(values, weights, add);
        }

        self.dirty[perspective] = false;
        self.needs_refresh[perspective] = false;
    }
}

impl Default for Accumulator {
    #[inline]
    fn default() -> Self {
        Self {
            values: enum_map! { _ => NET.ft_bias },
            dirty: enum_map! { _ => false },
            needs_refresh: enum_map! { _ => false },
            updates: FeatureUpdates::default(),
        }
    }
}

#[inline]
fn acc_add(values: &mut [i16; L1], weights: &[[i16; L1]; INPUT], add: usize) {
    let add = &weights[add];
    for i in 0..L1 {
        values[i] += add[i];
    }
}

#[inline]
fn acc_sub(values: &mut [i16; L1], weights: &[[i16; L1]; INPUT], sub: usize) {
    let sub = &weights[sub];
    for i in 0..L1 {
        values[i] -= sub[i];
    }
}

#[inline]
fn acc_add4(
    values: &mut [i16; L1],
    weights: &[[i16; L1]; INPUT],
    add1: usize,
    add2: usize,
    add3: usize,
    add4: usize,
) {
    let add1 = &weights[add1];
    let add2 = &weights[add2];
    let add3 = &weights[add3];
    let add4 = &weights[add4];

    for i in 0..L1 {
        values[i] += add1[i] + add2[i] + add3[i] + add4[i];
    }
}
