use crate::board::Board;
use crate::common::{Color, Move, Square};
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct QuietEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct QuietHistory {
    // Indexing: [stm][src][dest]
    entries: [[[QuietEntry; Square::COUNT]; Square::COUNT]; Color::COUNT],
}

impl QuietHistory {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i32 {
        let (src, dest) = (mv.src(), mv.dest());

        self.entries[board.stm()][src][dest].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let (src, dest) = (mv.src(), mv.dest());

        &mut self.entries[board.stm()][src][dest].0
    }

    #[inline]
    pub fn update<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        let amount = if BONUS {
            Params::quiet_bonus(depth)
        } else {
            Params::quiet_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}
