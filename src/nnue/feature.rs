use crate::common::{Color, Piece, Square};
use crate::nnue::{HM, should_mirror};

#[derive(Debug, Copy, Clone)]
pub struct PieceFeature {
    pub piece: Piece,
    pub color: Color,
    pub sq: Square,
}

impl PieceFeature {
    #[inline]
    pub fn new(piece: Piece, color: Color, sq: Square) -> Self {
        Self { piece, color, sq }
    }

    #[inline]
    pub fn to_index(self, king: Square, perspective: Color) -> usize {
        let mut sq = self.sq.relative_to(perspective);
        let color = self.color ^ perspective;

        if HM && should_mirror(king) {
            sq = sq.flip_file();
        }

        color as usize * Square::COUNT * Piece::COUNT
            + self.piece as usize * Square::COUNT
            + sq as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DuckFeature(pub Square);

impl DuckFeature {
    #[inline]
    pub fn to_index(self, king: Square, perspective: Color) -> usize {
        let mut sq = self.0.relative_to(perspective);

        if HM && should_mirror(king) {
            sq = sq.flip_file();
        }

        768 + sq as usize
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FeatureUpdates {
    pub add: Option<PieceFeature>,
    pub add2: Option<PieceFeature>,
    pub sub: Option<PieceFeature>,
    pub sub2: Option<PieceFeature>,
    //pub duck_add: Option<DuckFeature>,
    //pub duck_sub: Option<DuckFeature>,
}
