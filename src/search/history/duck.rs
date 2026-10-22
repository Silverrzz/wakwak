use crate::board::Board;
use crate::common::{Color, Move, Square};
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct DuckEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct DuckHistory {
    // Indexing: [stm][duck_sq]
    entries: [[DuckEntry; Square::COUNT]; Color::COUNT],
}

impl DuckHistory {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i32 {
        let duck = mv.duck();

        self.entries[board.stm()][duck].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let duck = mv.duck();

        &mut self.entries[board.stm()][duck].0
    }

    #[inline]
    pub fn update<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        let amount = if BONUS {
            Params::duck_bonus(depth)
        } else {
            Params::duck_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}
