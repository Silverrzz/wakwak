use crate::board::Board;
use crate::common::{Color, Move, Piece, Square};
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct NoisyEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct NoisyHistory {
    // Indexing: [stm][piece][dest][victim]
    entries: [[[[NoisyEntry; Piece::COUNT + 1]; Square::COUNT]; Piece::COUNT]; Color::COUNT],
}

impl NoisyHistory {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i32 {
        let dest = mv.dest();
        let piece = board.piece_on(mv.src()).unwrap();
        let victim = board.piece_on(mv.dest()).map_or(0, |p| p as usize + 1);

        self.entries[board.stm()][piece][dest][victim].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let dest = mv.dest();
        let piece = board.piece_on(mv.src()).unwrap();
        let victim = board.piece_on(mv.dest()).map_or(0, |p| p as usize + 1);

        &mut self.entries[board.stm()][piece][dest][victim].0
    }

    #[inline]
    pub fn update<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        let amount = if BONUS {
            Params::noisy_bonus(depth)
        } else {
            Params::noisy_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}
