use crate::board::Board;
use crate::common::Move;

#[derive(Clone)]
pub struct Position {
    current: Board,
    previous_boards: Vec<Board>,
}

impl Position {
    #[inline]
    pub fn new(board: Board) -> Self {
        Self {
            current: board,
            previous_boards: Vec::new(),
        }
    }

    #[inline]
    pub fn reset(&mut self, board: Board) {
        self.current = board;
        self.previous_boards.clear();
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn make_move(&mut self, _mv: Move) {
        self.previous_boards.push(self.current);
        //self.current.make_move(mv);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
    }
}
