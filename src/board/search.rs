use std::cmp::{max_by_key};
use rand::{RngExt, rngs::ThreadRng};
use crate::{board::Board, common::Move};

#[derive(Debug, Clone, Copy)]
struct SearchOutcome{
    score: i32,
    mv: Option<Move>
}

#[inline]
fn eval(board: &Board, rng: &mut ThreadRng) -> i32 {
    rng.random::<i32>()
}

fn start_negamax(board: Board, depth: u8) -> SearchOutcome {
    let mut rng = rand::rng();
    negamax(SearchOutcome { score: i32::MIN, mv: None}, board, depth, &mut rng)
}

#[inline]
fn negamax(outcome: SearchOutcome, board: Board, depth: u8, rng: &mut ThreadRng) -> SearchOutcome {
    if depth == 0 {
        return outcome;
    }
    let moves = board.gen_moves();
    if moves.is_empty() {
        return SearchOutcome { score: i32::MAX, mv: None };
    }
    let mut best_outcome: SearchOutcome = SearchOutcome{mv: None, score: i32::MIN};
    for mv in moves.iter() {
        let mut new_board = board.clone();
        let outcome = SearchOutcome{mv: Some(*mv), score: eval(&board, rng)};
        new_board.make_move(*mv);
        let new_outcome = negamax(outcome, new_board, depth - 1, rng);
        best_outcome = max_by_key(best_outcome, new_outcome, |o: &SearchOutcome| o.score);
    }
    best_outcome
}

#[test]
fn depth_1() {
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    println!("{:?}", start_negamax(board, 1));
    assert!(false);
}