use crate::board::Board;
use crate::common::{Color, Move, Piece, Square};
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct PawnEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct PawnHistory<const SIZE: usize> {
    // Indexing: [stm][hash % SIZE][piece][dest]
    entries: [[[[PawnEntry; Square::COUNT]; Piece::COUNT]; SIZE]; Color::COUNT],
}

impl<const SIZE: usize> PawnHistory<SIZE> {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move) -> i32 {
        let piece = board.piece_on(mv.src()).unwrap();
        let hash = board.pawn_hash();
        let dest = mv.dest();

        self.entries[board.stm()][(hash % SIZE as u64) as usize][piece][dest].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let piece = board.piece_on(mv.src()).unwrap();
        let hash = board.pawn_hash();
        let dest = mv.dest();

        &mut self.entries[board.stm()][(hash % SIZE as u64) as usize][piece][dest].0
    }

    #[inline]
    pub fn update<const BONUS: bool>(&mut self, board: &Board, depth: i32, mv: Move) {
        let amount = if BONUS {
            Params::pawn_bonus(depth)
        } else {
            Params::pawn_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, mv), amount);
    }
}
