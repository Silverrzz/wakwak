use crate::board::Board;
use crate::common::{Bitboard, Color, Move, Square};
use crate::search::{
    FactorisedThreatBucket, MAX_FACTORISER, MAX_HISTORY, Params, ThreatIndex, gravity,
};

#[derive(Debug, Copy, Clone)]
pub struct QuietEntry(pub i16);

#[derive(Debug, Copy, Clone)]
pub struct QuietHistory {
    // Indexing: [stm][src][dest]
    entries: [[[FactorisedThreatBucket<QuietEntry>; Square::COUNT]; Square::COUNT]; Color::COUNT],
}

impl QuietHistory {
    #[inline]
    pub fn entry(&self, board: &Board, threats: Bitboard, mv: Move) -> i32 {
        let (src, dest) = (mv.src(), mv.dest());
        let threat_idx = ThreatIndex::new(mv, threats);
        let entry = &self.entries[board.stm()][src][dest];
        entry.bucket[threat_idx.src()][threat_idx.dest()].0 as i32 + entry.factoriser as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, board: &Board, threats: Bitboard, mv: Move) -> &mut i16 {
        let (src, dest) = (mv.src(), mv.dest());
        let threat_idx = ThreatIndex::new(mv, threats);

        &mut self.entries[board.stm()][src][dest].bucket[threat_idx.src()][threat_idx.dest()].0
    }

    #[inline]
    pub fn factoriser_mut(&mut self, board: &Board, mv: Move) -> &mut i16 {
        let (src, dest) = (mv.src(), mv.dest());
        &mut self.entries[board.stm()][src][dest].factoriser
    }

    #[inline]
    pub fn update<const BONUS: bool>(
        &mut self,
        board: &Board,
        threats: Bitboard,
        depth: i32,
        mv: Move,
    ) {
        let entry_amount = if BONUS {
            Params::quiet_bonus(depth)
        } else {
            Params::quiet_malus(depth)
        };

        let factoriser_amount = if BONUS {
            Params::quiet_factoriser_bonus(depth)
        } else {
            Params::quiet_factoriser_malus(depth)
        };

        gravity::<MAX_HISTORY, MAX_HISTORY>(self.entry_mut(board, threats, mv), entry_amount);
        gravity::<MAX_FACTORISER, MAX_FACTORISER>(
            self.factoriser_mut(board, mv),
            factoriser_amount,
        );
    }
}
