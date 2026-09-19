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
        score: impl Fn(Move) -> i32,
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
            self.stack
                .extend(moves.iter().map(|mv| ScoredMove(mv, score(mv))));
            Abort::No
        });

        self.start[self.ply] = self.stack.len();
        old_len - start
    }

    #[inline]
    pub fn sort_from(&mut self, start: usize) {
        self.get_mut()[start..].sort_unstable_by_key(|m| Reverse(m.1));
    }

    #[inline]
    pub fn pop_ply(&mut self) {
        debug_assert!(self.ply > 0, "MoveStack::pop(): Empty stack");

        self.ply -= 1;
        self.stack.truncate(self.start[self.ply]);
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
    pub fn reset(&mut self) {
        self.stack.clear();
        self.ply = 0;
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
        Params::piece_value(Piece::Pawn)
    } else if mv.flag().is_capture() {
        Params::piece_value(board.piece_on(mv.dest()).unwrap())
    } else {
        0
    };
    let promotion = mv.flag().promotion().map_or(0, |p| {
        Params::piece_value(p) - Params::piece_value(Piece::Pawn)
    });

    victim + promotion
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    TTMove,
    GenerateNoisies,
    YieldNoisies,
    GenerateQuiets,
    YieldQuiets,
    Finished,
}

pub struct MovePicker {
    stage: Stage,
    tt_move: Option<Move>,
    skip_quiets: bool,
    neutral_ducks: Bitboard,
    prune_quiet_neutrals: bool,
    prune_noisy_neutrals: bool,
    cursor: usize,
}

impl MovePicker {
    #[inline]
    pub fn new(
        tt_move: Option<Move>,
        neutral_ducks: Bitboard,
        prune_quiet_neutrals: bool,
        prune_noisy_neutrals: bool,
    ) -> Self {
        Self {
            stage: Stage::TTMove,
            tt_move,
            skip_quiets: false,
            neutral_ducks,
            prune_quiet_neutrals,
            prune_noisy_neutrals,
            cursor: 0,
        }
    }

    #[inline]
    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
        if matches!(self.stage, Stage::GenerateQuiets | Stage::YieldQuiets) {
            self.stage = Stage::Finished;
        }
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
            let history: &History = &thread.history;
            let start = thread.move_stack.add_moves::<Noisy>(
                board,
                self.neutral_ducks,
                self.prune_noisy_neutrals,
                history,
                |mv| self.score_noisy(board, history, mv),
            );
            thread.move_stack.sort_from(start);
            self.stage = Stage::YieldNoisies;
        }

        if self.stage == Stage::YieldNoisies {
            if let Some(mv) = self.yield_next(thread) {
                return Some(mv);
            }

            self.stage = Stage::GenerateQuiets;
        }

        if self.stage == Stage::GenerateQuiets {
            if self.skip_quiets {
                self.stage = Stage::Finished;
            } else {
                let history: &History = &thread.history;
                let start = thread.move_stack.add_moves::<Quiet>(
                    board,
                    self.neutral_ducks,
                    self.prune_quiet_neutrals,
                    history,
                    |mv| self.score_quiet(board, history, indices, mv),
                );
                thread.move_stack.sort_from(start);
                self.stage = Stage::YieldQuiets;
            }
        }

        if self.stage == Stage::YieldQuiets {
            if !self.skip_quiets
                && let Some(mv) = self.yield_next(thread)
            {
                return Some(mv);
            }

            self.stage = Stage::Finished;
        }

        None
    }

    #[inline]
    fn yield_next(&mut self, thread: &ThreadData) -> Option<Move> {
        let moves = thread.move_stack.get();

        while self.cursor < moves.len() {
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
    fn score_noisy(&self, board: &Board, history: &History, mv: Move) -> i32 {
        if self.tt_move == Some(mv) {
            return 0;
        }

        mvv(board, mv) * 8 + history.noisy(board, mv) / 8 + history.duck(board, mv) / 8
    }

    #[inline]
    fn score_quiet(&self, board: &Board, history: &History, indices: ContIndices, mv: Move) -> i32 {
        if self.tt_move == Some(mv) {
            return 0;
        }
        let is_neutral = self.neutral_ducks.has(mv.duck());

        history.quiet(board, mv) + history.duck(board, mv) + history.cont(board, indices, mv)
            - Params::mp_quiet_neutral_malus() * is_neutral as i32
    }
}
