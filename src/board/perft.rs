use crate::board::Board;
use crate::common::Piece;

impl Board {
    #[inline]
    pub fn perft<const BULK: bool>(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        if self.pieces(Piece::King).popcnt() != 2 {
            return 0;
        }

        let moves = self.gen_moves();
        if BULK && depth == 1 {
            return moves.len() as u64;
        }

        let mut nodes = 0u64;
        for &mv in moves.iter() {
            let mut child = *self;
            child.make_move(mv);
            nodes = nodes
                .checked_add(child.perft::<BULK>(depth - 1))
                .expect("Board::perft(): Node count overflow");
        }
        nodes
    }
}
