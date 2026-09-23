use crate::common::Move;
use crate::score::Score;
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

#[inline]
fn i16_to_i15(value: i16) -> u16 {
    (value.clamp(-16384, 16383) as u16) & 0x7FFF
}

#[inline]
fn i15_to_i16(value: u16) -> i16 {
    (value << 1) as i16 >> 1
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TTFlag {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl TTFlag {
    #[inline]
    pub fn bounds_match(self, score: Score, lower: Score, upper: Score) -> bool {
        match self {
            TTFlag::None => false,
            TTFlag::Exact => true,
            TTFlag::Lower => score >= upper,
            TTFlag::Upper => score <= lower,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    pub key: u16,
    pub mv: Option<Move>,
    pub score: Score,
    pub eval: Score,
    pub depth: i32,
    pub flag: TTFlag,
    pub pv: bool,
}

impl TTEntry {
    #[inline]
    pub fn pack(self) -> u64 {
        let packed_flags = self.flag as u64 | (self.pv as u64) << 2;

        let mut bits = 0;
        bits |= self.mv.map_or(0, |mv| mv.raw().get()) as u64;
        bits |= (self.score.0 as u16 as u64) << 22;
        bits |= (i16_to_i15(self.eval.0 as i16) as u64) << 38;
        bits |= (self.depth as u64) << 53;
        bits |= packed_flags << 61;
        bits
    }

    #[inline]
    pub fn unpack(data: u64, key: u16) -> Self {
        Self {
            key,
            mv: unsafe { Move::from_raw((data & 0x3FFFFF) as u32) },
            score: Score((data >> 22) as i16 as i32),
            eval: Score(i15_to_i16((data >> 38) as u16) as i32),
            depth: (data >> 53) as u8 as i32,
            flag: unsafe { std::mem::transmute::<u8, TTFlag>((data >> 61) as u8 & 0b11) },
            pv: (data >> 63) != 0,
        }
    }
}

const CLUSTER_SIZE: usize = 3;

#[repr(C, align(32))]
pub struct TTCluster {
    data: [AtomicU64; CLUSTER_SIZE],
    keys: [AtomicU16; CLUSTER_SIZE],
}

impl TTCluster {
    #[inline]
    pub fn empty() -> Self {
        Self {
            data: std::array::from_fn(|_| AtomicU64::new(0)),
            keys: std::array::from_fn(|_| AtomicU16::new(0)),
        }
    }

    #[inline]
    pub fn load(&self, index: usize) -> TTEntry {
        let data = self.data[index].load(Ordering::Relaxed);
        let key = self.keys[index].load(Ordering::Relaxed);
        TTEntry::unpack(data, key)
    }

    #[inline]
    pub fn store(&self, index: usize, entry: TTEntry) {
        self.data[index].store(entry.pack(), Ordering::Relaxed);
        self.keys[index].store(entry.key, Ordering::Relaxed);
    }

    #[inline]
    pub fn clear(&self) {
        for i in 0..CLUSTER_SIZE {
            self.data[i].store(0, Ordering::Relaxed);
            self.keys[i].store(0, Ordering::Relaxed);
        }
    }
}

pub struct TTable {
    clusters: Box<[TTCluster]>,
    size: usize,
}

impl TTable {
    #[inline]
    pub fn new(size_mb: usize) -> TTable {
        let size = size_mb * 1024 * 1024 / size_of::<TTCluster>();
        let clusters = (0..size).map(|_| TTCluster::empty()).collect();
        TTable { clusters, size }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn insert<const STATIC_EVAL: bool>(
        &self,
        hash: u64,
        best_move: Option<Move>,
        score: Score,
        eval: Score,
        depth: i32,
        flag: TTFlag,
        pv: bool,
    ) {
        let partial_key = hash as u16;
        let cluster = &self.clusters[self.idx(hash)];

        let mut index = 0;
        let mut min_value = i32::MAX;
        for i in 0..CLUSTER_SIZE {
            let entry = cluster.load(i);
            if entry.key == partial_key || entry.flag == TTFlag::None {
                index = i;
                break;
            }

            let entry_value = entry.depth;
            if entry_value < min_value {
                index = i;
                min_value = entry_value;
            }
        }

        let old_entry = cluster.load(index);

        // Early static eval inserts should only replace empty entries
        if !STATIC_EVAL || old_entry.flag == TTFlag::None {
            cluster.store(
                index,
                TTEntry {
                    key: partial_key,
                    mv: best_move.or(old_entry.mv.filter(|_| old_entry.key == partial_key)),
                    score,
                    eval,
                    depth,
                    flag,
                    pv,
                },
            );
        }
    }

    #[inline]
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let partial_key = hash as u16;
        let cluster = &self.clusters[self.idx(hash)];

        for i in 0..CLUSTER_SIZE {
            let entry = cluster.load(i);
            if entry.key == partial_key {
                return Some(entry);
            }
        }

        None
    }

    #[inline]
    pub fn clear(&self) {
        self.clusters.iter().for_each(|c| c.clear());
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

impl Default for TTable {
    #[inline]
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE_MB)
    }
}
