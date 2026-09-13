use super::piece_square_tables::PIECE_SQUARE_TABLES;
use crate::board::Board;
use crate::common::Color;
use crate::common::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::score::Score;

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];
const PIECE_SCORES: [[i32; 64]; 6] = combine_scores(PIECE_VALUES, PIECE_SQUARE_TABLES);

const fn combine_scores(values: [i32; 6], mut tables: [[i32; 64]; 6]) -> [[i32; 64]; 6] {
    let mut piece = 0;
    while piece < 6 {
        let mut square = 0;
        while square < 64 {
            tables[piece][square] += values[piece];
            square += 1;
        }
        piece += 1;
    }
    tables
}

pub fn evaluate(board: &Board) -> Score {
    let us = board.stm();
    let them = !us;
    side_score(board, us) - side_score(board, them)
}

fn side_score(board: &Board, color: Color) -> Score {
    let mut score = Score::ZERO;
    for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
        for square in board.colored_pieces(color, piece) {
            score += PIECE_SCORES[piece][square.relative_to(color)];
        }
    }
    score
}
