use crate::{
    board::{Board, SliderTag, bishop_attacks, rook_attacks},
    common::{Bitboard, Color, Piece, between, king_attacks, knight_attacks, pawn_attacks},
    util::tagged_cell::TaggedCell,
};

struct DuckShieldFeatures {
    unblockable: u8,
    blockable: u8,
    escape_squares: u8,
}

pub(crate) fn duck_shield_features(board: &Board, defender: Color) -> DuckShieldFeatures {
    let king = board.king(defender);
    // Occupancy minus the duck.
    let occ = board.occupied() & !board.duck().expect("Duck sqaure must be set.");
    let them = !defender;

    let their_knights = board.colored_pieces(them, Piece::Knight);
    let their_pawns = board.colored_pieces(them, Piece::Pawn);
    let their_king = board.king(them);
    let their_rook_queens = board.colored_orth_sliders(them);
    let their_bishop_queens = board.colored_diag_sliders(them);

    // Calculate what attacks the duck cannot block.
    let mut unblockable: u8 = 0;
    unblockable += (knight_attacks(king) & their_knights).popcnt() as u8;
    unblockable += (pawn_attacks(king, defender) & their_pawns).popcnt() as u8;
    unblockable += (king_attacks(king) & their_king).popcnt() as u8;

    let mut blockable: u8 = 0;
    let slider_tag = board.slider_tag();

    let sliders = (rook_attacks(occ, king, slider_tag) & their_rook_queens)
        | (bishop_attacks(occ, king, slider_tag) & their_bishop_queens);

    for sq in sliders.iter() {
        if between(king, sq).is_empty() {
            unblockable += 1;
        }
        // adjacent
        else {
            blockable += 1;
        }
    }

    let mut escape_squares = 0;
    // let escapes = king_attacks(king) & !board.colors(defender) & !attacked_by(them, occ);
    DuckShieldFeatures {
        unblockable,
        blockable,
        escape_squares,
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn basic_feature_detection() {
        const FEN: &str = "4r2k/8/8/b7/4*3/3n4/8/4K3 w - - 0 1";
        let board = Board::from_fen(FEN).unwrap();
        let features = super::duck_shield_features(&board, board.stm());
    }
}
