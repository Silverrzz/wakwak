pub mod cont;
pub mod corr;
pub mod duck;
pub mod noisy;
pub mod quiet;

use crate::board::Board;
use crate::common::Move;
use crate::score::Score;
use crate::search::Params;
pub use cont::*;
pub use corr::*;
pub use duck::*;
pub use noisy::*;
pub use quiet::*;

pub const MAX_HISTORY: i32 = 16384;
pub const PAWN_CORR_SIZE: usize = 4096;
pub const MINOR_CORR_SIZE: usize = 16384;
pub const MAJOR_CORR_SIZE: usize = 16384;
pub const NONPAWN_CORR_SIZE: usize = 16384;

pub struct History {
    quiet: QuietHistory,
    noisy: NoisyHistory,
    duck: DuckHistory,
    cont_odd: ContHistory,
    cont_even: ContHistory,
    pawn_corr: CorrHistory<PAWN_CORR_SIZE>,
    minor_corr: CorrHistory<MINOR_CORR_SIZE>,
    major_corr: CorrHistory<MAJOR_CORR_SIZE>,
    white_corr: CorrHistory<NONPAWN_CORR_SIZE>,
    black_corr: CorrHistory<NONPAWN_CORR_SIZE>,
}

impl History {
    #[inline]
    pub fn update(
        &mut self,
        board: &Board,
        indices: ContIndices,
        depth: i32,
        best_move: Move,
        failed_quiets: &[Move],
        failed_noisies: &[Move],
    ) {
        let best_duck = best_move.duck();
        let (best_src, best_dest) = (best_move.src(), best_move.dest());
        if best_move.flag().is_noisy() {
            self.update_noisy::<true>(board, depth, best_move);
        } else {
            self.update_quiet::<true>(board, indices, depth, best_move);

            // Only give malus to failed quiets when best move is quiet
            for &quiet in failed_quiets {
                if (quiet.src(), quiet.dest()) != (best_src, best_dest) {
                    self.update_quiet::<false>(board, indices, depth, quiet);
                }
            }
        }

        // Always give malus to failed noisies
        for &noisy in failed_noisies {
            if (noisy.src(), noisy.dest()) != (best_src, best_dest) {
                self.update_noisy::<false>(board, depth, noisy);
            }
        }

        self.update_duck::<true>(board, depth, best_move);
        for &quiet in failed_quiets {
            if quiet.duck() != best_duck {
                self.update_duck::<false>(board, depth, quiet);
            }
        }
        for &noisy in failed_noisies {
            if noisy.duck() != best_duck {
                self.update_duck::<false>(board, depth, noisy);
            }
        }
    }

    #[inline]
    pub fn update_corr(&mut self, board: &Board, depth: i32, score: Score, static_eval: Score) {
        let stm = board.stm();
        let diff = score.0 as i64 - static_eval.0 as i64;

        self.pawn_corr.update(stm, board.pawn_hash(), depth, diff);
        self.minor_corr.update(stm, board.minor_hash(), depth, diff);
        self.major_corr.update(stm, board.major_hash(), depth, diff);
        self.white_corr.update(stm, board.white_hash(), depth, diff);
        self.black_corr.update(stm, board.black_hash(), depth, diff);
    }

    #[inline]
    fn update_quiet<const BONUS: bool>(
        &mut self,
        board: &Board,
        indices: ContIndices,
        depth: i32,
        mv: Move,
    ) {
        self.quiet.update::<BONUS>(board, depth, mv);
        self.cont_odd
            .update::<1, BONUS>(board, depth, mv, indices.cont1);
        self.cont_even
            .update::<2, BONUS>(board, depth, mv, indices.cont2);
    }

    #[inline]
    fn update_noisy<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        self.noisy.update::<BONUS>(board, depth, mv);
    }

    #[inline]
    fn update_duck<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        self.duck.update::<BONUS>(board, depth, mv);
    }

    #[inline]
    pub fn quiet(&self, board: &Board, mv: Move) -> i32 {
        self.quiet.entry(board, mv)
    }

    #[inline]
    pub fn noisy(&self, board: &Board, mv: Move) -> i32 {
        self.noisy.entry(board, mv)
    }

    #[inline]
    pub fn duck(&self, board: &Board, mv: Move) -> i32 {
        self.duck.entry(board, mv)
    }

    #[inline]
    pub fn cont(&self, board: &Board, indices: ContIndices, mv: Move) -> i32 {
        let mut value = self
            .cont_odd
            .entry(board, mv, indices.cont1)
            .unwrap_or_default();
        value += self
            .cont_even
            .entry(board, mv, indices.cont2)
            .unwrap_or_default();
        value
    }

    #[inline]
    pub fn corr(&self, board: &Board) -> i32 {
        let stm = board.stm();
        let mut corr = 0;

        corr += Params::pawn_corr() * self.pawn_corr.entry(stm, board.pawn_hash());
        corr += Params::minor_corr() * self.minor_corr.entry(stm, board.minor_hash());
        corr += Params::major_corr() * self.major_corr.entry(stm, board.major_hash());
        corr += Params::nonpawn_corr() * self.white_corr.entry(stm, board.white_hash());
        corr += Params::nonpawn_corr() * self.black_corr.entry(stm, board.black_hash());
        corr / MAX_CORR
    }
}

#[inline]
pub fn gravity_with_decay<const MAX_BONUS: i32, const MAX_VALUE: i32>(
    entry: &mut i16,
    decay: i32,
    amount: i32,
) {
    let amount = amount.clamp(-MAX_BONUS, MAX_BONUS);
    let decay = (decay * amount.abs() / MAX_VALUE) as i16;
    *entry += amount as i16 - decay;
}

#[inline]
pub fn gravity<const MAX_BONUS: i32, const MAX_VALUE: i32>(entry: &mut i16, amount: i32) {
    gravity_with_decay::<MAX_BONUS, MAX_VALUE>(entry, *entry as i32, amount);
}
