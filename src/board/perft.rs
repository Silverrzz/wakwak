use crate::board::Board;
use crate::util::Abort;

impl Board {
    #[inline]
    pub fn perft<const BULK: bool>(&self, depth: u8) -> u64 {
        if depth == 0 || self.terminal_state().is_some() {
            return 1;
        }

        if BULK && depth == 1 {
            let mut len = 0;
            self.gen_moves(|moves| {
                len += moves.len() as u64;
                Abort::No
            });
            return len;
        }

        let mut nodes = 0u64;
        self.gen_moves(|moves| {
            for mv in moves {
                let mut child = *self;
                child.make_move(mv);

                nodes += child.perft::<BULK>(depth - 1);
            }

            Abort::No
        });

        nodes
    }
}
