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
        let weights = &NET.l0w;
        let previous = &prev.values[perspective];
        let updates = prev.updates;
        let index = |feature: PieceFeature| feature.to_index(king, perspective);

        match (updates.add, updates.add2, updates.sub, updates.sub2) {
            (Some(add), None, Some(sub), None) => {
                acc_add_sub(previous, values, weights, index(add), index(sub));
            }
            (Some(add), None, Some(sub), Some(sub2)) => {
                acc_add_sub2(
                    previous,
                    values,
                    weights,
                    index(add),
                    index(sub),
                    index(sub2),
                );
            }
            (Some(add), Some(add2), Some(sub), Some(sub2)) => acc_add2_sub2(
                previous,
                values,
                weights,
                index(add),
                index(add2),
                index(sub),
                index(sub2),
            ),
            _ => unreachable!("Invalid accumulator updates"),
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

        let weights = &NET.l0w;
        let (chunks, rem) = adds.as_chunks();
        let values = &mut self.values[perspective];
        *values = NET.l0b;

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
            values: enum_map! { _ => NET.l0b },
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

pub fn acc_add_sub(
    src: &[i16; L1],
    dst: &mut [i16; L1],
    weights: &[[i16; L1]; INPUT],
    add: usize,
    sub: usize,
) {
    let add: &[i16; L1] = &weights[add];
    let sub: &[i16; L1] = &weights[sub];

    for i in 0..L1 {
        dst[i] = src[i] + add[i] - sub[i];
    }
}

pub fn acc_add_sub2(
    src: &[i16; L1],
    dst: &mut [i16; L1],
    weights: &[[i16; L1]; INPUT],
    add: usize,
    sub1: usize,
    sub2: usize,
) {
    let add: &[i16; L1] = &weights[add];
    let sub1: &[i16; L1] = &weights[sub1];
    let sub2: &[i16; L1] = &weights[sub2];

    for i in 0..L1 {
        dst[i] = src[i] + add[i] - sub1[i] - sub2[i];
    }
}

pub fn acc_add2_sub2(
    src: &[i16; L1],
    dst: &mut [i16; L1],
    weights: &[[i16; L1]; INPUT],
    add1: usize,
    add2: usize,
    sub1: usize,
    sub2: usize,
) {
    let add1: &[i16; L1] = &weights[add1];
    let add2: &[i16; L1] = &weights[add2];
    let sub1: &[i16; L1] = &weights[sub1];
    let sub2: &[i16; L1] = &weights[sub2];

    for i in 0..L1 {
        dst[i] = src[i] + add1[i] + add2[i] - sub1[i] - sub2[i];
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
