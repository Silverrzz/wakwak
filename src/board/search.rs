use std::cmp::{max, Ordering};
use rand::{RngExt, rngs::ThreadRng};
use crate::{board::Board, common::Move};

#[derive(Debug, Clone)]
struct SearchOutcome{
    score: i32,
    mv: Option<Move>
}

#[inline]
fn eval(board: &Board, rng: &mut ThreadRng) -> i32 {
    rng.random::<i32>()
}

#[inline]
fn oubbbsmax(outcome: SearchOutcome, board: Board, depth: u8, rng: &mut ThreadRng) -> SearchOutcome {
    if depth == 0 {
        return outcome;
    }
    let moves = board.gen_moves();
    if moves.is_empty() {
        return Some(SearchOutcome { score: i32::MAX, mv: None });
    }
    let best_move: Option<Move> = None;
    for mv in moves.iter() {
        let mut new_board = board.clone();
        let outcome = SearchOutcome{mv: Some(*mv), score: eval(&board, rng)};
        new_board.make_move(*mv);
        let new_outcome = oubbbsmax(outcome, new_board, depth, rng);
        best_move = Some(max(v1, v2))
    }
    todo!()
}

#[test]
fn depth_1() {
    let mut rng = rand::rng();
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    let eval = wakamax(board, 1, &mut rng);
    assert_eq!(Eval(0), eval);
}