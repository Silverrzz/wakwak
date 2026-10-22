use crate::board::{Board, bishop_attacks, rook_attacks};
use crate::common::{Bitboard, Move, MoveFlag, Piece, Square, between};
use crate::search::Params;

impl Board {
    #[inline]
    fn move_value(&self, mv: Move) -> i32 {
        let (dest, flag) = (mv.dest(), mv.flag());

        match flag {
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
            _ if flag.is_castling() => 0,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn cmp_see(&self, mv: Move, threshold: i32) -> bool {
        if mv.flag().is_castling() {
            return threshold <= 0;
        }

        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let next_victim = flag
            .promotion()
            .unwrap_or_else(|| self.piece_on(src).unwrap());

        let mut balance = -threshold + self.move_value(mv);
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
        let mut attackers = self.attackers_to(dest, occupied);
        let mut duck = mv.duck().bitboard();
        let mut stm = !self.stm;

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
