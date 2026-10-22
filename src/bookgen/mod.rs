mod config;
mod filter;
mod generate;
mod output;
mod source;

use crate::board::{Board, SplitMix64};
use crate::common::{Move, Piece};
use crate::engine::EngineOptions;
use crate::search::{Bound, RootMove, Searcher};
use config::{Config, Grouping, Policy};
use source::{Candidate, Input};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

struct Analysis {
    board: Board,
    roots: Vec<RootMove>,
}

impl Analysis {
    fn cp(&self, index: usize) -> Option<i32> {
        let score = self.roots.get(index)?.score;
        (!score.is_mate()).then(|| score.normalized(&self.board).0)
    }

    fn gap(&self) -> Option<i32> {
        Some(self.cp(0)? - self.cp(1)?)
    }
}

struct Record {
    source: String,
    initial_fen: String,
    moves: Vec<Move>,
    board: Board,
}

fn analyze(
    searcher: &mut Searcher,
    candidate: &Candidate,
    nodes: u64,
    multipv: usize,
    stopped: &AtomicBool,
) -> Option<Analysis> {
    if candidate.ended() || stopped.load(Ordering::Relaxed) {
        return None;
    }
    let options = EngineOptions {
        multipv,
        frc: true,
        ..EngineOptions::default()
    };
    let roots = searcher.analyze(candidate.position.clone(), options, nodes);
    if stopped.load(Ordering::Relaxed)
        || roots.is_empty()
        || roots
            .iter()
            .any(|root| root.bound != Bound::Exact || root.pv.is_empty())
    {
        return None;
    }
    Some(Analysis {
        board: *candidate.position.board(),
        roots,
    })
}

fn random(rng: &mut SplitMix64) -> f64 {
    (rng.next() >> 11) as f64 / (1u64 << 53) as f64
}

fn choose<T: Copy>(rng: &mut SplitMix64, choices: &[(T, f64)]) -> Option<T> {
    let mut target = random(rng) * choices.iter().map(|(_, weight)| weight).sum::<f64>();
    for &(value, weight) in choices {
        target -= weight;
        if target <= 0.0 {
            return Some(value);
        }
    }
    choices.last().map(|&(value, _)| value)
}

fn choose_random(board: &Board, grouping: Grouping, rng: &mut SplitMix64) -> Option<Move> {
    let mut groups = Vec::new();
    board.gen_all_moves(|moves| {
        if moves.is_nonempty() && board.piece_on(moves.dest) != Some(Piece::King) {
            groups.push(moves);
        }
        crate::util::Abort::No
    });
    let weights: Vec<_> = groups
        .iter()
        .enumerate()
        .map(|(index, moves)| {
            (
                index,
                if grouping == Grouping::Piece {
                    1.0
                } else {
                    moves.len() as f64
                },
            )
        })
        .collect();
    let moves = &groups[choose(rng, &weights)?];
    moves.iter().nth((rng.next() % moves.len() as u64) as usize)
}

fn walk(
    config: &config::Walk,
    mut candidate: Candidate,
    searcher: &mut Searcher,
    rng: &mut SplitMix64,
    stopped: &AtomicBool,
) -> Option<Candidate> {
    for _ in 0..config.plies.sample(rng) {
        if stopped.load(Ordering::Relaxed) || candidate.ended() {
            return None;
        }
        let use_random = config.policy == Policy::Random
            || (config.policy == Policy::Mixed && random(rng) < config.random_probability);
        let mv = if use_random {
            choose_random(candidate.position.board(), config.duck_sampling, rng)?
        } else {
            let analysis = analyze(searcher, &candidate, config.nodes, config.multipv, stopped)?;
            let best = analysis.cp(0)?;
            if config.policy == Policy::Best {
                analysis.roots[0].pv[0]
            } else {
                let choices: Vec<_> = analysis
                    .roots
                    .iter()
                    .enumerate()
                    .filter_map(|(index, root)| {
                        let cp = analysis.cp(index)?;
                        (best - cp <= config.max_loss_cp).then(|| {
                            (
                                root.pv[0],
                                ((cp - best) as f64 / config.temperature_cp).exp(),
                            )
                        })
                    })
                    .collect();
                choose(rng, &choices)?
            }
        };
        if candidate.position.board().piece_on(mv.dest()) == Some(Piece::King) {
            return None;
        }
        candidate.play(mv);
    }
    if candidate.ended() {
        return None;
    }
    Some(candidate)
}

fn verify(
    config: &Config,
    candidate: &Candidate,
    searcher: &mut Searcher,
    stopped: &AtomicBool,
) -> Option<()> {
    let verify = &config.verify;
    let board = candidate.position.board();
    if !config.filter.matches(board, None) {
        return None;
    }
    let mut final_stage: Option<Analysis> = None;
    for &nodes in &verify.nodes {
        let analysis = analyze(searcher, candidate, nodes, verify.multipv, stopped)?;
        let best = &analysis.roots[0];
        if best.searched_depth < verify.min_depth || (best.score.is_mate() && !verify.allow_mate) {
            return None;
        }
        if verify
            .early_cp_margin
            .is_some_and(|margin| !config.filter.matches_early(&analysis, margin))
        {
            return None;
        }
        if let (Some(limit), Some(previous)) = (verify.max_cp_shift, &final_stage)
            && !analysis
                .cp(0)
                .zip(previous.cp(0))
                .is_some_and(|(cp, old)| (cp - old).abs() <= limit)
        {
            return None;
        }
        final_stage = Some(analysis);
    }
    let final_stage = final_stage.unwrap();
    if !config.filter.matches(board, Some(&final_stage))
        || verify
            .min_gap_cp
            .is_some_and(|minimum| final_stage.gap().is_none_or(|gap| gap < minimum))
    {
        return None;
    }
    if verify.rollout_plies > 0 {
        let baseline = final_stage.cp(0)? * i32::from(board.stm().signum());
        let mut continuation = candidate.clone();
        for ply in 0..=verify.rollout_plies {
            let analysis = analyze(searcher, &continuation, verify.rollout_nodes, 1, stopped)?;
            if analysis.roots[0].searched_depth < verify.min_depth {
                return None;
            }
            let cp = analysis.cp(0)? * i32::from(analysis.board.stm().signum());
            if config
                .verify
                .max_rollout_cp_shift
                .is_some_and(|limit| (cp - baseline).abs() > limit)
            {
                return None;
            }
            if ply < verify.rollout_plies {
                let mv = analysis.roots[0].pv[0];
                if analysis.board.piece_on(mv.dest()) == Some(Piece::King) {
                    return None;
                }
                continuation.play(mv);
            }
        }
    }
    Some(())
}

fn identity(fen: &str) -> String {
    fen.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
}

pub fn run() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Usage: wakwak bookgen <recipe.toml> [--validate]\n\nRelative paths resolve beside the recipe. workers = 0 uses available CPUs.\nthreads and hash_mb are per worker. CP and WDL filters use White's perspective."
        );
        return Ok(());
    }
    let work = || -> Result<()> {
        ensure(
            args[1..].iter().all(|arg| arg == "--validate"),
            "unknown bookgen option",
        )?;
        let config = Config::load(Path::new(&args[0]))?;
        if args.iter().any(|arg| arg == "--validate") {
            println!("Recipe loaded. Output: {}", config.output.path.display());
            return Ok(());
        }
        let inputs = config
            .sources
            .iter()
            .cloned()
            .map(Input::load)
            .collect::<Result<Vec<_>>>()?;
        generate::run(&config, &inputs)
    };
    work().map_err(|error| std::io::Error::other(error.to_string()))
}
