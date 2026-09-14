use crate::board::{Board, bishop_attacks, rook_attacks};
use crate::common::{
    Move, MoveFlag, Piece, Rank, Square, between, king_attacks, knight_attacks, pawn_attacks,
};

impl Board {
    #[inline]
    pub fn is_legal(&self, mv: Move) -> bool {
        let (src, dest, duck, flag) = (mv.src(), mv.dest(), mv.duck(), mv.flag());

        if self.color_on(src) != Some(self.stm) {
            return false;
        }

        let src_piece = self.piece_on(src).unwrap();

        match flag {
            MoveFlag::DoublePush => {
                let src_rank = Rank::Second.relative_to(self.stm);
                let dest_rank = Rank::Fourth.relative_to(self.stm);
                let between = src.offset(0, self.stm.signum() as isize);

                if src_piece != Piece::Pawn
                    || src.rank() != src_rank
                    || dest.rank() != dest_rank
                    || self.occupied().has(between)
                {
                    return false;
                }
            }
            MoveFlag::EnPassant => {
                return if let Some(ep) = self.en_passant {
                    let ep_file = ep.file();
                    let ep_dest = Square::new(ep_file, Rank::Sixth.relative_to(self.stm));
                    let src_rank = Rank::Fifth.relative_to(self.stm);

                    if src_piece != Piece::Pawn || src.rank() != src_rank || dest != ep_dest {
                        return false;
                    }

                    let left = src.file() < dest.file();
                    let valid_ep = if left {
                        ep.left() && src == Square::new(ep_file.offset(-1), src_rank)
                    } else {
                        ep.right() && src == Square::new(ep_file.offset(1), src_rank)
                    };

                    if !valid_ep {
                        return false;
                    }

                    let victim = Square::new(ep_file, Rank::Fifth.relative_to(self.stm));
                    (!self.occupied() ^ src ^ dest ^ victim).has(duck)
                } else {
                    false
                };
            }
            _ if let Some(dir) = flag.castling_dir() => {
                if src_piece != Piece::King {
                    return false;
                }

                let rank = Rank::First.relative_to(self.stm);
                let rights = self.castling_rights(self.stm);
                let rook_src = rights.get(dir).map(|f| Square::new(f, rank));
                let king_dest = Square::new(dir.king_dest(), rank);
                let rook_dest = Square::new(dir.rook_dest(), rank);

                if rook_src.is_none_or(|sq| dest != sq) {
                    return false;
                }
                let rook_src = rook_src.unwrap();

                let must_be_empty =
                    between(src, king_dest) | between(rook_src, rook_dest) | king_dest | rook_dest;
                let blockers = self.occupied() ^ src ^ rook_src;
                if (blockers & must_be_empty).is_nonempty() {
                    return false;
                }

                return !(blockers ^ king_dest ^ rook_dest).has(duck);
            }
            _ if flag.is_promotion() => {
                let src_rank = Rank::Seventh.relative_to(self.stm);
                let dest_rank = Rank::Eighth.relative_to(self.stm);

                if src_piece != Piece::Pawn || src.rank() != src_rank || dest.rank() != dest_rank {
                    return false;
                }
            }
            _ => {
                let legal_piece_moves = match src_piece {
                    Piece::Pawn => {
                        pawn_attacks(src, self.stm) | src.offset(0, self.stm.signum() as isize)
                    }
                    Piece::Knight => knight_attacks(src),
                    Piece::Bishop => bishop_attacks(self.occupied(), src, self.slider_tag),
                    Piece::Rook => rook_attacks(self.occupied(), src, self.slider_tag),
                    Piece::Queen => {
                        bishop_attacks(self.occupied(), src, self.slider_tag)
                            | rook_attacks(self.occupied(), src, self.slider_tag)
                    }
                    Piece::King => king_attacks(src),
                };

                if !legal_piece_moves.has(dest)
                    || self.colors(self.stm).has(dest)
                    || self.colors(!self.stm).has(dest) != flag.is_capture()
                {
                    return false;
                }
            }
        }

        !((self.occupied() ^ src) & !dest).has(duck)
    }
}
