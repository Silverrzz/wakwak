use crate::common::{Color, Move, Piece, Square};
use crate::position::Position;
use crate::search::{MAX_CORR, Params, gravity};

#[derive(Debug, Copy, Clone)]
pub struct ContCorrIndices {
    pub prev_move: Option<(Piece, Move)>,
    pub cont1: Option<(Piece, Move)>,
    pub cont2: Option<(Piece, Move)>,
}

impl ContCorrIndices {
    #[inline]
    pub fn new(pos: &Position) -> Self {
        Self {
            prev_move: pos.prev_move(1),
            cont1: pos.prev_move(2),
            cont2: pos.prev_move(3),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ContCorrEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct ContCorrHistory {
    // Indexing: [stm][prev piece][prev dest][piece][dest]
    entries: [[[[[ContCorrEntry; Square::COUNT]; Piece::COUNT]; Square::COUNT]; Piece::COUNT];
        Color::COUNT],
}

impl ContCorrHistory {
    #[inline]
    pub fn entry(
        &self,
        stm: Color,
        mv: Option<(Piece, Move)>,
        prev_mv: Option<(Piece, Move)>,
    ) -> Option<i32> {
        let mv = mv?;
        let prev_mv = prev_mv?;

        Some(self.entries[stm][prev_mv.0][prev_mv.1.dest()][mv.0][mv.1.dest()].0 as i32)
    }

    #[inline]
    pub fn entry_mut(
        &mut self,
        stm: Color,
        mv: Option<(Piece, Move)>,
        prev_mv: Option<(Piece, Move)>,
    ) -> Option<&mut i16> {
        let mv = mv?;
        let prev_mv = prev_mv?;

        Some(&mut self.entries[stm][prev_mv.0][prev_mv.1.dest()][mv.0][mv.1.dest()].0)
    }

    #[inline]
    pub fn update(
        &mut self,
        stm: Color,
        mv: Option<(Piece, Move)>,
        prev_mv: Option<(Piece, Move)>,
        depth: i32,
        diff: i64,
    ) {
        if let Some(entry) = self.entry_mut(stm, mv, prev_mv) {
            gravity::<{ MAX_CORR / 4 }, MAX_CORR>(entry, Params::corr_bonus(depth, diff));
        }
    }
}
