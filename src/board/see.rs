use crate::board::{Board, bishop_attacks, rook_attacks};
use crate::common::{
    Bitboard, Color, Move, MoveFlag, Piece, Square, king_attacks, knight_attacks, pawn_attacks,
};
use crate::search::Params;

impl Board {
    #[inline]
    pub fn cmp_see(&self, mv: Move, threshold: i32) -> bool {
        if mv.flag().is_castling() {
            return threshold <= 0;
        }

        // Ignore the duck
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let next_victim = flag
            .promotion()
            .unwrap_or_else(|| self.piece_on(src).unwrap());
        let mut balance = -threshold
            + match flag {
                MoveFlag::Normal | MoveFlag::DoublePush => 0,
                MoveFlag::EnPassant => Params::see_value(Piece::Pawn),
                MoveFlag::Capture => Params::see_value(self.piece_on(dest).unwrap()),
                _ if flag.is_capture_promotion() => {
                    Params::see_value(self.piece_on(dest).unwrap())
                        + Params::see_value(flag.promotion().unwrap())
                        - Params::see_value(Piece::Pawn)
                }
                _ if let Some(promo) = flag.promotion() => {
                    Params::see_value(promo) - Params::see_value(Piece::Pawn)
                }
                _ => unreachable!(),
            };

        if balance < 0 {
            return false;
        }

        balance -= Params::see_value(next_victim);

        if balance >= 0 {
            return true;
        }

        let mut occupied =
            self.occupied() ^ self.duck.map_or(Bitboard::EMPTY, Square::bitboard) ^ src | dest;
        if flag == MoveFlag::EnPassant {
            occupied ^= Square::new(dest.file(), src.rank());
        }

        let diag = self.diag_sliders();
        let orth = self.orth_sliders();

        #[rustfmt::skip]
        let mut attackers = occupied & (
            (pawn_attacks(dest, Color::White) & self.colored_pieces(Color::Black, Piece::Pawn))
            | (pawn_attacks(dest, Color::Black) & self.colored_pieces(Color::White, Piece::Pawn))
            | (knight_attacks(dest) & self.pieces(Piece::Knight))
            | (bishop_attacks(occupied, dest, self.slider_tag) & diag)
            | (rook_attacks(occupied, dest, self.slider_tag) & orth)
            | (king_attacks(dest) & self.pieces(Piece::King))
        );

        let mut stm = !self.stm;
        loop {
            let our_attackers = attackers & self.colors(stm);
            if our_attackers.is_empty() {
                break;
            }

            let next_victim = *Piece::ALL
                .iter()
                .find(|&&p| (our_attackers & self.pieces(p)).is_nonempty())
                .unwrap();

            occupied ^= (self.pieces(next_victim) & our_attackers).next();

            if matches!(next_victim, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                attackers |= bishop_attacks(occupied, dest, self.slider_tag) & diag;
            }

            if matches!(next_victim, Piece::Rook | Piece::Queen) {
                attackers |= rook_attacks(occupied, dest, self.slider_tag) & orth;
            }

            attackers &= occupied;
            stm = !stm;

            balance = -balance - 1 - Params::see_value(next_victim);
            if balance >= 0 {
                if next_victim == Piece::King && (attackers & self.colors(self.stm)).is_nonempty() {
                    stm = !stm;
                }

                break;
            }
        }

        self.stm != stm
    }
}
