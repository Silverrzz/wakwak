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
                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);

                self.remove_castling_right(self.stm, src);
            }
            MoveFlag::DoublePush => {
                debug_assert_eq!(piece, Piece::Pawn);

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, piece, self.stm);
                new_en_passant = Some(dest.file());
            }
            MoveFlag::Capture => {
                let victim = victim.expect("Board::make_move(): No victim on dest square");

                self.toggle_square(src, piece, self.stm);
                self.toggle_square(dest, victim, self.stm);
                self.toggle_square(dest, piece, self.stm);

                self.remove_castling_right(self.stm, src);
                self.remove_castling_right(!self.stm, dest);
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
                    self.toggle_square(dest, victim, !self.stm);
                    self.remove_castling_right(!self.stm, dest);
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
