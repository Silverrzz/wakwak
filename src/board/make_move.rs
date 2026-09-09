use crate::board::{Board, CastlingDirection};
use crate::common::{Color, Move, MoveFlag, Piece, Rank, Square};

impl Board {
    /// Makes a move on the board assuming it is legal
    pub fn make_move(&mut self, mv: Move) {
        self.hmc += 1;
        self.fmc += (self.stm == Color::Black) as u16;
        self.set_en_passant(None);
        self.set_duck(Some(mv.duck()));

        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let piece = self
            .piece_on(src)
            .expect("Board::make_move(): Empty source square");
        let victim = self.piece_on(dest);

        if piece == Piece::Pawn || (victim.is_some() || flag.is_castling()) {
            self.hmc = 0;
        }

        let mut new_en_passant = None;
        match flag {
            MoveFlag::Normal => {
                self.remove_castling_right(self.stm, src);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
            }
            MoveFlag::DoublePush => {
                debug_assert_eq!(piece, Piece::Pawn);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                new_en_passant = Some(dest.file());
            }
            MoveFlag::Capture => {
                let victim = victim.expect("Board::make_move(): No victim on dest square");

                self.remove_castling_right(self.stm, src);
                self.remove_castling_right(!self.stm, dest);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, victim, self.stm);
                self.toggle_square(dest, piece, self.stm);
            }
            MoveFlag::EnPassant => {
                debug_assert_eq!(piece, Piece::Pawn);

                let victim = Square::new(dest.file(), Rank::Fifth.relative_to(self.stm));
                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                self.toggle_square(victim, piece, !self.stm);
            }
            MoveFlag::LongCastling => {
                debug_assert_eq!(piece, Piece::King);

                let rank = Rank::First.relative_to(self.stm);
                let king_dest = Square::new(CastlingDirection::Long.king_dest(), rank);
                let rook_dest = Square::new(CastlingDirection::Long.rook_dest(), rank);

                self.castling(src, king_dest, dest, rook_dest);
            }
            MoveFlag::ShortCastling => {
                debug_assert_eq!(piece, Piece::King);

                let rank = Rank::First.relative_to(self.stm);
                let king_dest = Square::new(CastlingDirection::Short.king_dest(), rank);
                let rook_dest = Square::new(CastlingDirection::Short.rook_dest(), rank);

                self.castling(src, king_dest, dest, rook_dest);
            }
            flag if flag.is_promotion() => {
                debug_assert_eq!(piece, Piece::Pawn);
                debug_assert_eq!(dest.rank(), Rank::Eighth.relative_to(self.stm));

                let promotion = flag.promotion().unwrap();

                self.toggle_square(src, piece, self.stm);
                if let Some(victim) = self.piece_on(dest) {
                    self.remove_castling_right(!self.stm, dest);
                    self.toggle_square(dest, victim, !self.stm);
                }

                self.toggle_square(dest, promotion, self.stm);
            }
            _ => unreachable!("All variants are covered by the match arm"),
        }

        self.toggle_stm();
        self.calc_en_passant(new_en_passant);
    }

    #[inline]
    fn remove_castling_right(&mut self, color: Color, sq: Square) {
        if sq.rank() == Rank::First.relative_to(color) {
            let rights = self.castling_rights(color);
            let file = sq.file();

            if rights.get(CastlingDirection::Long) == Some(file) {
                self.set_castling_rights(color, CastlingDirection::Long, None);
            }

            if rights.get(CastlingDirection::Short) == Some(file) {
                self.set_castling_rights(color, CastlingDirection::Short, None);
            }

            if sq == self.king(self.stm) {
                self.set_castling_rights(color, CastlingDirection::Short, None);
                self.set_castling_rights(color, CastlingDirection::Long, None);
            }
        }
    }

    #[inline]
    fn castling(
        &mut self,
        king_src: Square,
        king_dest: Square,
        rook_src: Square,
        rook_dest: Square,
    ) {
        self.toggle_square(king_src, Piece::King, self.stm);
        self.toggle_square(rook_src, Piece::Rook, self.stm);
        self.toggle_square(king_dest, Piece::King, self.stm);
        self.toggle_square(rook_dest, Piece::Rook, self.stm);

        self.set_castling_rights(self.stm, CastlingDirection::Long, None);
        self.set_castling_rights(self.stm, CastlingDirection::Short, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::File;

    #[test]
    fn white_pawn_promotes_to_queen() {
        let mut board = Board::from_fen("4k3/P7/8/8/3*4/8/8/4K3 w - - 7 1").unwrap();

        // Promote on a8 and place the duck on the old pawn square
        let mv = Move::new(Square::A7, Square::A8, Square::A7, MoveFlag::PromotionQueen);
        assert!(board.gen_moves().contains(&mv));
        board.make_move(mv);

        // The pawn is gone and a white queen occupies a8
        assert_eq!(board.piece_on(Square::A7), None);
        assert!(board.colored_pieces(Color::White, Piece::Pawn).is_empty());
        assert_eq!(board.piece_on(Square::A8), Some(Piece::Queen));
        assert_eq!(board.color_on(Square::A8), Some(Color::White));
        let expected = Board::from_fen("Q3k3/*7/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        assert_eq!(board.hash(), expected.hash());
    }

    #[test]
    fn black_pawn_promotes_to_knight() {
        let mut board = Board::from_fen("4k3/8/8/8/3*4/8/7p/4K3 b - - 7 1").unwrap();

        // Underpromote on h1 and move the duck to h2
        let mv = Move::new(Square::H2, Square::H1, Square::H2, MoveFlag::PromotionKnight);
        assert!(board.gen_moves().contains(&mv));
        board.make_move(mv);

        // The replacement piece belongs to Black
        assert_eq!(board.piece_on(Square::H2), None);
        assert!(board.colored_pieces(Color::Black, Piece::Pawn).is_empty());
        assert_eq!(board.piece_on(Square::H1), Some(Piece::Knight));
        assert_eq!(board.color_on(Square::H1), Some(Color::Black));
        let expected = Board::from_fen("4k3/8/8/8/8/8/7*/4K2n w - - 0 2").unwrap();
        assert_eq!(board.hash(), expected.hash());
    }

    #[test]
    fn promotion_capture_removes_queenside_castling_right() {
        let mut board = Board::from_fen("r3k2r/1P6/8/8/3*4/8/8/4K3 w kq - 0 1").unwrap();

        // Capture the a8 rook while promoting, with the duck moving to b7
        let mv = Move::new(Square::B7, Square::A8, Square::B7, MoveFlag::CapturePromotionQueen);
        assert!(board.gen_moves().contains(&mv));
        board.make_move(mv);

        // Only black queen castling should be gone
        let rights = board.castling_rights(Color::Black);
        assert_eq!(rights.get(CastlingDirection::Long), None);
        assert_eq!(rights.get(CastlingDirection::Short), Some(File::H));
        assert_eq!(board.piece_on(Square::B7), None);
        assert_eq!(board.piece_on(Square::A8), Some(Piece::Queen));
        assert_eq!(board.color_on(Square::A8), Some(Color::White));
        assert!(!board.colors(Color::Black).has(Square::A8));
        let expected = Board::from_fen("Q3k2r/1*6/8/8/8/8/8/4K3 b k - 0 1").unwrap();
        assert_eq!(board.hash(), expected.hash());
    }

    #[test]
    fn king_move_clears_both_castling_rights() {
        let mut board = Board::from_fen("4k3/8/8/8/3*4/8/8/R3K2R w KQ - 0 1").unwrap();
        let rights = board.castling_rights(Color::White);
        assert_eq!(rights.get(CastlingDirection::Long), Some(File::A));
        assert_eq!(rights.get(CastlingDirection::Short), Some(File::H));

        let mv = Move::new(Square::E1, Square::E2, Square::E1, MoveFlag::Normal);
        assert!(board.gen_moves().contains(&mv));
        board.make_move(mv);

        assert_eq!(board.king(Color::White), Square::E2);
        let rights = board.castling_rights(Color::White);
        assert_eq!(
            (rights.get(CastlingDirection::Long), rights.get(CastlingDirection::Short)),
            (None, None),
        );
    }
}
