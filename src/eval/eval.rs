use super::psqt::{EG_PSQT, MG_PSQT};
use crate::board::Board;
use crate::common::{Color, Piece, Square};
use crate::score::Score;

const MG_PIECE_VALUES: [i32; Piece::COUNT] = [82, 337, 365, 477, 1025, 0];
const EG_PIECE_VALUES: [i32; Piece::COUNT] = [94, 281, 297, 512, 936, 0];
const MG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(MG_PIECE_VALUES, MG_PSQT);
const EG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(EG_PIECE_VALUES, EG_PSQT);
const PHASE_WEIGHTS: [i32; Piece::COUNT] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

#[inline]
const fn combine_scores(
    values: [i32; Piece::COUNT],
    mut tables: [[i32; Square::COUNT]; Piece::COUNT],
) -> [[i32; Square::COUNT]; Piece::COUNT] {
    let mut piece = 0;
    while piece < Piece::COUNT {
        let mut square = 0;
        while square < Square::COUNT {
            tables[piece][square] += values[piece];
            square += 1;
        }
        piece += 1;
    }
    tables
}

#[inline]
pub fn eval(board: &Board) -> Score {
    let us = board.stm();
    let them = !us;
    let (us_mg, us_eg, us_phase) = side_score(board, us);
    let (them_mg, them_eg, them_phase) = side_score(board, them);
    let phase = (us_phase + them_phase).min(MAX_PHASE);
    ((us_mg - them_mg) * phase + (us_eg - them_eg) * (MAX_PHASE - phase)) / MAX_PHASE
}

#[inline]
fn side_score(board: &Board, color: Color) -> (Score, Score, i32) {
    let mut mg = Score::ZERO;
    let mut eg = Score::ZERO;
    let mut phase = 0;
    for &piece in Piece::ALL {
        for square in board.colored_pieces(color, piece) {
            let square = square.relative_to(color);
            mg += MG_PIECE_SCORES[piece][square];
            eg += EG_PIECE_SCORES[piece][square];
            phase += PHASE_WEIGHTS[piece];
        }
    }
    (mg, eg, phase)
}
