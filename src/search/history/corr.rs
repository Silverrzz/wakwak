use crate::common::Color;
use crate::search::{Params, gravity};

pub const MAX_CORR: i32 = 1024;

#[derive(Debug, Copy, Clone)]
pub struct CorrEntry(i16);

#[derive(Debug, Copy, Clone)]
pub struct CorrHistory<const SIZE: usize> {
    // Indexing: [stm][hash % SIZE]
    entries: [[CorrEntry; SIZE]; Color::COUNT],
}

impl<const SIZE: usize> CorrHistory<SIZE> {
    #[inline]
    pub fn entry(&self, stm: Color, hash: u64) -> i32 {
        self.entries[stm][(hash % SIZE as u64) as usize].0 as i32
    }

    #[inline]
    pub fn entry_mut(&mut self, stm: Color, hash: u64) -> &mut i16 {
        &mut self.entries[stm][(hash % SIZE as u64) as usize].0
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn update(&mut self, stm: Color, hash: u64, depth: i32, diff: i64) {
        gravity::<{ MAX_CORR / 4 }, MAX_CORR>(
            self.entry_mut(stm, hash),
            Params::corr_bonus(depth, diff),
        );
    }
}
