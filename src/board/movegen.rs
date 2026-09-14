use crate::abort_if;
use crate::board::{Board, CastlingDirection, bishop_attacks, rook_attacks};
use crate::common::{
    Bitboard, DuckMoves, MoveFlag, North, NorthEast, NorthWest, Piece, Rank, South, SouthEast,
    SouthWest, Square, between, king_attacks, knight_attacks,
};
use crate::util::Abort;

impl Board {
    #[inline]
    pub fn gen_moves<V: FnMut(DuckMoves) -> Abort>(&self, mut visitor: V) -> Abort {
        let valid_dest =
            !self.colors(self.stm) & !self.duck.map_or(Bitboard::EMPTY, Square::bitboard);

        abort_if!(self.gen_pawn_moves(&mut visitor));
        abort_if!(self.gen_knight_moves(valid_dest, &mut visitor));
        abort_if!(self.gen_slider_moves(valid_dest, &mut visitor));
        abort_if!(self.gen_king_moves(valid_dest, &mut visitor));

        Abort::No
    }

    #[inline]
    pub fn any_moves<V: FnMut(DuckMoves) -> bool>(&self, mut visitor: V) -> bool {
        self.gen_moves(|moves| {
            if visitor(moves) {
                Abort::Yes
            } else {
                Abort::No
            }
        }) == Abort::Yes
    }

    #[inline]
    fn gen_pawn_moves<V: FnMut(DuckMoves) -> Abort>(&self, visitor: &mut V) -> Abort {
        let empty = !self.occupied();
        let pawns = self.colored_pieces(self.stm, Piece::Pawn);
        let promo_rank = Rank::Eighth.relative_to(self.stm);

        //Pawn Pushes
        for dest in empty & pawns.shift::<North>(self.stm.signum()) {
            let src = dest.offset_dir::<South>(self.stm.signum() as isize);

            if dest.rank() == promo_rank {
                //Push Promotions
                for &flag in &[
                    MoveFlag::PromotionQueen,
                    MoveFlag::PromotionRook,
                    MoveFlag::PromotionBishop,
                    MoveFlag::PromotionKnight,
                ] {
                    abort_if!(visitor(
                        DuckMoves::new(src, dest, flag, empty ^ src ^ dest,)
                    ));
                }
            } else {
                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    MoveFlag::Normal,
                    empty ^ src ^ dest,
                )));
            }
        }

        let start_rank = Rank::Second.relative_to(self.stm);
        let first_step = empty & (pawns & start_rank).shift::<North>(self.stm.signum());
        let second_step = empty & first_step.shift::<North>(self.stm.signum());

        //Pawn Double Pushes
        for dest in second_step {
            let src = dest.offset_dir::<South>(2 * self.stm.signum() as isize);
            abort_if!(visitor(DuckMoves::new(
                src,
                dest,
                MoveFlag::DoublePush,
                empty ^ src ^ dest,
            )));
        }

        //Pawn Captures Left
        let their_pieces = self.colors(!self.stm);
        for dest in their_pieces & pawns.shift::<NorthWest>(self.stm.signum()) {
            let src = dest.offset_dir::<SouthEast>(self.stm.signum() as isize);

            if dest.rank() == promo_rank {
                //Capture Promotions
                for &flag in &[
                    MoveFlag::CapturePromotionQueen,
                    MoveFlag::CapturePromotionRook,
                    MoveFlag::CapturePromotionBishop,
                    MoveFlag::CapturePromotionKnight,
                ] {
                    abort_if!(visitor(DuckMoves::new(
                        src,
                        dest,
                        flag,
                        (empty ^ src) & !dest,
                    )));
                }
            } else {
                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    MoveFlag::Capture,
                    (empty ^ src) & !dest,
                )));
            }
        }

        //Pawn Captures Right
        for dest in their_pieces & pawns.shift::<NorthEast>(self.stm.signum()) {
            let src = dest.offset_dir::<SouthWest>(self.stm.signum() as isize);

            if dest.rank() == promo_rank {
                //Capture Promotions
                for &flag in &[
                    MoveFlag::CapturePromotionQueen,
                    MoveFlag::CapturePromotionRook,
                    MoveFlag::CapturePromotionBishop,
                    MoveFlag::CapturePromotionKnight,
                ] {
                    abort_if!(visitor(DuckMoves::new(
                        src,
                        dest,
                        flag,
                        (empty ^ src) & !dest,
                    )));
                }
            } else {
                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    MoveFlag::Capture,
                    (empty ^ src) & !dest,
                )));
            }
        }

        //En Passant
        if let Some(en_passant) = self.en_passant() {
            let file = en_passant.file();
            let dest = Square::new(file, Rank::Sixth.relative_to(self.stm));
            let victim = Square::new(file, Rank::Fifth.relative_to(self.stm));

            // `calc_en_passant` already calculated all the legal en pheasants
            for src in en_passant.attackers(self.stm) {
                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    MoveFlag::EnPassant,
                    empty ^ src ^ dest ^ victim,
                )));
            }
        }

        Abort::No
    }

    #[inline]
    pub fn gen_knight_moves<V: FnMut(DuckMoves) -> Abort>(
        &self,
        valid_dest: Bitboard,
        visitor: &mut V,
    ) -> Abort {
        let knights = self.colored_pieces(self.stm, Piece::Knight);
        let empty = !self.occupied();

        for src in knights {
            for dest in valid_dest & knight_attacks(src) {
                let flag = if self.piece_on(dest).is_some() {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Normal
                };

                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    flag,
                    (empty ^ src) & !dest,
                )));
            }
        }

        Abort::No
    }

    #[inline]
    pub fn gen_slider_moves<V: FnMut(DuckMoves) -> Abort>(
        &self,
        valid_dest: Bitboard,
        visitor: &mut V,
    ) -> Abort {
        let (empty, blockers) = (!self.occupied(), self.occupied());
        let diag = self.colored_diag_sliders(self.stm);
        let orth = self.colored_orth_sliders(self.stm);

        for src in diag {
            for dest in valid_dest & bishop_attacks(blockers, src, self.slider_tag) {
                let flag = if self.piece_on(dest).is_some() {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Normal
                };

                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    flag,
                    (empty ^ src) & !dest,
                )));
            }
        }

        for src in orth {
            for dest in valid_dest & rook_attacks(blockers, src, self.slider_tag) {
                let flag = if self.piece_on(dest).is_some() {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Normal
                };

                abort_if!(visitor(DuckMoves::new(
                    src,
                    dest,
                    flag,
                    (empty ^ src) & !dest,
                )));
            }
        }

        Abort::No
    }

    #[inline]
    pub fn gen_king_moves<V: FnMut(DuckMoves) -> Abort>(
        &self,
        valid_dest: Bitboard,
        visitor: &mut V,
    ) -> Abort {
        let king = self.king(self.stm);
        let empty = !self.occupied();

        for dest in valid_dest & king_attacks(king) {
            let flag = if self.piece_on(dest).is_some() {
                MoveFlag::Capture
            } else {
                MoveFlag::Normal
            };

            abort_if!(visitor(DuckMoves::new(
                king,
                dest,
                flag,
                (empty ^ king) & !dest,
            )));
        }

        let rank = Rank::First.relative_to(self.stm);
        for &dir in &[CastlingDirection::Long, CastlingDirection::Short] {
            if let Some(file) = self.castling_rights(self.stm).get(dir) {
                let king_dest = Square::new(dir.king_dest(), rank);
                let rook_dest = Square::new(dir.rook_dest(), rank);
                let rook_src = Square::new(file, rank);

                let must_be_empty =
                    between(king, king_dest) | between(rook_src, rook_dest) | king_dest | rook_dest;
                let mut blockers = self.occupied() ^ king ^ rook_src;

                if (blockers & must_be_empty).is_empty() {
                    blockers = blockers ^ king_dest ^ rook_dest;
                    let flag = MoveFlag::new_castling(dir);

                    abort_if!(visitor(DuckMoves::new(king, rook_src, flag, !blockers,)));
                }
            }
        }

        Abort::No
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::util::Abort;

    #[test]
    fn pawn_attack() {
        const EXPECTED_RESULT: usize = 8 * (64 - 6) + 2 * (64 - 5);
        let board = Board::from_fen("8/3k4/8/2p*p3/3P4/8/3K4/8 w - - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }

    #[test]
    fn knight_corners() {
        const EXPECTED_RESULT: usize = 13 * (64 - 7);
        let board = Board::from_fen("N3k2N/8/8/4*3/8/8/8/N3K2N w - - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }

    #[test]
    fn knight_captures_and_blockers() {
        const EXPECTED_RESULT: usize = 20 * (64 - 7) + 2 * (64 - 6);
        let board = Board::from_fen("7k/7n/4*3/1r3R2/3N4/8/8/K7 w - - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }

    #[test]
    fn king_empty() {
        const EXPECTED_RESULT: usize = 8 * (64 - 3);
        let board = Board::from_fen("8/4K3/8/4*3/8/8/4k3/8 b - - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }

    #[test]
    fn king_capture() {
        const EXPECTED_RESULT: usize = 8 * (64 - 10);
        let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }

    #[test]
    fn king_castling() {
        const EXPECTED_RESULT: usize = 23 * (64 - 7) + (64 - 6);
        let board = Board::from_fen("8/1k6/8/8/3*4/6n1/8/R1n1K2R w KQ - 0 1")
            .expect("board couldnt parse fen string");
        board.display(true);

        let mut len = 0;
        board.gen_moves(|moves| {
            len += moves.len();
            Abort::No
        });

        assert_eq!(len, EXPECTED_RESULT);
    }
}
