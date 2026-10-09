use crate::engine::EngineOptions;
use crate::score::Score;
use crate::search::{Bound, SharedData, ThreadData};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchInfo {
    Full,
    Minimal,
    None,
}

impl SearchInfo {
    #[inline]
    pub fn depth(self, thread: &ThreadData, shared: &SharedData, options: EngineOptions) {
        let nodes = thread.nodes.global();
        let time = shared.time_man.elapsed();
        let nps = ((nodes as f64) / (time.as_micros().max(1) as f64) * 1e6) as u64;

        for (pv_idx, root_move) in thread.root_moves[..thread.multipv].iter().enumerate() {
            let mut score = root_move.display_score;
            let mut bound = root_move.bound;

            if score == -Score::INFINITE {
                score = root_move.previous_score;
                bound = Bound::Exact;
            }

            if score == -Score::INFINITE {
                break;
            }

            let bound = match bound {
                Bound::Lower => " lowerbound",
                Bound::Upper => " upperbound",
                _ => "",
            };

            if options.multipv > 1 {
                print!("info multipv {}", pv_idx + 1);
            } else {
                print!("info");
            }

            println!(
                " depth {} seldepth {} score {}{bound} time {} nodes {nodes} nps {nps} pv {}",
                root_move.searched_depth,
                root_move.sel_depth,
                score,
                time.as_millis(),
                root_move.pv.display(options)
            );
        }
    }
}
