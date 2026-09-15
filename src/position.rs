use crate::board::Board;
use crate::common::{Move, Piece};
use crate::search::MAX_PLY;

#[derive(Clone)]
pub struct Position {
    current: Board,
    previous_boards: Vec<Board>,
    previous_moves: Vec<Option<(Piece, Move)>>, // idk maybe we'll have null moves in the future
}

impl Position {
    #[inline]
    pub fn new(board: Board) -> Self {
        Self {
            current: board,
            previous_boards: Vec::with_capacity(MAX_PLY),
            previous_moves: Vec::with_capacity(MAX_PLY),
        }
    }

    #[inline]
    pub fn reset(&mut self, board: Board) {
        self.current = board;
        self.previous_boards.clear();
        self.previous_moves.clear();
    }

    #[inline]
    pub fn make_move(&mut self, mv: Move) {
        let piece = self.current.piece_on(mv.src()).unwrap();

        self.previous_boards.push(self.current);
        self.previous_moves.push(Some((piece, mv)));
        self.current.make_move(mv);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
        self.previous_moves.pop().unwrap();
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
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
