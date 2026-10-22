use crate::{
    board::Board,
    common::{Color, Move, Square},
    search::{MAX_HISTORY, Params, gravity},
};

#[derive(Clone, Copy)]
pub struct DuckCountEntry(pub i16);

pub struct DuckCountHistory {
    // indexing: [stm][duck_sq]
    entries: [[DuckCountEntry; Square::COUNT]; Color::COUNT],
}

impl DuckCountHistory {
    #[inline]
    pub fn init(&mut self) {
        self.entries = [[DuckCountEntry(2); Square::COUNT]; Color::COUNT];
    }

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
    pub fn update(&mut self, board: &Board, depth: i32, mv: Move, duck_count: u8) {
        let diff = duck_count as i32 * Params::duck_count_hist_scale() - self.entry(board, mv);
        gravity::<{ MAX_HISTORY / 4 }, MAX_HISTORY>(self.entry_mut(board, mv), diff * depth / 10);
    }
}
