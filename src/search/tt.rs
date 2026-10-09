use crate::common::Move;
use crate::score::Score;
use std::sync::atomic::{AtomicU64, Ordering};

#[inline]
fn score_from_tt(score: Score, ply: usize) -> Score {
    if score.is_loss() {
        score + ply as i32
    } else if score.is_win() {
        score - ply as i32
    } else {
        score
    }
}

#[inline]
fn score_to_tt(score: Score, ply: usize) -> Score {
    if score.is_loss() {
        score - ply as i32
    } else if score.is_win() {
        score + ply as i32
    } else {
        score
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Bound {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl Bound {
    #[inline]
    pub fn matches(&self, score: Score, lower: Score, upper: Score) -> bool {
        match self {
            Bound::None => false,
            Bound::Exact => true,
            Bound::Lower => score >= upper,
            Bound::Upper => score <= lower,
        }
    }
}

const MOVE_SHIFT: u32 = 16;
const SCORE_SHIFT: u32 = 38;
const DEPTH_SHIFT: u32 = 54;
const BOUND_SHIFT: u32 = 62;

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub key: u16,                // 2 bytes
    pub best_move: Option<Move>, // 4 bytes
    pub score: Score,
    pub depth: i32,
    pub bound: Bound,
}

impl TTEntry {
    #[inline]
    pub fn unpack(bits: u64) -> Self {
        Self {
            key: (bits & 0xFFFF) as u16,
            best_move: unsafe { Move::from_raw(((bits >> MOVE_SHIFT) & 0x3FFFFF) as u32) },
            score: Score((((bits >> SCORE_SHIFT) & 0xFFFF) as u16 as i16) as i32),
            depth: ((bits >> DEPTH_SHIFT) & 0xFF) as u8 as i32,
            bound: unsafe {
                core::mem::transmute::<u8, Bound>(((bits >> BOUND_SHIFT) & 0x3) as u8)
            },
        }
    }

    #[inline]
    pub fn pack(self) -> u64 {
        let mut bits = 0u64;
        bits |= self.key as u64;
        bits |= (self.best_move.map_or(0, |mv| mv.raw().get()) as u64) << MOVE_SHIFT;
        bits |= (self.score.0 as u16 as u64) << SCORE_SHIFT;
        bits |= (self.depth as u8 as u64) << DEPTH_SHIFT;
        bits |= (self.bound as u64) << BOUND_SHIFT;
        bits
    }
}

struct AtomicTTEntry {
    packed: AtomicU64,
}

impl Default for AtomicTTEntry {
    fn default() -> AtomicTTEntry {
        AtomicTTEntry {
            packed: AtomicU64::new(0),
        }
    }
}

pub struct TranspositionTable {
    table: Vec<AtomicTTEntry>,
    size: usize,
}

impl Default for TranspositionTable {
    fn default() -> TranspositionTable {
        TranspositionTable::new(TranspositionTable::DEFAULT_SIZE_MB)
    }
}

impl TranspositionTable {
    #[inline]
    pub fn new(size_mb: usize) -> TranspositionTable {
        let size = size_mb * 1024 * 1024 / size_of::<AtomicTTEntry>();
        let table = (0..size).map(|_| AtomicTTEntry::default()).collect();
        TranspositionTable { table, size }
    }

    #[inline]
    pub fn clear(&self) {
        self.table.iter().for_each(|entry| {
            entry.packed.store(0, Ordering::Relaxed);
        });
    }

    #[inline]
    pub fn probe(&self, hash: u64, ply: usize) -> Option<TTEntry> {
        let idx = self.idx(hash);
        let partial_key = Self::partial_key(hash);
        let mut entry = TTEntry::unpack(self.table[idx].packed.load(Ordering::Relaxed));

        if entry.key == partial_key {
            entry.score = score_from_tt(entry.score, ply);
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
        ply: usize,
        bound: Bound,
    ) {
        let idx = self.idx(hash);
        let partial_key = Self::partial_key(hash);
        let entry = &self.table[idx];

        let old_entry = TTEntry::unpack(entry.packed.load(Ordering::Relaxed));
        let new_entry = TTEntry {
            key: partial_key,
            best_move: best_move.or(old_entry.best_move.filter(|_| old_entry.key == partial_key)),
            score: score_to_tt(score, ply),
            depth,
            bound,
        };

        entry.packed.store(new_entry.pack(), Ordering::Relaxed);
    }

    #[inline]
    fn partial_key(hash: u64) -> u16 {
        (hash & 0xFFFF) as u16
    }

    #[inline]
    fn idx(&self, hash: u64) -> usize {
        let key = hash as u128;
        let len = self.size as u128;
        ((key * len) >> 64) as usize
    }

    pub const DEFAULT_SIZE_MB: usize = 64;
    pub const MAX_SIZE_MB: usize = 16 * 1024 * 1024; // duck it we ball
}
