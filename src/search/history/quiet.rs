use crate::board::Board;
use crate::common::{Bitboard, Color, Move, Square};
use crate::search::history::ThreatBucket;
use crate::search::{MAX_HISTORY, Params, ThreatIndex, gravity};

#[derive(Debug, Copy, Clone)]
pub struct QuietEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct QuietHistory {
    // Indexing: [stm][src][dest]
    entries: [[[ThreatBucket<QuietEntry>; Square::COUNT]; Square::COUNT]; Color::COUNT],
}

impl QuietHistory {
    #[inline]
    pub fn entry(&self, board: &Board, threats: Bitboard, mv: Move) -> i32 {
        let (src, dest) = (mv.src(), mv.dest());
        let threat_idx = ThreatIndex::new(mv, threats);

        self.entries[board.stm()][src][dest][threat_idx.src()][threat_idx.dest()].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, threats: Bitboard, mv: Move) -> &mut i16 {
        let (src, dest) = (mv.src(), mv.dest());
        let threat_idx = ThreatIndex::new(mv, threats);

        &mut self.entries[board.stm()][src][dest][threat_idx.src()][threat_idx.dest()].0
    }

    #[inline]
    pub fn update<const BONUS: bool>(
        &mut self,
        board: &Board,
        threats: Bitboard,
        depth: i32,
        mv: Move,
    ) {
        let amount = if BONUS {
            Params::quiet_bonus(depth)
        } else {
            Params::quiet_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, threats, mv), amount);
    }
}
