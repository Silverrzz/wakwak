use super::psqt::{EG_PSQT, MG_PSQT};
use crate::board::{Board, bishop_attacks, rook_attacks};
use crate::common::{Color, Piece, Square, bitboard::Bitboard};
use crate::score::Score;

const MG_PIECE_VALUES: [i32; Piece::COUNT] = [82, 337, 365, 477, 1025, 0];
const EG_PIECE_VALUES: [i32; Piece::COUNT] = [94, 281, 297, 512, 936, 0];
const MG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(MG_PIECE_VALUES, MG_PSQT);
const EG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(EG_PIECE_VALUES, EG_PSQT);
const PHASE_WEIGHTS: [i32; Piece::COUNT] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

const KING_SCORE_FACTOR: i32 = 2;
const CENTERED_EXPOSURE: i32 = 18;

#[inline]
pub fn eval(board: &Board) -> Score {
    let us = board.stm();
    let them = !us;
    let (us_mg, us_eg, us_phase) = side_score(board, us);
    let (them_mg, them_eg, them_phase) = side_score(board, them);
    let phase = (us_phase + them_phase).min(MAX_PHASE);
    let us_king_score = king_score(board, us);
    let them_king_score = king_score(board, them);
    ((us_mg - them_mg) * phase + (us_eg - them_eg) * (MAX_PHASE - phase)) / MAX_PHASE
        + us_king_score
        - them_king_score
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
fn king_score(board: &Board, color: Color) -> i32 {
    let king_bb = board.pieces(Piece::King) & board.colors(color);
    let king_sq = king_bb.next();
    let valid_dest = !board.colors(color) & !board.duck().map_or(Bitboard::EMPTY, Square::bitboard);
    let exposure_bb = valid_dest
        & (rook_attacks(board.occupied(), king_sq, board.slider_tag())
            | bishop_attacks(board.occupied(), king_sq, board.slider_tag()));
    let mut exposure = 0i32;
    for _sqr in exposure_bb {
        exposure += 1;
    }
    let exposure_score = 27 - exposure;
    let centered_exposure = exposure_score - CENTERED_EXPOSURE;
    centered_exposure * KING_SCORE_FACTOR
}