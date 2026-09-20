use crate::board::Board;
use crate::common::{Move, Piece};
use crate::nnue::Nnue;
use crate::score::Score;
use crate::search::MAX_PLY;

#[derive(Clone)]
pub struct Position {
    current: Board,
    previous_boards: Vec<Board>,
    previous_moves: Vec<Option<(Piece, Move)>>,
    nnue: Nnue,
}

impl Position {
    #[inline]
    pub fn new(board: Board) -> Self {
        Self {
            current: board,
            previous_boards: Vec::with_capacity(MAX_PLY),
            previous_moves: Vec::with_capacity(MAX_PLY),
            nnue: Nnue::new(&board),
        }
    }

    #[inline]
    pub fn reset(&mut self, board: Board) {
        self.current = board;
        self.previous_boards.clear();
        self.previous_moves.clear();
        self.nnue.full_reset(&board);
    }

    #[inline]
    pub fn reset_nnue(&mut self) {
        self.nnue.full_reset(&self.current);
    }

    #[inline]
    pub fn make_move(&mut self, mv: Move) {
        let piece = self.current.piece_on(mv.src()).unwrap();

        self.previous_boards.push(self.current);
        self.previous_moves.push(Some((piece, mv)));
        self.nnue.make_move(&self.current, mv);
        self.current.make_move(mv);
    }

    #[inline]
    pub fn make_null_move(&mut self) {
        self.previous_boards.push(self.current);
        self.previous_moves.push(None);
        self.nnue.make_null_move(&self.current, None);
        self.current.make_null_move(None);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
        self.previous_moves.pop().unwrap();
        self.nnue.unmake_move();
    }

    // TODO: When going from 768 to 832 inputs, remove this and replace the calls with `unmake_move` calls
    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
        self.previous_moves.pop().unwrap();
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn eval(&mut self) -> Score {
        self.nnue.update(&self.current);
        self.nnue.eval(self.current.stm())
    }

    #[inline]
    pub fn prev_move(&self, ply: usize) -> Option<(Piece, Move)> {
        self.previous_moves
            .len()
            .checked_sub(ply)
            .and_then(|i| self.previous_moves[i])
    }

    #[inline]
    pub fn repetition(&self) -> bool {
        self.previous_boards
            .iter()
            .rev()
            .take(self.current.hmc() as usize)
            .any(|b| b.duckless_hash() == self.current.duckless_hash())
    }
}
