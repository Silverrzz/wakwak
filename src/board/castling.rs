use crate::common::File;

/// The castling rights for one side. Bit Layout:
/// - Bits 0-2: Long File
/// - Bit 3: Long Flag
/// - Bits 4-6: Short File
/// - Bit 7: Short Flag
///
/// For the flags, 0 is used to indicate that the king may castle in that direction.
/// This way, the nibble can be used as the file index.
///
/// This slightly overengineered tech was adapted from [Icarus](https://github.com/Sp00ph/icarus/).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    #[inline]
    pub fn new(long: Option<File>, short: Option<File>) -> Self {
        let mut bits = 0;
        bits |= long.map_or(8, |f| f as u8);
        bits |= short.map_or(8, |f| f as u8) << 4;

        Self(bits)
    }

    #[inline]
    pub fn set(&mut self, dir: CastlingDirection, file: Option<File>) {
        self.0 &= !(0xF << dir as u8); //Mask out the old file
        self.0 |= file.map_or(8, |f| f as u8) << dir as u8; //Put in the new file
    }

    #[inline]
    pub fn get(self, dir: CastlingDirection) -> Option<File> {
        File::try_index((self.0 as usize >> dir as usize) & 0xF)
    }
}

impl Default for CastlingRights {
    #[inline]
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CastlingDirection {
    Long = 0,
    Short = 4,
}

impl CastlingDirection {
    #[inline]
    pub const fn king_dest(self) -> File {
        match self {
            CastlingDirection::Long => File::C,
            CastlingDirection::Short => File::G,
        }
    }

    #[inline]
    pub const fn rook_dest(self) -> File {
        match self {
            CastlingDirection::Long => File::D,
            CastlingDirection::Short => File::F,
        }
    }
}
