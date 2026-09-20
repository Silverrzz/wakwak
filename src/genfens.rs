use crate::board::{Board, SplitMix64};
use crate::engine::{Engine, EngineOptions};
use crate::position::Position;
use crate::search::SearchInfo;
use crate::uci::SearchLimit;
use crate::util::Abort;
use std::sync::atomic::Ordering;

impl Engine {
    pub fn gen_fens(&mut self, count: usize, seed: u64, dfrc: bool, plies: u16) {
        let mut rng = SplitMix64::new(seed);
        let options = EngineOptions {
            soft_target: true,
            frc: dfrc,
            ..self.options
        };
        self.searcher.newgame();
        for _ in 0..count {
            loop {
                let Some(position) = random_opening(&mut rng, dfrc, plies) else {
                    continue;
                };
                self.searcher.search(
                    position.clone(),
                    options,
                    vec![SearchLimit::Nodes(1000)],
                    SearchInfo::None,
                );
                self.searcher.wait();
                if self
                    .searcher
                    .shared
                    .best_score
                    .load(Ordering::Relaxed)
                    .abs()
                    >= 1000
                {
                    continue;
                }
                println!("info string genfens {}", position.board().to_fen(dfrc));
                break;
            }
        }
    }
}

fn random_opening(rng: &mut SplitMix64, dfrc: bool, plies: u16) -> Option<Position> {
    let board = if dfrc {
        Board::dfrc_startpos((rng.next() % 960) as u16, (rng.next() % 960) as u16)
    } else {
        Board::startpos()
    };
    let mut position = Position::new(board);
    let mut moves = Vec::new();
    let plies = usize::from(plies) + (rng.next() % 2) as usize;
    for _ in 0..plies {
        if position.board().terminal_state().is_some() || position.repetition() {
            return None;
        }
        moves.clear();
        position.board().gen_all_moves(|group| {
            moves.extend(group);
            Abort::No
        });
        position.make_move(moves[(rng.next() % moves.len() as u64) as usize]);
    }
    if position.board().terminal_state().is_some() || position.repetition() {
        return None;
    }
    Some(position)
}
