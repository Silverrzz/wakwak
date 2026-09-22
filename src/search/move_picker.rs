use crate::board::{Board, MoveFilter, Noisy, Quiet};
use crate::common::{Bitboard, Move, MoveFlag, Piece};
use crate::position::Position;
use crate::search::cont::ContIndices;
use crate::search::{History, MAX_PLY, Params, ThreadData};
use crate::util::Abort;
use std::cmp::Reverse;

pub struct ScoredMove(Move, i32);

pub struct MoveStack {
    stack: Vec<ScoredMove>,
    start: [usize; MAX_PLY + 1],
    ply: usize,
}

impl MoveStack {
    #[inline]
    pub fn push_ply(&mut self) {
        debug_assert!(
            self.ply < MAX_PLY,
            "MoveStack::push(): Attempted to push on ply `MAX_PLY`"
        );

        // self.stack.truncate(self.start[self.ply]);

        self.start[self.ply + 1] = self.start[self.ply];
        self.ply += 1;
    }

    #[inline]
    pub fn add_moves<F: MoveFilter>(
        &mut self,
        board: &Board,
        neutral_ducks: Bitboard,
        prune_neutral_ducks: bool,
        history: &History,
    ) -> usize {
        let start = self.start[self.ply - 1];
        let old_len = self.stack.len();

        board.gen_moves::<F, _>(|mut moves| {
            if prune_neutral_ducks
                && let Some(duck) = (moves.duck & neutral_ducks).iter().max_by_key(|&duck| {
                    history.duck(board, Move::new(moves.src, moves.dest, duck, moves.flag))
                })
            {
                moves.duck &= !neutral_ducks | duck;
            }
            self.stack.extend(moves.iter().map(|w| ScoredMove(w, 0)));
            Abort::No
        });

        self.start[self.ply] = self.stack.len();
        old_len - start
    }

    #[inline]
    pub fn pop_ply(&mut self) {
        debug_assert!(self.ply > 0, "MoveStack::pop(): Empty stack");

        self.ply -= 1;
        self.stack.truncate(self.start[self.ply]);
    }

    #[inline]
    pub fn reset(&mut self) {
        self.stack.clear();
        self.ply = 0;
    }

    #[inline]
    pub fn get(&self) -> &[ScoredMove] {
        debug_assert!(self.ply > 0, "MoveStack::get(): Empty stack");

        &self.stack[self.start[self.ply - 1]..]
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut [ScoredMove] {
        debug_assert!(self.ply > 0, "MoveStack::get_mut(): Empty stack");

        &mut self.stack[self.start[self.ply - 1]..]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.get().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get().is_empty()
    }
}

impl Default for MoveStack {
    #[inline]
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            start: [0; MAX_PLY + 1],
            ply: 0,
        }
    }
}

#[inline]
fn mvv(board: &Board, mv: Move) -> i32 {
    let victim = if mv.flag() == MoveFlag::EnPassant {
        Params::mvv_value(Piece::Pawn)
    } else if mv.flag().is_capture() {
        Params::mvv_value(board.piece_on(mv.dest()).unwrap())
    } else {
        0
    };
    let promotion = mv
        .flag()
        .promotion()
        .map_or(0, |p| Params::mvv_value(p) - Params::mvv_value(Piece::Pawn));

    victim + promotion
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    TTMove,
    GenerateNoisies,
    YieldGoodNoisies,
    GenerateQuiets,
    YieldQuiets,
    YieldBadNoisies,
    Finished,
}

pub struct MovePicker {
    stage: Stage,
    tt_move: Option<Move>,
    see_threshold: i32,
    skip_quiets: bool,
    skip_bad_noisies: bool,
    neutral_ducks: Bitboard,
    prune_quiet_neutrals: bool,
    prune_noisy_neutrals: bool,
    bad_noisy_count: usize,
    cursor: usize,
}

impl MovePicker {
    #[inline]
    pub fn new(
        tt_move: Option<Move>,
        see_threshold: i32,
        neutral_ducks: Bitboard,
        prune_quiet_neutrals: bool,
        prune_noisy_neutrals: bool,
    ) -> Self {
        Self {
            stage: Stage::TTMove,
            tt_move,
            see_threshold,
            skip_quiets: false,
            skip_bad_noisies: false,
            neutral_ducks,
            prune_quiet_neutrals,
            prune_noisy_neutrals,
            bad_noisy_count: 0,
            cursor: 0,
        }
    }

    #[inline]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    #[inline]
    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
    }

    #[inline]
    pub fn skip_bad_noisies(&mut self) {
        self.skip_bad_noisies = true;
    }

    pub fn next(
        &mut self,
        pos: &Position,
        thread: &mut ThreadData,
        indices: ContIndices,
    ) -> Option<Move> {
        let board = pos.board();
        if self.stage == Stage::TTMove {
            self.stage = Stage::GenerateNoisies;
            if let Some(mv) = self.tt_move
                && board.is_legal(mv)
            {
                return Some(mv);
            }
        }

        if self.stage == Stage::GenerateNoisies {
            let start = thread.move_stack.add_moves::<Noisy>(
                board,
                self.neutral_ducks,
                self.prune_noisy_neutrals,
                &thread.history,
            );
            self.score_noisies(board, thread, start);
            self.stage = Stage::YieldGoodNoisies;
        }

        if self.stage == Stage::YieldGoodNoisies {
            if let Some(mv) = self.yield_good_noisy(board, thread) {
                return Some(mv);
            }

            self.stage = Stage::GenerateQuiets;
        }

        if self.stage == Stage::GenerateQuiets {
            if self.skip_quiets {
                self.stage = Stage::YieldBadNoisies;
            } else {
                let start = thread.move_stack.add_moves::<Quiet>(
                    board,
                    self.neutral_ducks,
                    self.prune_quiet_neutrals,
                    &thread.history,
                );
                self.score_quiets(board, thread, indices, start);
                self.stage = Stage::YieldQuiets;
            }
        }

        if self.stage == Stage::YieldQuiets {
            if !self.skip_quiets
                && let Some(mv) = self.yield_until(thread, thread.move_stack.len())
            {
                return Some(mv);
            }

            self.stage = Stage::YieldBadNoisies;
            self.cursor = 0;
        }

        if self.stage == Stage::YieldBadNoisies {
            if !self.skip_bad_noisies
                && let Some(mv) = self.yield_until(thread, self.bad_noisy_count)
            {
                return Some(mv);
            }

            self.stage = Stage::Finished;
        }

        None
    }

    #[inline]
    fn yield_good_noisy(&mut self, board: &Board, thread: &mut ThreadData) -> Option<Move> {
        let moves = thread.move_stack.get_mut();

        while self.cursor < moves.len() {
            let mv = moves[self.cursor].0;
            self.cursor += 1;

            // Don't yield the TT move a second time
            if self.tt_move == Some(mv) {
                continue;
            }

            if board.cmp_see(mv, self.see_threshold) {
                return Some(mv);
            }

            moves.swap(self.bad_noisy_count, self.cursor - 1);
            self.bad_noisy_count += 1;
        }

        None
    }

    #[inline]
    fn yield_until(&mut self, thread: &ThreadData, index: usize) -> Option<Move> {
        let moves = thread.move_stack.get();

        while self.cursor < index {
            let mv = moves[self.cursor].0;
            self.cursor += 1;

            // Don't yield the TT move a second time
            if self.tt_move != Some(mv) {
                return Some(mv);
            }
        }

        None
    }

    #[inline]
    fn score_noisies(&self, board: &Board, thread: &mut ThreadData, start: usize) {
        let moves = thread.move_stack.get_mut();

        for scored in moves[start..].iter_mut() {
            let mv = scored.0;
            if self.tt_move == Some(mv) {
                continue;
            }

            scored.1 =
                mvv(board, mv) * 8 + Params::noisy_mp_history(thread.history.as_ref(), board, mv);
        }

        moves[start..].sort_unstable_by_key(|m| Reverse(m.1));
    }

    #[inline]
    fn score_quiets(
        &self,
        board: &Board,
        thread: &mut ThreadData,
        indices: ContIndices,
        start: usize,
    ) {
        let moves = thread.move_stack.get_mut();

        for scored in moves[start..].iter_mut() {
            let mv = scored.0;
            if self.tt_move == Some(mv) {
                continue;
            }
            let is_neutral = self.neutral_ducks.has(mv.duck());

            scored.1 = Params::quiet_mp_history(thread.history.as_ref(), board, indices, mv)
                - Params::mp_quiet_neutral_malus() * is_neutral as i32;
        }

        moves[start..].sort_unstable_by_key(|m| Reverse(m.1));
    }
}
