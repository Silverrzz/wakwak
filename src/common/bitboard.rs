use crate::common::{Color, Direction, File, Rank, Square, horizontal_shift_mask};
use std::fmt;
use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline]
    pub const fn shift<D: Direction>(self, steps: i8) -> Bitboard {
        let mask = horizontal_shift_mask(D::DX * steps);
        let shift = (D::DX + D::DY * 8) * steps;

        //`shl` takes an `isize` as a parameter but panics if you try to shift by a negative number.
        Bitboard(if shift > 0 {
            (self.0 << shift) & mask
        } else {
            (self.0 >> -shift) & mask
        })
    }

    #[inline]
    pub const fn smear<D: Direction>(self) -> Bitboard {
        let mut result = self;

        result.0 |= result.shift::<D>(1).0;
        result.0 |= result.shift::<D>(2).0;
        result.0 |= result.shift::<D>(4).0;

        result
    }

    #[inline]
    pub const fn flip_files(self) -> Bitboard {
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0F0F0F0F0F0F0F0F;

        let mut result = self.0;
        result = ((result >> 1) & K1) | ((result & K1) << 1);
        result = ((result >> 2) & K2) | ((result & K2) << 2);
        result = ((result >> 4) & K4) | ((result & K4) << 4);

        Bitboard(result)
    }

    #[inline]
    pub const fn flip_ranks(self) -> Bitboard {
        Bitboard(self.0.swap_bytes())
    }

    #[inline]
    pub const fn relative_to(self, color: Color) -> Bitboard {
        match color {
            Color::White => self,
            Color::Black => self.flip_ranks(),
        }
    }

    #[inline]
    pub const fn iter(self) -> BitboardIter {
        BitboardIter(self)
    }

    #[inline]
    pub const fn next(self) -> Square {
        self.try_next()
            .expect("Bitboard::next() called on an empty Bitboard")
    }

    #[inline]
    pub const fn try_next(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }

    #[inline]
    pub const fn next_back(self) -> Square {
        self.try_next_back()
            .expect("Bitboard::next_back() called on an empty Bitboard")
    }

    #[inline]
    pub const fn try_next_back(self) -> Option<Square> {
        Square::try_index(63usize.wrapping_sub(self.0.leading_zeros() as usize))
    }

    #[inline]
    pub const fn is_superset(self, rhs: Bitboard) -> bool {
        rhs.is_subset(self)
    }

    #[inline]
    pub const fn is_subset(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == self.0
    }

    #[inline]
    pub const fn is_disjoint(self, rhs: Bitboard) -> bool {
        self.0 & rhs.0 == 0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_nonempty(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn popcnt(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    pub const fn has(self, sq: Square) -> bool {
        !self.is_disjoint(sq.bitboard())
    }

    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Not for Bitboard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &rank in Rank::ALL.iter().rev() {
            writeln!(f)?;
            for &file in File::ALL {
                if self.has(Square::new(file, rank)) {
                    write!(f, " x")?;
                } else {
                    write!(f, " .")?;
                }
            }
        }

        writeln!(f)?;
        Ok(())
    }
}

macro_rules! impl_bb_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Bitboard {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Bitboard(self.0.$fn(rhs.0))
            }
        }

        impl $trait<File> for Bitboard {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: File) -> Self::Output {
                self.$fn(rhs.bitboard())
            }
        }

        impl $trait<Rank> for Bitboard {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Rank) -> Self::Output {
                self.$fn(rhs.bitboard())
            }
        }

        impl $trait<Square> for Bitboard {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Square) -> Self::Output {
                self.$fn(rhs.bitboard())
            }
        }
    )*}
}

macro_rules! impl_bb_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: Self) {
                self.0.$fn(rhs.0);
            }
        }

        impl $trait<File> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: File) {
                self.$fn(rhs.bitboard());
            }
        }

        impl $trait<Rank> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: Rank) {
                self.$fn(rhs.bitboard());
            }
        }

        impl $trait<Square> for Bitboard {
            #[inline]
            fn $fn(&mut self, rhs: Square) {
                self.$fn(rhs.bitboard());
            }
        }
    )*}
}

macro_rules! impl_bb_shift_ops {
    ($($ty:ty,)*) => {$(
        impl Shl<$ty> for Bitboard {
            type Output = Self;

            #[inline]
            fn shl(self, rhs: $ty) -> Self::Output {
                Bitboard(self.0 << rhs)
            }
        }

        impl Shr<$ty> for Bitboard {
            type Output = Self;

            #[inline]
            fn shr(self, rhs: $ty) -> Self::Output {
                Bitboard(self.0 >> rhs)
            }
        }

        impl ShlAssign<$ty> for Bitboard {
            #[inline]
            fn shl_assign(&mut self, rhs: $ty) {
                self.0 <<= rhs;
            }
        }

        impl ShrAssign<$ty> for Bitboard {
            #[inline]
            fn shr_assign(&mut self, rhs: $ty) {
                self.0 >>= rhs;
            }
        }
    )*}
}

impl_bb_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

impl_bb_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl_bb_shift_ops! {
    u8, u16, u32, u64, usize,
}

pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.0.try_next();

        if let Some(sq) = sq {
            self.0 ^= sq.bitboard();
        }

        sq
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl DoubleEndedIterator for BitboardIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let sq = self.0.try_next_back();

        if let Some(sq) = sq {
            self.0 ^= sq.bitboard();
        }

        sq
    }
}

impl ExactSizeIterator for BitboardIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.popcnt()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{
        Bitboard, East, File, North, NorthEast, NorthWest, South, SouthEast, SouthWest, Square,
        West,
    };

    #[test]
    fn bitboard_empty() {
        assert!(Bitboard::EMPTY.is_empty());
        assert!(!Bitboard::EMPTY.is_nonempty());

        assert!(!Bitboard::FULL.is_empty());
        assert!(Bitboard::FULL.is_nonempty());

        assert!(!Square::E4.bitboard().is_empty());
        assert!(Square::E4.bitboard().is_nonempty());
    }

    #[test]
    fn bitboard_popcnt() {
        assert_eq!(Bitboard::EMPTY.popcnt(), 0);
        assert_eq!(Bitboard::FULL.popcnt(), 64);

        assert_eq!(Bitboard(0b1010).popcnt(), 2);
        assert_eq!(Square::E4.bitboard().popcnt(), 1);
    }

    #[test]
    fn bitboard_try_next() {
        assert_eq!(Bitboard::EMPTY.try_next(), None);
        assert_eq!(Bitboard::EMPTY.try_next_back(), None);

        let bb = Square::A1.bitboard() | Square::E4.bitboard() | Square::H8.bitboard();

        assert_eq!(bb.try_next(), Some(Square::A1));
        assert_eq!(bb.try_next_back(), Some(Square::H8));
    }

    #[test]
    #[should_panic]
    fn bitboard_next_panics() {
        Bitboard::EMPTY.next();
    }

    #[test]
    #[should_panic]
    fn bitboard_next_back_panics() {
        Bitboard::EMPTY.next_back();
    }

    #[test]
    fn bitboard_subset_superset() {
        let a = Bitboard(0b0011);
        let b = Bitboard(0b0111);
        let c = Bitboard(0b1000);

        assert!(a.is_subset(b));
        assert!(b.is_superset(a));

        assert!(!b.is_subset(a));
        assert!(!a.is_superset(b));

        assert!(a.is_subset(a));
        assert!(a.is_superset(a));

        assert!(!a.is_subset(c));
        assert!(!a.is_superset(c));
        assert!(!b.is_subset(c));
        assert!(!b.is_superset(c));
    }

    #[test]
    fn bitboard_disjoint() {
        let a = Bitboard(0b0011);
        let b = Bitboard(0b1100);
        let c = Bitboard(0b0110);

        assert!(a.is_disjoint(b));
        assert!(b.is_disjoint(a));
        assert!(!a.is_disjoint(c));
        assert!(!b.is_disjoint(c));
        assert!(Bitboard::EMPTY.is_disjoint(Bitboard::FULL));
    }

    #[test]
    fn bitboard_has_sq() {
        let bb = Square::E4.bitboard() | Square::H8.bitboard();

        assert!(bb.has(Square::E4));
        assert!(bb.has(Square::H8));
        assert!(!bb.has(Square::A1));
        assert!(!bb.has(Square::D4));
    }

    #[test]
    fn bitboard_iter_ascending() {
        let bb = Square::A1.bitboard() | Square::E4.bitboard() | Square::H8.bitboard();
        let expected = vec![Square::A1, Square::E4, Square::H8];
        let squares: Vec<Square> = bb.iter().collect();

        assert_eq!(squares, expected);
    }

    #[test]
    fn bitboard_iter_descending() {
        let bb = Square::A1.bitboard() | Square::E4.bitboard() | Square::H8.bitboard();
        let expected = vec![Square::H8, Square::E4, Square::A1];
        let squares: Vec<Square> = bb.iter().rev().collect();

        assert_eq!(squares, expected);
    }

    #[test]
    fn bitboard_iter_empty() {
        let mut iter = Bitboard::EMPTY.iter();

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn bitboard_shift() {
        assert_eq!(
            Square::E4.bitboard().shift::<North>(1),
            Square::E5.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<NorthEast>(1),
            Square::F5.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<East>(1),
            Square::F4.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<SouthEast>(1),
            Square::F3.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<South>(1),
            Square::E3.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<SouthWest>(1),
            Square::D3.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<West>(1),
            Square::D4.bitboard()
        );
        assert_eq!(
            Square::E4.bitboard().shift::<NorthWest>(1),
            Square::D5.bitboard()
        );
    }

    #[test]
    fn bitboard_smear() {
        assert_eq!(
            Square::E4.bitboard().smear::<North>(),
            Bitboard(0x1010101010000000)
        );
        assert_eq!(
            Square::E4.bitboard().smear::<NorthEast>(),
            Bitboard(0x80402010000000)
        );
        assert_eq!(Square::E4.bitboard().smear::<East>(), Bitboard(0xF0000000));
        assert_eq!(
            Square::E4.bitboard().smear::<SouthEast>(),
            Bitboard(0x10204080)
        );
        assert_eq!(Square::E4.bitboard().smear::<South>(), Bitboard(0x10101010));
        assert_eq!(
            Square::E4.bitboard().smear::<SouthWest>(),
            Bitboard(0x10080402)
        );
        assert_eq!(Square::E4.bitboard().smear::<West>(), Bitboard(0x1F000000));
        assert_eq!(
            Square::E4.bitboard().smear::<NorthWest>(),
            Bitboard(0x102040810000000)
        );

        assert_eq!(File::A.bitboard().smear::<East>(), Bitboard::FULL);
        assert_eq!(
            File::A.bitboard().smear::<SouthEast>(),
            Bitboard(0x103070F1F3F7FFF)
        );
        assert_eq!(File::H.bitboard().smear::<West>(), Bitboard::FULL);
        assert_eq!(
            File::H.bitboard().smear::<NorthWest>(),
            Bitboard(0xFFFEFCF8F0E0C080)
        );
    }
}
