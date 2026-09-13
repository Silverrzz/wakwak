use crate::engine::EngineOptions;
use crate::score::Score;
use crate::search::{PrincipalVariation, SharedData, ThreadData};

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
        depth: u8,
        score: Score,
        pv: &PrincipalVariation,
    ) {
        let nodes = thread.nodes.global();
        let time = shared.time_man.elapsed();
        let nps = ((nodes as f64) / (time.as_micros().max(1) as f64) * 1e6) as u64;

        println!(
            "info depth {depth} seldepth {} score {score} time {} nodes {nodes} nps {nps} pv {}",
            thread.sel_depth,
            time.as_millis(),
            pv.display(options)
        );
    }
}
