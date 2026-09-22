use crate::board::{Board, bishop_attacks, rook_attacks};
use crate::common::{
    Bitboard, Color, Move, MoveFlag, Piece, Square, between, king_attacks, knight_attacks,
    pawn_attacks,
};
use crate::search::Params;

impl Board {
    #[inline]
    pub fn cmp_see(&self, mv: Move, threshold: i32) -> bool {
        if mv.flag().is_castling() {
            return threshold <= 0;
        }

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

        let mut occupied = self.occupied_duckless() ^ src | dest;
        if flag == MoveFlag::EnPassant {
            occupied ^= Square::new(dest.file(), src.rank());
        }

        let diag = self.diag_sliders();
        let orth = self.orth_sliders();

        // All possible attackers to dest
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
        let mut duck = mv.duck().bitboard();

        loop {
            // Possible attackers taking into account the duck as a blocker
            let non_sliders = attackers & !(diag | orth);
            let unblocked_sliders = attackers
                & ((bishop_attacks(occupied | duck, dest, self.slider_tag) & diag)
                    | (rook_attacks(occupied | duck, dest, self.slider_tag) & orth));
            let our_attackers = (non_sliders | unblocked_sliders) & self.colors(stm);
            if our_attackers.is_empty() {
                break;
            }

            // Our next attacker
            let our_attacker = Piece::ALL
                .iter()
                .copied()
                .find(|&p| (our_attackers & self.pieces(p)).is_nonempty())
                .unwrap();

            occupied ^= (self.pieces(our_attacker) & our_attackers).next();

            // Reveal x-ray attackers behind our attacker
            if matches!(our_attacker, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                attackers |= bishop_attacks(occupied, dest, self.slider_tag) & diag;
            }
            if matches!(our_attacker, Piece::Rook | Piece::Queen) {
                attackers |= rook_attacks(occupied, dest, self.slider_tag) & orth;
            }
            attackers &= occupied;

            // Opponent's next attacker
            let their_attackers = attackers & self.colors(!stm);
            let their_attacker = Piece::ALL
                .iter()
                .copied()
                .find(|&p| (their_attackers & self.pieces(p)).is_nonempty());

            let prev_duck = duck;
            // If the opponent's next attacker is not a slider, it doesn't really matter where we place the duck.
            duck = Bitboard::EMPTY;

            if let Some(piece) = their_attacker {
                // Duck can only block sliders.
                if matches!(piece, Piece::Bishop | Piece::Rook | Piece::Queen) {
                    let sq = (their_attackers & self.pieces(piece)).next();

                    // Duck MUST move.
                    let valid_blocks = between(dest, sq) & !occupied & !prev_duck;
                    if valid_blocks.is_nonempty() {
                        duck = valid_blocks.next().bitboard();
                    }
                }
            }

            stm = !stm;
            balance = -balance - 1 - Params::see_value(our_attacker);

            if balance >= 0 {
                if our_attacker == Piece::King {
                    let non_sliders = attackers & !(diag | orth);
                    let unblocked_sliders = attackers
                        & ((bishop_attacks(occupied | duck, dest, self.slider_tag) & diag)
                            | (rook_attacks(occupied | duck, dest, self.slider_tag) & orth));

                    let their_attackers = (non_sliders | unblocked_sliders) & self.colors(stm);
                    if their_attackers.is_nonempty() {
                        // King can be recaptured, we lost
                        stm = !self.stm;
                    }
                }

                break;
            }
        }

        self.stm != stm
    }
}
