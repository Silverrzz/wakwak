use crate::{
    board::{Board, bishop_attacks, rook_attacks},
    common::{Color, Piece, between, king_attacks, knight_attacks, pawn_attacks},
};

/// King danger inputs for the defending side, as if the duck were not on
/// the board. The defending side is the one about to move a piece and then
/// relocate the duck, so the current position doesn't really matter.
pub(crate) struct DuckShieldFeatures {
    /// Checks knight/pawn/king contact, or a slider with no gap that no
    /// single duck placement can block.
    pub(crate) unblockable: u8,
    /// Slider rays with at least one empty square between attacker and
    /// king. A duck can cover exactly one of these.
    pub(crate) blockable: u8,
}

impl DuckShieldFeatures {
    /// If `danger > 0`, then a single duck placement cannot fully shield the
    /// king. So the defending side must resolve the threat with a move that
    /// addresses the danger.
    pub(crate) fn danger(&self) -> i32 {
        self.unblockable as i32 + (self.blockable as i32 - 1).max(0)
    }
}

#[inline]
pub(crate) fn duck_shield_features(board: &Board, defender: Color) -> DuckShieldFeatures {
    let king = board.king(defender);
    // Occupancy without the duck: `defender` relocates it themselves, so its
    // current square can't help and is ignored.
    let occ = board.colors(defender) | board.colors(!defender);
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
        // If there is at least one square between the slider and the king, it is blockable.
        else {
            blockable += 1;
        }
    }

    DuckShieldFeatures {
        unblockable,
        blockable,
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::common::Color;

    #[test]
    fn no_threats_when_king_is_safe() {
        const FEN: &str = "7k/8/8/8/4*3/8/8/4K3 w - - 0 1";
        let board = Board::from_fen(FEN).unwrap();
        let features = super::duck_shield_features(&board, board.stm());
        assert_eq!(features.unblockable, 0);
        assert_eq!(features.blockable, 0);
        assert_eq!(features.danger(), 0);
    }

    #[test]
    fn knight_king_attack_is_unblockable() {
        const FEN: &str = "7k/8/8/8/4*3/3n4/8/4K3 w - - 0 1";
        let board = Board::from_fen(FEN).unwrap();
        let features = super::duck_shield_features(&board, board.stm());
        assert_eq!(features.unblockable, 1);
        assert_eq!(features.blockable, 0);
        assert_eq!(features.danger(), 1);
    }

    #[test]
    fn single_slider_king_attack_is_blockable_and_not_dangerous() {
        const FEN: &str = "4r2k/8/8/8/8/8/8/4K3 w - - 0 1";
        let board = Board::from_fen(FEN).unwrap();
        assert!(board.duck().is_none());
        let features = super::duck_shield_features(&board, board.stm());
        assert_eq!(features.unblockable, 0);
        assert_eq!(features.blockable, 1);
        assert_eq!(features.danger(), 0);
    }

    #[test]
    fn two_sliders_on_the_king_are_undefendable_by_one_duck() {
        // White rook attacks via the e-file, white bishop checks along the
        // h1-e4 diagonal.
        const FEN: &str = "4R3/8/8/8/4k3/8/8/K6B w - - 0 1";
        let board = Board::from_fen(FEN).unwrap();
        let features = super::duck_shield_features(&board, Color::Black);
        assert_eq!(features.unblockable, 0);
        assert_eq!(features.blockable, 2);
        assert_eq!(features.danger(), 1);
    }
}
