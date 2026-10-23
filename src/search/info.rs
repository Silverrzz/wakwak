use crate::board::Board;
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
    pub fn depth(
        self,
        thread: &ThreadData,
        shared: &SharedData,
        options: EngineOptions,
        board: &Board,
    ) {
        let nodes = thread.nodes.global();
        let time = shared.time_man.elapsed();
        let nps = ((nodes as f64) / (time.as_micros().max(1) as f64) * 1e6) as u64;
        let hashfull = shared.tt.hashfull();

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

            let multipv = if options.multipv > 1 {
                format!(" multipv {}", pv_idx + 1)
            } else {
                "".into()
            };

            let bound = match bound {
                Bound::Lower => " lowerbound",
                Bound::Upper => " upperbound",
                _ => "",
            };

            let wdl = if options.show_wdl {
                let [wins, draws, losses] = score.wdl(board);
                format!(" wdl {wins} {draws} {losses}")
            } else {
                String::new()
            };

            let output_score = if options.normalise_score {
                score.normalized(board)
            } else {
                score
            };

            println!(
                "info{multipv} depth {} seldepth {} score {}{bound}{wdl} time {} nodes {nodes} nps {nps} hashfull {hashfull} pv {}",
                root_move.searched_depth,
                root_move.sel_depth,
                output_score,
                time.as_millis(),
                root_move.pv.display(options)
            );
        }
    }
}
