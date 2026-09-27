use crate::common::Move;
use crate::score::Score;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TTFlag {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl TTFlag {
    #[inline]
    pub fn bounds_match(&self, score: Score, lower: Score, upper: Score) -> bool {
        match self {
            TTFlag::None => false,
            TTFlag::Exact => true,
            TTFlag::Lower => score >= upper,
            TTFlag::Upper => score <= lower,
        }
    }
}

const MOVE_SHIFT: u32 = 15;
const SCORE_SHIFT: u32 = 37;
const DEPTH_SHIFT: u32 = 53;
const FLAG_SHIFT: u32 = 61;
const PV_SHIFT: u32 = 63;

#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    pub key: u16,
    pub best_move: Option<Move>,
    pub score: Score,
    pub depth: i32,
    pub flag: TTFlag,
    pub pv: bool,
}

impl TTEntry {
    #[inline]
    pub fn unpack(bits: u64) -> Self {
        Self {
            key: (bits & 0x7FFF) as u16,
            best_move: unsafe { Move::from_raw((bits >> MOVE_SHIFT) as u32 & 0x3FFFFF) },
            score: Score(((bits >> SCORE_SHIFT) as u16 as i16) as i32),
            depth: (bits >> DEPTH_SHIFT) as u8 as i32,
            flag: unsafe { std::mem::transmute::<u8, TTFlag>((bits >> FLAG_SHIFT) as u8 & 0x3) },
            pv: (bits >> PV_SHIFT) != 0,
        }
    }

    #[inline]
    pub fn pack(&self) -> u64 {
        let mut bits = 0;
        bits |= (self.key as u64) & 0x7FFF;
        bits |= (self.best_move.map_or(0, |mv| mv.raw().get() as u64)) << MOVE_SHIFT;
        bits |= ((self.score.0 as i16 as u16) as u64) << SCORE_SHIFT;
        bits |= (self.depth as u8 as u64) << DEPTH_SHIFT;
        bits |= (self.flag as u64) << FLAG_SHIFT;
        bits |= (self.pv as u64) << PV_SHIFT;
        bits
    }
}

pub struct AtomicTTEntry(pub AtomicU64);

impl Default for AtomicTTEntry {
    #[inline]
    fn default() -> AtomicTTEntry {
        AtomicTTEntry(AtomicU64::new(0))
    }
}

pub struct TranspositionTable {
    table: Vec<AtomicTTEntry>,
    size: usize,
}

impl TranspositionTable {
    pub const DEFAULT_SIZE_MB: usize = 64;
    pub const MAX_SIZE_MB: usize = 16 * 1024 * 1024; // duck it we ball

    #[inline]
    pub fn new(size_mb: usize) -> TranspositionTable {
        let size = size_mb * 1024 * 1024 / size_of::<AtomicTTEntry>();
        let table = (0..size).map(|_| AtomicTTEntry::default()).collect();
        TranspositionTable { table, size }
    }

    #[inline]
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = self.idx(hash);
        let partial_key = hash as u16 & 0x7FFF;
        let entry = TTEntry::unpack(self.table[idx].0.load(Ordering::Relaxed));

        if entry.key == partial_key {
            Some(entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn insert(
        &self,
        hash: u64,
        best_move: Option<Move>,
        score: Score,
        depth: i32,
        flag: TTFlag,
        pv: bool,
    ) {
        let idx = self.idx(hash);
        let partial_key = hash as u16 & 0x7FFF;
        let old_entry = TTEntry::unpack(self.table[idx].0.load(Ordering::Relaxed));
        let new_entry = TTEntry {
            key: partial_key,
            best_move: best_move.or(old_entry.best_move.filter(|_| old_entry.key == partial_key)),
            score,
            depth,
            flag,
            pv,
        };

        self.table[idx].0.store(new_entry.pack(), Ordering::Relaxed);
    }

    #[inline]
    pub fn clear(&self) {
        self.table.iter().for_each(|entry| {
            entry.0.store(0, Ordering::Relaxed);
        });
    }

    #[inline]
    fn idx(&self, hash: u64) -> usize {
        let key = hash as u128;
        let len = self.size as u128;
        ((key * len) >> 64) as usize
    }
}

impl Default for TranspositionTable {
    #[inline]
    fn default() -> TranspositionTable {
        TranspositionTable::new(TranspositionTable::DEFAULT_SIZE_MB)
    }
}
