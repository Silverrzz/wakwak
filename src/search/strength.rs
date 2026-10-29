use crate::board::SplitMix64;
use crate::common::Move;
use crate::position::Position;
use crate::score::Score;
use crate::search::RootMove;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const TAIL_MASS: f64 = 0.01;

pub struct Strength {
    level: i8,
    candidates: Vec<(Move, f64)>,
    multipv: usize,
    estimates: HashMap<u32, Score>,
}

impl Strength {
    pub fn new(level: i8) -> Self {
        Self {
            level,
            candidates: Vec::new(),
            multipv: 0,
            estimates: HashMap::new(),
        }
    }

    pub fn update(&mut self, root_moves: &[RootMove], searched: usize, pos: &mut Position) {
        self.candidates.clear();
        // Secret WorstWak Setting, introduced so Dan has a chance of winning
        if self.level == -21 {
            self.candidates
                .extend(root_moves.last().map(|root| (root.pv[0], 1.0)));
            return;
        }
        for (index, root) in root_moves.iter().enumerate() {
            let mv = root.pv[0];
            let score = if index < searched {
                root.score
            } else {
                *self.estimates.entry(mv.raw().get()).or_insert_with(|| {
                    pos.make_move(mv);
                    let score = if pos.board().try_king(pos.board().stm()).is_none() {
                        Score::mate(1)
                    } else {
                        -pos.eval()
                    };
                    pos.unmake_move();
                    score
                })
            };
            self.candidates
                .push((mv, f64::from(score.normalized(pos.board()).0)));
        }
        let boundary = self.candidates[searched - 1].1;
        let remaining = &mut self.candidates[searched..];
        remaining.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        if let Some(&(_, best)) = remaining.first() {
            for (_, score) in remaining {
                *score += boundary - best;
            }
        }
        let temperature = self.temperature();
        let top = self.candidates[0].1;
        let mut total = 0.0;
        for (_, score) in &mut self.candidates {
            total += ((*score - top) / temperature).exp();
            *score = total;
        }
        for (_, cumulative) in &mut self.candidates {
            *cumulative /= total;
        }
        self.multipv = (self
            .candidates
            .partition_point(|&(_, p)| p < 1.0 - TAIL_MASS)
            + 1)
        .min(searched + 1);
    }

    pub fn multipv(&self, requested: usize, available: usize) -> usize {
        if self.level == -21 {
            available
        } else {
            self.multipv.max(requested.max(2)).min(available)
        }
    }

    pub fn pick(&self) -> Option<Move> {
        let mut rng = SplitMix64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        let target = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
        let index = self.candidates.partition_point(|&(_, p)| p <= target);
        self.candidates.get(index).map(|&(mv, _)| mv)
    }

    fn temperature(&self) -> f64 {
        if self.level == -20 {
            return f64::INFINITY;
        }
        let target = (20.0 - f64::from(self.level)) / 40.0 * (self.candidates.len() as f64).ln();
        let top = self.candidates[0].1;
        let lowest = self.candidates.last().unwrap().1;
        let (mut low, mut high) = (0.0, (top - lowest).max(1.0) * 40.0);
        for _ in 0..32 {
            let temperature = (low + high) / 2.0;
            let (mut total, mut loss) = (0.0, 0.0);
            for &(_, score) in &self.candidates {
                let gap = (top - score) / temperature;
                let weight = (-gap).exp();
                total += weight;
                loss += weight * gap;
            }
            if total.ln() + loss / total < target {
                low = temperature;
            } else {
                high = temperature;
            }
        }
        (low + high) / 2.0
    }
}
