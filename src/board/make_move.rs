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

        if piece == Piece::Pawn || flag.is_capture() {
            self.hmc = 0;
        }

        let mut new_en_passant = None;
        match flag {
            MoveFlag::Normal => {
                self.remove_castling_right(self.stm, src);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                self.mailbox[src] = None;
                self.mailbox[dest] = Some(piece);
            }
            MoveFlag::DoublePush => {
                debug_assert_eq!(piece, Piece::Pawn);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                self.mailbox[src] = None;
                self.mailbox[dest] = Some(piece);

                new_en_passant = Some(dest.file());
            }
            MoveFlag::Capture => {
                let victim = victim.expect("Board::make_move(): No victim on dest square");

                self.remove_castling_right(self.stm, src);
                self.remove_castling_right(!self.stm, dest);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, victim, !self.stm);
                self.toggle_square(dest, piece, self.stm);

                self.mailbox[src] = None;
                self.mailbox[dest] = Some(piece);
            }
            MoveFlag::EnPassant => {
                debug_assert_eq!(piece, Piece::Pawn);

                let victim = Square::new(dest.file(), Rank::Fifth.relative_to(self.stm));
                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                self.toggle_square(victim, piece, !self.stm);

                self.mailbox[src] = None;
                self.mailbox[dest] = Some(piece);
                self.mailbox[victim] = None;
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
                self.mailbox[src] = None;
                self.mailbox[dest] = Some(promotion);
            }
            _ => {
                unreachable!("Board::make_move(): All variants should be covered by the match arm")
            }
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
        }

        if sq == self.king(self.stm) {
            self.set_castling_rights(color, CastlingDirection::Short, None);
            self.set_castling_rights(color, CastlingDirection::Long, None);
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

        self.mailbox[king_src] = None;
        self.mailbox[rook_src] = None;
        self.mailbox[king_dest] = Some(Piece::King);
        self.mailbox[rook_dest] = Some(Piece::Rook);

        self.set_castling_rights(self.stm, CastlingDirection::Long, None);
        self.set_castling_rights(self.stm, CastlingDirection::Short, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::File;

    #[test]
    fn short_castling() {
        let mut board = Board::from_fen("4k3/8/8/8/3*4/8/8/R3K2R w KQ - 7 1").unwrap();
        // king and rook move, rights go away
        board.make_move(Move::new(
            Square::E1,
            Square::H1,
            Square::E1,
            MoveFlag::ShortCastling,
        ));
        let expected = "4k3/8/8/8/8/8/8/R3*RK1 b - - 8 1";
        assert_eq!(board.to_fen(false), expected);
        assert_eq!(board.hash(), Board::from_fen(expected).unwrap().hash());
    }

    #[test]
    fn long_castling() {
        let mut board = Board::from_fen("r3k2r/8/8/8/3*4/8/8/4K3 b kq - 7 1").unwrap();
        // same for black
        board.make_move(Move::new(
            Square::E8,
            Square::A8,
            Square::E8,
            MoveFlag::LongCastling,
        ));
        let expected = "2kr*2r/8/8/8/8/8/8/4K3 w - - 8 2";
        assert_eq!(board.to_fen(false), expected);
        assert_eq!(board.hash(), Board::from_fen(expected).unwrap().hash());
    }

    #[test]
    fn en_passant_capture() {
        let mut board = Board::from_fen("4k3/8/8/3pP3/7*/8/8/4K3 w - d6 0 1").unwrap();
        // d5 takes d6, duck goes where the captured pawn was
        board.make_move(Move::new(
            Square::E5,
            Square::D6,
            Square::D5,
            MoveFlag::EnPassant,
        ));
        let expected = "4k3/8/3P4/3*4/8/8/8/4K3 b - - 0 1";
        assert_eq!(board.to_fen(false), expected);
        assert_eq!(board.hash(), Board::from_fen(expected).unwrap().hash());
    }

    #[test]
    fn duck_blocks_en_passant_after_double_push() {
        let start = Board::from_fen("4k3/8/8/8/3p3*/8/4P3/4K3 w - - 0 1").unwrap();

        // e4 available for ep
        let mut clear = start;
        let mv = Move::new(Square::E2, Square::E4, Square::E2, MoveFlag::DoublePush);
        assert!(clear.any_moves(|moves| moves.has(mv)));
        clear.make_move(mv);
        assert!(clear.en_passant().is_some());
        assert!(clear.any_moves(|moves| moves.flag == MoveFlag::EnPassant));

        // duck e3 should block ep
        let mut blocked = start;
        let mv = Move::new(Square::E2, Square::E4, Square::E3, MoveFlag::DoublePush);
        assert!(blocked.any_moves(|moves| moves.has(mv)));

        blocked.make_move(mv);
        assert_eq!(blocked.en_passant(), None);
        assert!(!blocked.any_moves(|moves| moves.flag == MoveFlag::EnPassant));
    }

    #[test]
    fn capture_updates_color_bitboards_and_hash() {
        let mut board = Board::from_fen("4k3/8/8/8/3n3*/8/3R4/4K3 w - - 0 1").unwrap();

        // white rook captures the black knight on d4
        let mv = Move::new(Square::D2, Square::D4, Square::D2, MoveFlag::Capture);
        assert!(board.any_moves(|moves| moves.has(mv)));
        board.make_move(mv);

        // dest belongs only to white, and the knight is gone.
        assert_eq!(board.piece_on(Square::D2), None);
        assert_eq!(board.piece_on(Square::D4), Some(Piece::Rook));
        assert!(board.pieces(Piece::Knight).is_empty());
        let expected = Board::from_fen("4k3/8/8/8/3R4/8/3*4/4K3 b - - 0 1").unwrap();
        assert_eq!(board.colors(Color::White), expected.colors(Color::White));
        assert_eq!(board.colors(Color::Black), expected.colors(Color::Black));
        assert_eq!(board.hash(), expected.hash());
    }

    #[test]
    fn white_pawn_promotes_to_queen() {
        let mut board = Board::from_fen("4k3/P7/8/8/3*4/8/8/4K3 w - - 7 1").unwrap();

        // Promote on a8 and place the duck on the old pawn square
        let mv = Move::new(Square::A7, Square::A8, Square::A7, MoveFlag::PromotionQueen);
        assert!(board.any_moves(|moves| moves.has(mv)));
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
        let mv = Move::new(
            Square::H2,
            Square::H1,
            Square::H2,
            MoveFlag::PromotionKnight,
        );
        assert!(board.any_moves(|moves| moves.has(mv)));
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
        let mv = Move::new(
            Square::B7,
            Square::A8,
            Square::B7,
            MoveFlag::CapturePromotionQueen,
        );
        assert!(board.any_moves(|moves| moves.has(mv)));
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
        assert!(board.any_moves(|moves| moves.has(mv)));
        board.make_move(mv);

        assert_eq!(board.king(Color::White), Square::E2);
        let rights = board.castling_rights(Color::White);
        assert_eq!(
            (
                rights.get(CastlingDirection::Long),
                rights.get(CastlingDirection::Short)
            ),
            (None, None),
        );
    }
}
