use crate::common::{Bitboard, Color, File, Rank, Square};
use std::num::NonZeroU8;

/// Bit Layout:
/// - Bits 0-2: File
/// - Bit 3: Pawn on the left can capture
/// - Bit 4: Pawn on the right can capture
///
/// This slightly overengineered tech was adapted from [Icarus](https://github.com/Sp00ph/icarus/).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnPassant {
    bits: NonZeroU8,
}

impl EnPassant {
    #[inline]
    pub fn new(file: File, left: bool, right: bool) -> EnPassant {
        let mut bits = 0;
        bits |= file as u8;
        bits |= (left as u8) << 3;
        bits |= (right as u8) << 4;

        EnPassant {
            bits: NonZeroU8::new(bits).unwrap(),
        }
    }

    #[inline]
    pub fn attackers(self, stm: Color) -> Bitboard {
        let victim = Square::new(self.file(), Rank::Fifth.relative_to(stm));
        let mut bb = Bitboard::EMPTY;

        if self.left() {
            bb |= Square::index(victim as usize - 1);
        }

        if self.right() {
            bb |= Square::index(victim as usize + 1);
        }

        bb
    }

    #[inline]
    pub fn file(self) -> File {
        File::index((self.bits.get() & 0x7) as usize)
    }

    #[inline]
    pub fn left(self) -> bool {
        (self.bits.get() & 0x8) != 0
    }

    #[inline]
    pub fn right(self) -> bool {
        (self.bits.get() & 0x10) != 0
    }
}
