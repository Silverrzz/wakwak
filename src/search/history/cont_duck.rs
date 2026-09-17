use crate::board::Board;
use crate::common::{Color, Move, Piece, Square};
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct ContDuckEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct ContDuckHistory {
    // Indexing: [stm][prev piece][prev dest][duck]
    entries: [[[[ContDuckEntry; Square::COUNT]; Square::COUNT]; Piece::COUNT]; Color::COUNT],
}

impl ContDuckHistory {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move, prev_mv: Option<(Piece, Move)>) -> Option<i32> {
        prev_mv.map(|(prev_piece, prev_mv)| {
            self.entries[board.stm()][prev_piece][prev_mv.dest()][mv.duck()].0 as i32
        })
    }

    #[inline]
    pub fn entry_mut(
        &mut self,
        board: &Board,
        mv: Move,
        prev_mv: Option<(Piece, Move)>,
    ) -> Option<&mut i16> {
        prev_mv.map(|(prev_piece, prev_mv)| {
            &mut self.entries[board.stm()][prev_piece][prev_mv.dest()][mv.duck()].0
        })
    }

    #[inline]
    pub fn update<const PLY: usize, const BONUS: bool>(
        &mut self,
        board: &Board,
        depth: i32,
        mv: Move,
        prev_mv: Option<(Piece, Move)>,
    ) {
        let amount = if BONUS {
            Params::cont_bonus::<PLY>(depth)
        } else {
            Params::cont_malus::<PLY>(depth)
        };

        if let Some(entry) = self.entry_mut(board, mv, prev_mv) {
            gravity::<MAX_HISTORY, MAX_HISTORY>(entry, amount);
        }
    }
}
