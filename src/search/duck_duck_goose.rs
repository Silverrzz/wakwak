use crate::board::Board;
use crate::common::{Piece, Square, bishop_rays, king_attacks, rook_rays};
use crate::score::Score;

const SIZE: usize = 4096;

pub struct DuckDuckGooseTable {
    entries: Box<[Entry]>,
}

#[derive(Clone, Copy, Default)]
struct Entry {
    hash: u64,
    depth: i32,
    ply: u16,
    ducks: [u8; 3],
    scores: [i16; 3],
    count: u8,
}

impl Default for DuckDuckGooseTable {
    fn default() -> Self {
        Self {
            entries: vec![Entry::default(); SIZE].into_boxed_slice(),
        }
    }
}

impl DuckDuckGooseTable {
    pub fn clear(&mut self) {
        self.entries.fill(Entry::default());
    }

    pub fn probe(&self, board: &Board, depth: i32, ply: usize, alpha: Score) -> Option<Score> {
        let duck = board.duck()?;
        if board.hmc() != 0 && alpha < Score::ZERO {
            return None;
        }
        let hash = board.duckless_hash();
        let entry = &self.entries[(hash as usize ^ depth as usize ^ (ply << 8)) & (SIZE - 1)];
        if entry.count != 3 || (entry.hash, entry.depth, entry.ply as usize) != (hash, depth, ply) {
            return None;
        }
        let mut score = Score(entry.scores[0].max(entry.scores[1]).max(entry.scores[2]) as i32);
        if board.hmc() != 0 {
            score = score.max(Score::ZERO);
        }
        (score <= alpha
            && ((king_attacks(board.king(board.stm())) & !board.colors(board.stm()) & !duck)
                .is_nonempty()
                || board.any_moves(|moves| moves.is_nonempty())))
        .then_some(score)
    }

    pub fn insert(&mut self, board: &Board, score: Score, depth: i32, ply: usize) {
        let Some(duck) = board.duck() else { return };
        if board.hmc() != 0 || score <= -Score::MAX_MATE || score >= Score::MAX_MATE {
            return;
        }
        let Ok(stored_ply) = u16::try_from(ply) else {
            return;
        };
        let hash = board.duckless_hash();
        let entry = &mut self.entries[(hash as usize ^ depth as usize ^ (ply << 8)) & (SIZE - 1)];
        if (entry.hash, entry.depth, entry.ply as usize) != (hash, depth, ply) {
            *entry = Entry {
                hash,
                depth,
                ply: stored_ply,
                ..Entry::default()
            };
        }

        let count = entry.count as usize;
        let mut index = count;
        let rays = rook_rays(duck) | bishop_rays(duck);
        for i in 0..count {
            if rays.0 & (1 << entry.ducks[i]) != 0 {
                let other = Square::index(entry.ducks[i] as usize);
                if duck == other {
                    entry.scores[i] = entry.scores[i].min(score.0 as i16);
                    return;
                }
                let stm = board.stm();
                let (line, pieces) = if duck.rank() == other.rank() {
                    (
                        duck.rank().bitboard(),
                        board.colored_orth_sliders(stm) | board.colored_pieces(stm, Piece::King),
                    )
                } else if duck.file() == other.file() {
                    (
                        duck.file().bitboard(),
                        board.colored_orth_sliders(stm) | board.colored_pieces(stm, Piece::Pawn),
                    )
                } else {
                    (
                        bishop_rays(duck) & bishop_rays(other),
                        board.colored_diag_sliders(stm),
                    )
                };
                if (line & pieces).is_empty() {
                    continue;
                }
                if index != count {
                    return;
                }
                index = i;
            }
        }
        if index == 3 {
            index = (0..3).max_by_key(|&i| entry.scores[i]).unwrap();
        }
        if index == count || score.0 < entry.scores[index] as i32 {
            entry.ducks[index] = duck as u8;
            entry.scores[index] = score.0 as i16;
            entry.count = entry.count.max(index as u8 + 1);
        }
    }
}
