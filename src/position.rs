use crate::board::{Board, ZOBRIST};
use crate::common::{Move, Piece, between};
use crate::nnue::Nnue;
use crate::score::Score;
use crate::search::cuckoo::{Cuckoo, CuckooTag};
use crate::search::{MAX_PLY, cuckoo};

#[derive(Clone)]
pub struct Position {
    current: Board,
    previous_boards: Vec<Board>,
    previous_moves: Vec<Option<(Piece, Move)>>,
    nnue: Nnue,

    cuckoo_tag: CuckooTag,
}

impl Position {
    #[inline]
    pub fn new(board: Board) -> Self {
        Self {
            current: board,
            previous_boards: Vec::with_capacity(MAX_PLY),
            previous_moves: Vec::with_capacity(MAX_PLY),
            nnue: Nnue::new(&board),
            cuckoo_tag: Cuckoo::init(),
        }
    }

    #[inline]
    pub fn reset(&mut self, board: Board) {
        self.current = board;
        self.previous_boards.clear();
        self.previous_moves.clear();
        self.nnue.full_reset(&board);
    }

    #[inline]
    pub fn reset_nnue(&mut self) {
        self.nnue.full_reset(&self.current);
    }

    #[inline]
    pub fn make_move(&mut self, mv: Move) {
        let piece = self.current.piece_on(mv.src()).unwrap();

        self.previous_boards.push(self.current);
        self.previous_moves.push(Some((piece, mv)));
        self.nnue.make_move(&self.current, mv);
        self.current.make_move(mv);
    }

    #[inline]
    pub fn make_null_move(&mut self) {
        self.previous_boards.push(self.current);
        self.previous_moves.push(None);
        self.nnue.make_null_move(&self.current, None);
        self.current.make_null_move(None);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
        self.previous_moves.pop().unwrap();
        self.nnue.unmake_move();
    }

    // TODO: When going from 768 to 832 inputs, remove this and replace the calls with `unmake_move` calls
    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.current = self.previous_boards.pop().unwrap();
        self.previous_moves.pop().unwrap();
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn eval(&mut self) -> Score {
        self.nnue.update(&self.current);
        self.nnue.eval(&self.current)
    }

    #[inline]
    pub fn prev_move(&self, ply: usize) -> Option<(Piece, Move)> {
        self.previous_moves
            .len()
            .checked_sub(ply)
            .and_then(|i| self.previous_moves[i])
    }

    #[inline]
    pub fn repetition(&self, ply: usize) -> bool {
        let mut matches = self
            .previous_boards
            .iter()
            .rev()
            .take(self.current.hmc() as usize)
            .enumerate()
            .filter(|(_, board)| board.hash() == self.current.hash());
        matches.next().is_some_and(|(index, _)| index < ply) || matches.next().is_some()
    }

    #[inline]
    pub fn upcoming_repetition(&self, ply: usize) -> bool {
        let end = self.previous_boards.len().min(self.current.hmc() as usize);

        if end < 3 {
            return false;
        }

        let old_key =
            |i: usize| self.previous_boards[self.previous_boards.len() - i].duckless_hash();
        let old_duck = |i: usize| self.previous_boards[self.previous_boards.len() - i].duck();

        let occ = self.current.occupied();
        let orig_key = self.current.duckless_hash();

        let mut other = orig_key ^ old_key(1) ^ ZOBRIST.stm;

        for i in (3..=end).step_by(2) {
            let curr_key = old_key(i);
            let curr_duck = old_duck(i);

            other ^= curr_key ^ old_key(i - 1) ^ ZOBRIST.stm;
            if other != 0 {
                continue;
            }

            let diff = orig_key ^ curr_key;

            let cuckoo = Cuckoo::get(self.cuckoo_tag);
            let mut slot = cuckoo::h1(diff);
            if diff != cuckoo.keys[slot] {
                slot = cuckoo::h2(diff);
            }
            if diff != cuckoo.keys[slot] {
                continue;
            }

            let (sq1, sq2) = cuckoo.moves[slot];

            if !occ.is_disjoint(between(sq1, sq2))
                || self
                    .current
                    .duck()
                    .is_some_and(|duck| duck == sq1 || duck == sq2)
                || curr_duck == self.current.duck()
            {
                continue;
            }

            if i <= ply {
                return true;
            }

            if (i + 2..=end)
                .step_by(2)
                .any(|j| old_key(j) == curr_key && old_duck(j) == curr_duck)
            {
                return true;
            }
        }

        false
    }
}
