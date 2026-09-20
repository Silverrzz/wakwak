use crate::board::Board;
use crate::common::{Color, Move, Piece, Square};
use crate::position::Position;
use crate::search::{MAX_HISTORY, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct ContIndices {
    pub cont1: Option<(Piece, Move)>,
    pub cont2: Option<(Piece, Move)>,
    pub cont4: Option<(Piece, Move)>,
}

impl ContIndices {
    #[inline]
    pub fn new(pos: &Position) -> Self {
        Self {
            cont1: pos.prev_move(1),
            cont2: pos.prev_move(2),
            cont4: pos.prev_move(4),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ContEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct ContHistory {
    // Indexing: [stm][prev piece][prev dest][piece][dest]
    entries:
        [[[[[ContEntry; Square::COUNT]; Piece::COUNT]; Square::COUNT]; Piece::COUNT]; Color::COUNT],
}

impl ContHistory {
    #[inline]
    pub fn entry(&self, board: &Board, mv: Move, prev_mv: Option<(Piece, Move)>) -> Option<i32> {
        prev_mv.map(|(prev_piece, prev_mv)| {
            let piece = board.piece_on(mv.src()).unwrap();
            self.entries[board.stm()][prev_piece][prev_mv.dest()][piece][mv.dest()].0 as i32
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
            let piece = board.piece_on(mv.src()).unwrap();
            &mut self.entries[board.stm()][prev_piece][prev_mv.dest()][piece][mv.dest()].0
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
