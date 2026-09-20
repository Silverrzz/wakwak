use crate::board::Board;
use crate::common::{Color, Move, MoveFlag, Piece, Rank, Square};
use crate::nnue::{Accumulator, FeatureUpdates, HM, PieceFeature, feed_forward, should_mirror};
use crate::score::Score;
use crate::search::MAX_PLY;
use enum_map::enum_map;

#[derive(Clone)]
pub struct Nnue {
    pub stack: Box<[Accumulator; MAX_PLY + 1]>,
    pub cursor: usize,
}

impl Nnue {
    #[inline]
    pub fn new(board: &Board) -> Self {
        let mut nnue = Self {
            stack: vec![Accumulator::default(); MAX_PLY + 1]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            cursor: 0,
        };

        nnue.full_reset(board);
        nnue
    }

    #[inline]
    pub fn eval(&self, stm: Color) -> Score {
        let (stm, ntm) = (
            &self.stack[self.cursor].values[stm],
            &self.stack[self.cursor].values[!stm],
        );

        Score(feed_forward(stm, ntm)).clamp_mate()
    }

    #[inline]
    pub fn full_reset(&mut self, board: &Board) {
        self.cursor = 0;
        self.reset(board, Color::White);
        self.reset(board, Color::Black);
    }

    #[inline]
    pub fn reset(&mut self, board: &Board, perspective: Color) {
        self.stack[self.cursor].reset(board, perspective);
    }

    #[inline]
    pub fn update(&mut self, board: &Board) {
        for &perspective in Color::ALL {
            if self.stack[self.cursor].needs_refresh[perspective] {
                self.reset(board, perspective);
            } else if self.stack[self.cursor].dirty[perspective] {
                self.update_dirty(board, perspective);
            }
        }
    }

    #[inline]
    fn update_dirty(&mut self, board: &Board, perspective: Color) {
        let mut clean_index = None;
        for i in (0..self.cursor).rev() {
            if self.stack[i].needs_refresh[perspective] {
                break;
            }

            if !self.stack[i].dirty[perspective] {
                clean_index = Some(i);
                break;
            }
        }

        let king = board.king(perspective);

        let Some(clean_index) = clean_index else {
            self.reset(board, perspective);
            return;
        };

        for i in clean_index..self.cursor {
            let [clean, dirty] = self.stack.get_disjoint_mut([i, i + 1]).unwrap();
            dirty.update(clean, king, perspective);
        }
    }

    #[inline]
    pub fn make_move(&mut self, board: &Board, mv: Move) {
        let mut updates = FeatureUpdates::default();
        let (src, mut dest, flag) = (mv.src(), mv.dest(), mv.flag());
        let piece = board.piece_on(mv.src()).unwrap();
        let stm = board.stm();

        //updates.duck_add = Some(DuckFeature(mv.duck()));
        //updates.duck_sub = board.duck().map(DuckFeature);

        if let Some(dir) = flag.castling_dir() {
            let rank = Rank::First.relative_to(board.stm());
            let king_dest = Square::new(dir.king_dest(), rank);
            let rook_dest = Square::new(dir.rook_dest(), rank);

            updates.add = Some(PieceFeature::new(Piece::King, stm, king_dest));
            updates.add2 = Some(PieceFeature::new(Piece::Rook, stm, rook_dest));
            updates.sub = Some(PieceFeature::new(Piece::King, stm, src));
            updates.sub2 = Some(PieceFeature::new(Piece::Rook, stm, dest));

            dest = king_dest;
        } else if let Some(promo) = flag.promotion() {
            updates.add = Some(PieceFeature::new(promo, stm, dest));
            updates.sub = Some(PieceFeature::new(piece, stm, src));
        } else {
            updates.add = Some(PieceFeature::new(piece, stm, dest));
            updates.sub = Some(PieceFeature::new(piece, stm, src));
        }

        if flag == MoveFlag::EnPassant {
            let victim = Square::new(dest.file(), src.rank());
            updates.sub2 = Some(PieceFeature::new(Piece::Pawn, !stm, victim));
        } else if flag.is_capture()
            && let Some(piece) = board.piece_on(dest)
        {
            updates.sub2 = Some(PieceFeature::new(piece, !stm, dest));
        }

        self.stack[self.cursor].updates = updates;
        self.cursor += 1;
        self.stack[self.cursor].dirty = enum_map! { _ => true };
        self.stack[self.cursor].needs_refresh = self.stack[self.cursor - 1].needs_refresh;

        if piece == Piece::King && (HM && should_mirror(src) != should_mirror(dest)) {
            self.stack[self.cursor].needs_refresh[stm] = true;
        }
    }

    #[inline]
    pub fn make_null_move(&mut self, _board: &Board, _new_duck: Option<Square>) {
        /*let updates = FeatureUpdates {
            duck_add: new_duck.map(DuckFeature),
            duck_sub: board.duck().map(DuckFeature),
            ..Default::default()
        };

        self.stack[self.cursor].updates = updates;
        self.cursor += 1;
        self.stack[self.cursor].dirty = enum_map! { _ => true };
        self.stack[self.cursor].needs_refresh = self.stack[self.cursor - 1].needs_refresh;*/
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.cursor -= 1;
    }
}
