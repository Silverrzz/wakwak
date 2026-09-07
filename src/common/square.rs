use crate::common::{Bitboard, Color, File, Rank};
use crate::def_enum;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

def_enum! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Enum)]
    #[rustfmt::skip]
    pub enum Square : u8 {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8
    }
}

impl Square {
    #[inline]
    pub const fn new(file: File, rank: Rank) -> Self {
        Self::index(((rank as usize) << 3) | file as usize)
    }

    #[inline]
    pub const fn try_offset(self, dx: isize, dy: isize) -> Option<Self> {
        match (self.file().try_offset(dx), self.rank().try_offset(dy)) {
            (Some(file), Some(rank)) => Some(Self::new(file, rank)),
            _ => None,
        }
    }

    #[inline]
    pub const fn offset(self, dx: isize, dy: isize) -> Self {
        Self::new(
            self.file()
                .try_offset(dx)
                .expect("Square::offset(dx, dy) New file index out of bounds"),
            self.rank()
                .try_offset(dy)
                .expect("Square::offset(dx, dy) New rank index out of bounds"),
        )
    }

    #[inline]
    pub const fn flip_file(self) -> Self {
        Self::index(self as usize ^ 7)
    }

    #[inline]
    pub const fn flip_rank(self) -> Self {
        Self::index(self as usize ^ 56)
    }

    #[inline]
    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip_rank(),
        }
    }

    #[inline]
    pub const fn file(self) -> File {
        File::index(self as usize & 7)
    }

    #[inline]
    pub const fn rank(self) -> Rank {
        Rank::index(self as usize >> 3)
    }

    #[inline]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(1u64 << (self as u8))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SquareParseError {
    FileParseError,
    RankParseError,
}

impl FromStr for Square {
    type Err = SquareParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars
            .next()
            .and_then(|c| File::try_from(c).ok())
            .ok_or(SquareParseError::FileParseError)?;
        let rank = chars
            .next()
            .and_then(|c| Rank::try_from(c).ok())
            .ok_or(SquareParseError::RankParseError)?;

        Ok(Self::new(file, rank))
    }
}

impl fmt::Display for Square {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Bitboard, File, Rank, Square};

    #[test]
    fn square_new() {
        assert_eq!(Square::new(File::A, Rank::First), Square::A1);
        assert_eq!(Square::new(File::H, Rank::First), Square::H1);
        assert_eq!(Square::new(File::A, Rank::Eighth), Square::A8);
        assert_eq!(Square::new(File::H, Rank::Eighth), Square::H8);
        assert_eq!(Square::new(File::E, Rank::Fourth), Square::E4);
    }

    #[test]
    fn square_new_round_trip() {
        for &sq in Square::ALL {
            assert_eq!(Square::new(sq.file(), sq.rank()), sq);
        }
    }

    #[test]
    fn square_flip_file() {
        assert_eq!(Square::A1.flip_file(), Square::H1);
        assert_eq!(Square::H1.flip_file(), Square::A1);
        assert_eq!(Square::A8.flip_file(), Square::H8);
        assert_eq!(Square::H8.flip_file(), Square::A8);
        assert_eq!(Square::E4.flip_file(), Square::D4);
    }

    #[test]
    fn square_flip_rank() {
        assert_eq!(Square::A1.flip_rank(), Square::A8);
        assert_eq!(Square::H1.flip_rank(), Square::H8);
        assert_eq!(Square::A8.flip_rank(), Square::A1);
        assert_eq!(Square::H8.flip_rank(), Square::H1);
        assert_eq!(Square::E4.flip_rank(), Square::E5);
    }

    #[test]
    fn square_double_flip_file() {
        for &sq in Square::ALL {
            assert_eq!(sq.flip_file().flip_file(), sq);
        }
    }

    #[test]
    fn square_double_flip_rank() {
        for &sq in Square::ALL {
            assert_eq!(sq.flip_rank().flip_rank(), sq);
        }
    }

    #[test]
    fn square_bitboard_unique() {
        let mut bb = Bitboard::EMPTY;

        for sq in Square::ALL.map(|sq| sq.bitboard()) {
            assert!(bb.is_disjoint(sq));
            bb |= sq;
        }

        assert_eq!(bb, Bitboard::FULL)
    }

    #[test]
    fn square_file() {
        assert_eq!(Square::A1.file(), File::A);
        assert_eq!(Square::H1.file(), File::H);
        assert_eq!(Square::A8.file(), File::A);
        assert_eq!(Square::H8.file(), File::H);
        assert_eq!(Square::E4.file(), File::E);
    }

    #[test]
    fn square_rank() {
        assert_eq!(Square::A1.rank(), Rank::First);
        assert_eq!(Square::H1.rank(), Rank::First);
        assert_eq!(Square::A8.rank(), Rank::Eighth);
        assert_eq!(Square::H8.rank(), Rank::Eighth);
        assert_eq!(Square::E4.rank(), Rank::Fourth);
    }
}
