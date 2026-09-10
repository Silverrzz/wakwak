use std::cmp::{max_by_key};
use rand::{RngExt, rngs::ThreadRng};
use crate::{common::Piece, board::Board, common::Move};

#[derive(Debug, Clone, Copy)]
struct SearchOutcome{
    score: i32,
    mv: Option<Move>
}

fn eval(board: &Board, rng: &mut ThreadRng) -> i32 {
    rng.random::<i32>()
}

fn start_negamax(board: Board, depth: u8) -> SearchOutcome {
    let mut rng = rand::rng();
    negamax(board, depth, &mut rng)
}

fn negamax(board: Board, depth: u8, rng: &mut ThreadRng) -> SearchOutcome {
    let friendly_king = board.pieces(Piece::King) & board.colors(board.stm);
    if friendly_king.is_empty() {
        return SearchOutcome { score: i32::MIN, mv: None };
    }

    if depth == 0 {
        return SearchOutcome { score: eval(&board, rng), mv: None };
    }

    let moves = board.gen_moves();
    let mut best: SearchOutcome = SearchOutcome{mv: None, score: i32::MIN};
    for mv in moves.iter() {
        let mut new_board = board.clone();
        new_board.make_move(*mv);

        let child_outcome = negamax(new_board, depth - 1, rng);
        let score = child_outcome.score.saturating_neg();

        if score > best.score {
            best = SearchOutcome { score, mv: Some(*mv)};
        }
    }
    best
}

#[test]
fn depth_1() {
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    println!("{:?}", start_negamax(board, 1));
}

#[test]
fn depth_2() {
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    println!("{:?}", start_negamax(board, 2));
}