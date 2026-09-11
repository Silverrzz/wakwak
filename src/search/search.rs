use crate::board::TerminalState;
use crate::position::Position;
use crate::score::Score;
use crate::search::MAX_PLY;
use crate::{board::Board, common::Move};
use rand::{RngExt, rngs::ThreadRng};
use std::ops::Neg;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct SearchOutcome {
    score: Option<Score>,
    mv: Option<Move>,
}

#[inline]
fn eval(_board: &Board, rng: &mut ThreadRng) -> Score {
    Score(rng.random_range((-Score::MAX_MATE.0 + 1)..=(Score::MAX_MATE.0 - 1)))
}

#[allow(dead_code)]
fn start_negamax(mut pos: Position, depth: u8) -> SearchOutcome {
    let mut rng = rand::rng();
    negamax(&mut pos, depth, 0, &mut rng)
}

fn negamax(pos: &mut Position, depth: u8, ply: u16, rng: &mut ThreadRng) -> SearchOutcome {
    if let Some(terminal_state) = pos.board().terminal_state() {
        return match terminal_state {
            TerminalState::Victory(_) => SearchOutcome {
                score: Some(Score::mated(ply)),
                mv: None,
            },
            TerminalState::Stalemate(_) => SearchOutcome {
                score: Some(Score::mate(ply)),
                mv: None,
            },
            TerminalState::Draw => SearchOutcome {
                score: Some(Score::draw()),
                mv: None,
            },
        };
    }

    if depth == 0 || ply >= MAX_PLY {
        return SearchOutcome {
            score: Some(eval(pos.board(), rng)),
            mv: None,
        };
    }

    let moves = pos.board().gen_moves();
    let mut best_outcome: SearchOutcome = SearchOutcome {
        mv: None,
        score: None,
    };
    for &mv in moves.iter() {
        pos.make_move(mv);
        let child_outcome = negamax(pos, depth - 1, ply + 1, rng);
        pos.unmake_move();

        let score = child_outcome.score.map(Score::neg);
        if score > best_outcome.score {
            best_outcome = SearchOutcome {
                score,
                mv: Some(mv),
            };
        }
    }
    best_outcome
}

#[test]
fn depth_1() {
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    let pos = Position::new(board);

    println!("{:?}", start_negamax(pos, 1));
}

#[test]
fn depth_2() {
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    let pos = Position::new(board);

    println!("{:?}", start_negamax(pos, 2));
}
