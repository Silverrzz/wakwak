use crate::common::Move;
use crate::score::Score;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub struct TranspositionTable {
    table: Vec<AtomicTTEntry>,
    size: usize,
}

struct AtomicTTEntry {
    packed: AtomicU64,
    best_move: AtomicU32,
}

impl Default for AtomicTTEntry {
    fn default() -> AtomicTTEntry {
        AtomicTTEntry {
            packed: AtomicU64::new(0),
            best_move: AtomicU32::new(0),
        }
    }
}

const KEY_SHIFT: u32 = 0;
const SCORE_SHIFT: u32 = 16;
const DEPTH_SHIFT: u32 = 32;
const FLAG_SHIFT: u32 = 40;

#[derive(Clone, Copy)]
pub struct TTEntry {
    key: u16,       // 2 bytes
    best_move: u32, // 4 bytes
    score: i16,     // 2 bytes
    depth: u8,      // 1 byte
    flag: u8,       // 1 byte
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum TTFlag {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl TTEntry {
    pub fn best_move(&self) -> Option<Move> {
        Move::from_raw(self.best_move)
    }

    pub fn score(&self) -> i16 {
        self.score
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn flag(&self) -> TTFlag {
        match self.flag {
            0 => TTFlag::None,
            1 => TTFlag::Exact,
            2 => TTFlag::Lower,
            3 => TTFlag::Upper,
            _ => unreachable!("invalid TT flag byte"),
        }
    }

    pub fn validate_key(&self, key: u64) -> bool {
        self.key == (key & 0xFFFF) as u16
    }
}

impl Default for TranspositionTable {
    fn default() -> TranspositionTable {
        TranspositionTable::new(TranspositionTable::DEFAULT_SIZE_MB)
    }
}

impl TranspositionTable {
    pub const DEFAULT_SIZE_MB: usize = 64;
    pub const MAX_SIZE_MB: usize = 16 * 1024 * 1024; // duck it we ball

    pub fn new(size_mb: usize) -> TranspositionTable {
        let size = size_mb * 1024 * 1024 / size_of::<AtomicTTEntry>();
        let table = (0..size).map(|_| AtomicTTEntry::default()).collect();
        TranspositionTable { table, size }
    }

    pub fn clear(&self) {
        self.table.iter().for_each(|entry| {
            entry.packed.store(0, Ordering::Relaxed);
            entry.best_move.store(0, Ordering::Relaxed);
        });
    }

    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = self.idx(hash);
        let atomic_entry = &self.table[idx];
        let packed = atomic_entry.packed.load(Ordering::Relaxed);

        let entry = TTEntry {
            key: (packed >> KEY_SHIFT) as u16,
            best_move: atomic_entry.best_move.load(Ordering::Relaxed),
            score: (packed >> SCORE_SHIFT) as u16 as i16,
            depth: (packed >> DEPTH_SHIFT) as u8,
            flag: (packed >> FLAG_SHIFT) as u8,
        };

        if entry.validate_key(hash) {
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&self, hash: u64, best_move: Option<Move>, score: i32, depth: u8, flag: TTFlag) {
        let idx = self.idx(hash);
        let entry = &self.table[idx];

        let key = hash as u16;
        let packed = (key as u64) << KEY_SHIFT
            | ((score as u16) as u64) << SCORE_SHIFT
            | (depth as u64) << DEPTH_SHIFT
            | (flag as u64) << FLAG_SHIFT;

        entry
            .best_move
            .store(best_move.map_or(0, |mv| mv.raw().get()), Ordering::Relaxed);
        entry.packed.store(packed, Ordering::Relaxed);
    }

    fn idx(&self, hash: u64) -> usize {
        let key = hash as u128;
        let len = self.size as u128;
        ((key * len) >> 64) as usize
    }
}

impl TTFlag {
    pub fn bounds_match(&self, score: Score, lower: Score, upper: Score) -> bool {
        match self {
            TTFlag::None => false,
            TTFlag::Exact => true,
            TTFlag::Lower => score >= upper,
            TTFlag::Upper => score <= lower,
        }
    }
}
