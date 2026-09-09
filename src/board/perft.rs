use crate::board::Board;
use crate::common::{Color, Piece};

impl Board {
    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        if [Color::White, Color::Black]
            .into_iter()
            .any(|color| self.colored_pieces(color, Piece::King).is_empty())
        {
            return 0;
        }

        let moves = self.gen_moves();
        if depth == 1 {
            return moves.len() as u64;
        }

        let mut nodes = 0u64;
        for &mv in moves.iter() {
            let mut child = *self;
            child.make_move(mv);
            nodes = nodes
                .checked_add(child.perft(depth - 1))
                .expect("Perft node count overflow");
        }
        nodes
    }
}
