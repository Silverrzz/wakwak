use crate::board::Board;
use crate::common::{Move, MoveFlag, Piece};
use crate::position::Position;
use crate::search::{MAX_PLY, Params, ThreadData};
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
    pub fn push(&mut self, board: &Board) {
        debug_assert!(
            self.ply < MAX_PLY,
            "MoveStack::push(): Attempted to push on ply `MAX_PLY`"
        );

        self.stack.truncate(self.start[self.ply]);

        let mut cursor = self.start[self.ply];
        board.gen_moves(|moves| {
            self.stack.extend(moves.iter().map(|w| ScoredMove(w, 0)));
            cursor += moves.len();
            Abort::No
        });

        self.start[self.ply + 1] = cursor;
        self.ply += 1;
    }

    #[inline]
    pub fn pop(&mut self) {
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
    SplitNoisy,
    YieldNoisy,
    YieldQuiet,
    Finished,
}

pub struct MovePicker {
    stage: Stage,
    skip_quiets: bool,
    noisy_count: usize,
    cursor: usize,
}

impl MovePicker {
    #[inline]
    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
        if matches!(self.stage, Stage::YieldQuiet) {
            self.stage = Stage::Finished;
        }
    }

    pub fn next(&mut self, pos: &Position, thread: &mut ThreadData) -> Option<Move> {
        let moves = thread.move_stack.get_mut();
        let board = pos.board();

        if self.stage == Stage::SplitNoisy {
            // Move all noisies to the front of the list
            let mut i = 0;
            for j in 0..moves.len() {
                let mv = moves[j].0;
                if moves[j].0.flag().is_noisy() {
                    // Score noisies here (moves[j].1 = pluh)
                    moves[j].1 = mvv(board, mv) * 8
                        + thread.history.noisy(pos.board(), mv) / 8
                        + thread.history.duck(pos.board(), mv) / 8;
                    moves.swap(i, j);
                    i += 1;
                } else {
                    // Score quiets here (moves[j].1 = pluh)
                    moves[j].1 =
                        thread.history.quiet(board, mv) + thread.history.duck(pos.board(), mv);
                }
            }

            self.noisy_count = i;
            self.stage = Stage::YieldNoisy;

            moves[..self.noisy_count].sort_unstable_by_key(|m| Reverse(m.1));
        }

        if self.stage == Stage::YieldNoisy {
            if self.cursor >= self.noisy_count {
                self.stage = Stage::YieldQuiet;
                moves[self.noisy_count..].sort_unstable_by_key(|m| Reverse(m.1));
            } else {
                self.cursor += 1;
                return Some(moves[self.cursor - 1].0);
            }
        }

        if self.stage == Stage::YieldQuiet {
            if self.skip_quiets {
                self.stage = Stage::Finished;
            } else {
                if self.cursor < self.noisy_count {
                    self.cursor = self.noisy_count;
                }

                if self.cursor >= moves.len() {
                    self.stage = Stage::Finished;
                } else {
                    self.cursor += 1;
                    return Some(moves[self.cursor - 1].0);
                }
            }
        }

        None
    }
}

impl Default for MovePicker {
    #[inline]
    fn default() -> Self {
        Self {
            stage: Stage::SplitNoisy,
            skip_quiets: false,
            noisy_count: 0,
            cursor: 0,
        }
    }
}
