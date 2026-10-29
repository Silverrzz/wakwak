use crate::board::{Board, SplitMix64};
use crate::common::Move;
use crate::score::Score;
use crate::search::RootMove;
use std::time::{SystemTime, UNIX_EPOCH};

const BASE_TEMPERATURE: f64 = 100.0;
const TAIL_MASS: f64 = 0.01;

pub struct Strength {
    level: u8,
    candidates: Vec<(Move, Score)>,
}

impl Strength {
    pub fn new(level: u8) -> Self {
        Self {
            level,
            candidates: Vec::new(),
        }
    }

    pub fn update(&mut self, root_moves: &[RootMove], board: &Board) {
        self.candidates.clear();
        self.candidates.extend(
            root_moves
                .iter()
                .map(|root| (root.pv[0], root.score.normalized(board))),
        );
    }

    pub fn multipv(&self, requested: usize, available: usize) -> usize {
        let mut count = self.candidates.len();
        if count > 0 {
            let mut weights = self.weights();
            let total = weights.clone().sum::<f64>();
            let tail = weights.clone().next_back().unwrap_or(0.0) * (available - count) as f64;
            let mut cumulative = 0.0;
            count = if tail > total * TAIL_MASS {
                count + 1
            } else {
                weights
                    .position(|weight| {
                        cumulative += weight;
                        cumulative >= total * (1.0 - TAIL_MASS)
                    })
                    .map_or(count, |index| index + 1)
            };
        }
        count.max(requested.max(2)).min(available)
    }

    pub fn pick(&self) -> Option<Move> {
        let &(best_move, _) = self.candidates.first()?;
        let mut rng = SplitMix64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        let weights = self.weights();
        let mut target =
            (rng.next() >> 11) as f64 / (1u64 << 53) as f64 * weights.clone().sum::<f64>();
        for (weight, &(mv, _)) in weights.zip(&self.candidates) {
            target -= weight;
            if target < 0.0 {
                return Some(mv);
            }
        }
        Some(best_move)
    }

    fn weights(&self) -> impl DoubleEndedIterator<Item = f64> + Clone + '_ {
        let top = self.candidates[0].1;
        let lowest = self
            .candidates
            .iter()
            .filter_map(|(_, score)| (!score.is_mate()).then_some(score.0))
            .min()
            .unwrap_or(top.0);
        let spread = f64::from(top.0 - lowest);
        let temperature = BASE_TEMPERATURE
            * ((20.0 - f64::from(self.level)) / 20.0).powi(2)
            * (1.0 + spread / (BASE_TEMPERATURE + spread));
        self.candidates
            .iter()
            .enumerate()
            .map(move |(index, &(_, score))| {
                if top.is_mate() {
                    f64::from(index == 0)
                } else if score.is_mate() {
                    0.0
                } else {
                    (f64::from(score.0 - top.0) / temperature).exp()
                }
            })
    }
}
