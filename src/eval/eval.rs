use super::duck_shield::duck_shield_features;
use super::psqt::{EG_PSQT, MG_PSQT};
use crate::board::Board;
use crate::common::{Color, North, NorthEast, NorthWest, Piece, Rank, South, Square};
use crate::score::Score;

const MG_PIECE_VALUES: [i32; Piece::COUNT] = [82, 337, 365, 477, 1025, 0];
const EG_PIECE_VALUES: [i32; Piece::COUNT] = [94, 281, 297, 512, 936, 0];
const MG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(MG_PIECE_VALUES, MG_PSQT);
const EG_PIECE_SCORES: [[i32; Square::COUNT]; Piece::COUNT] =
    combine_scores(EG_PIECE_VALUES, EG_PSQT);
const PHASE_WEIGHTS: [i32; Piece::COUNT] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;
const TEMPO_BONUS_MG: i32 = 30;
const TEMPO_BONUS_EG: i32 = 25;
const PAWN_DEFENCE_BONUS_MG: i32 = 8;
const PAWN_DEFENCE_BONUS_EG: i32 = 6;
const KNIGHT_OUTPOST_BONUS_MG: i32 = 12;
const KNIGHT_OUTPOST_BONUS_EG: i32 = 8;
const DUCK_SHIELD_DANGER_MG: i32 = 45;
const DUCK_SHIELD_DANGER_EG: i32 = 30;

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
    // Calculate duck shield danger factor. This is an asymmetric feature,
    // hence why it's done here and not in "side_score".
    let danger = duck_shield_features(board, us).danger();
    let danger_mg = danger * danger * DUCK_SHIELD_DANGER_MG;
    let danger_eg = danger * danger * DUCK_SHIELD_DANGER_EG;

    ((us_mg - them_mg - danger_mg) * phase + (us_eg - them_eg - danger_eg) * (MAX_PHASE - phase))
        / MAX_PHASE
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

    let outposts = knight_outposts(board, color);
    mg += outposts * KNIGHT_OUTPOST_BONUS_MG;
    eg += outposts * KNIGHT_OUTPOST_BONUS_EG;

    let pawns = board.colored_pieces(color, Piece::Pawn);
    let pawn_attacks =
        pawns.shift::<NorthEast>(color.signum()) | pawns.shift::<NorthWest>(color.signum());
    let defended_pieces = (pawn_attacks & board.colors(color)).popcnt() as i32;
    mg += defended_pieces * PAWN_DEFENCE_BONUS_MG;
    eg += defended_pieces * PAWN_DEFENCE_BONUS_EG;

    let stm = (board.stm() == color) as i32;
    mg += stm * TEMPO_BONUS_MG;
    eg += stm * TEMPO_BONUS_EG;

    (mg, eg, phase)
}

#[inline]
fn knight_outposts(board: &Board, color: Color) -> i32 {
    let ranks = (Rank::Fourth.bitboard() | Rank::Fifth | Rank::Sixth).relative_to(color);
    let knights = board.colored_pieces(color, Piece::Knight) & ranks;
    if knights.is_empty() {
        return 0;
    }

    let pawns = board.colored_pieces(color, Piece::Pawn);
    let support =
        pawns.shift::<NorthEast>(color.signum()) | pawns.shift::<NorthWest>(color.signum());
    let candidates = knights & support;
    if candidates.is_empty() {
        return 0;
    }

    let enemy = !color;
    let enemy_pawns = board.colored_pieces(enemy, Piece::Pawn);
    let enemy_attacks = enemy_pawns.shift::<NorthEast>(enemy.signum())
        | enemy_pawns.shift::<NorthWest>(enemy.signum());
    let challenge_span = match enemy {
        Color::White => enemy_attacks.smear::<North>(),
        Color::Black => enemy_attacks.smear::<South>(),
    };

    (candidates & !challenge_span).popcnt() as i32
}
