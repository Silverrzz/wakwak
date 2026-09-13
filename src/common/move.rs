use crate::board::{Board, CastlingDirection};
use crate::common::{Bitboard, BitboardIter, File, Piece, Rank, Square};
use std::fmt::Write;
use std::num::NonZeroU32;

pub const MAX_MOVES: usize = 218 * Square::COUNT;

/// A duck chess move. Bit Layout:
/// - Bits 0-5: Source Square
/// - Bits 6-11: Target Square
/// - Bits 12-17: Duck Square
/// - Bits 18-21: Move Flag
/// - Bits 22-31: Unused
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(NonZeroU32);

impl Move {
    #[inline]
    pub const fn new(src: Square, dest: Square, duck: Square, flag: MoveFlag) -> Self {
        let mut bits = 0;
        bits |= src as u32;
        bits |= (dest as u32) << 6;
        bits |= (duck as u32) << 12;
        bits |= (flag as u32) << 18;

        Self(NonZeroU32::new(bits).unwrap())
    }

    #[inline]
    pub const fn src(self) -> Square {
        Square::index(self.0.get() as usize & 0b111111)
    }

    #[inline]
    pub const fn dest(self) -> Square {
        Square::index((self.0.get() as usize >> 6) & 0b111111)
    }

    #[inline]
    pub const fn duck(self) -> Square {
        Square::index((self.0.get() as usize >> 12) & 0b111111)
    }

    #[inline]
    pub const fn flag(self) -> MoveFlag {
        // SAFETY: All moves created using `Move::new()` will have a valid `MoveFlag` bit pattern.

        unsafe { core::mem::transmute((self.0.get() >> 18) as u8) }
    }

    #[inline]
    pub fn display(self, dumb_interface: bool, frc: bool) -> String {
        let (src, mut dest) = (self.src(), self.dest());

        if !frc && self.flag().is_castling() {
            let dest_file = if src.file() < dest.file() {
                File::G
            } else {
                File::C
            };

            dest = Square::new(dest_file, src.rank());
        }

        let mut out = String::new();
        write!(out, "{}{}", src, dest).unwrap();

        if let Some(promotion) = self.flag().promotion() {
            write!(out, "{}", promotion).unwrap();
        }

        if dumb_interface {
            write!(out, ",{}{}", dest, self.duck()).unwrap(); //just why
        } else {
            write!(out, "@{}", self.duck()).unwrap();
        }

        out
    }

    #[inline]
    pub fn parse(board: &Board, dumb_interface: bool, str: &str) -> Option<Move> {
        if str.len() < 7 {
            return None;
        }

        let src = str.get(0..2)?.parse::<Square>().ok()?;
        let mut dest = str.get(2..4)?.parse::<Square>().ok()?;

        let (promotion, duck) = if dumb_interface {
            match str.chars().nth(4) {
                Some(',') => (None, str.get(7..9)?.parse::<Square>().ok()?),
                Some(c) => (
                    c.try_into().ok().filter(|p| {
                        [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen].contains(p)
                    }),
                    str.get(8..10)?.parse::<Square>().ok()?,
                ),
                None => return None,
            }
        } else {
            match str.chars().nth(4) {
                Some('@') => (None, str.get(5..7)?.parse::<Square>().ok()?),
                Some(c) => (
                    c.try_into().ok().filter(|p| {
                        [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen].contains(p)
                    }),
                    str.get(6..8)?.parse::<Square>().ok()?,
                ),
                None => return None,
            }
        };

        if duck == dest {
            return None;
        }

        let is_capture = board.piece_on(dest).is_some();
        let flag = match board.piece_on(src)? {
            Piece::Pawn => Self::parse_pawn_flag(board, src, dest, duck, promotion, is_capture)?,
            Piece::King => Self::parse_king_flag(board, src, &mut dest, duck, is_capture)?,
            _ => {
                if !(board.occupied() ^ src).has(duck) {
                    if is_capture {
                        MoveFlag::Capture
                    } else {
                        MoveFlag::Normal
                    }
                } else {
                    return None;
                }
            }
        };

        Some(Move::new(src, dest, duck, flag))
    }

    #[inline]
    fn parse_pawn_flag(
        board: &Board,
        src: Square,
        dest: Square,
        duck: Square,
        promotion: Option<Piece>,
        is_capture: bool,
    ) -> Option<MoveFlag> {
        let stm = board.stm();

        if let Some(promotion) = promotion {
            if is_capture {
                MoveFlag::new_capture_promotion(promotion)
            } else {
                MoveFlag::new_promotion(promotion)
            }
        } else if is_capture {
            Some(MoveFlag::Capture)
        } else if let Some(en_passant) = board.en_passant()
            && dest == Square::new(en_passant.file(), Rank::Sixth.relative_to(stm))
        {
            let blockers = board.occupied()
                ^ Square::new(en_passant.file(), Rank::Fifth.relative_to(stm))
                ^ dest
                ^ src;
            if !blockers.has(duck) {
                Some(MoveFlag::EnPassant)
            } else {
                None
            }
        } else if src.rank() == Rank::Second.relative_to(stm)
            && dest.rank() == Rank::Fourth.relative_to(stm)
        {
            Some(MoveFlag::DoublePush)
        } else {
            Some(MoveFlag::Normal)
        }
    }

    #[inline]
    fn parse_king_flag(
        board: &Board,
        src: Square,
        dest: &mut Square,
        duck: Square,
        is_capture: bool,
    ) -> Option<MoveFlag> {
        let stm = board.stm();
        let our_back_rank = Rank::First.relative_to(stm);

        if is_capture {
            return if board.color_on(*dest) == Some(stm) {
                let rights = board.castling_rights(stm);

                if Some(*dest)
                    == rights
                        .get(CastlingDirection::Short)
                        .map(|f| Square::new(f, our_back_rank))
                {
                    let blockers = board.occupied() ^ src ^ *dest ^ Bitboard(0x60).relative_to(stm);
                    if !blockers.has(duck) {
                        Some(MoveFlag::ShortCastling)
                    } else {
                        None
                    }
                } else if Some(*dest)
                    == rights
                        .get(CastlingDirection::Long)
                        .map(|f| Square::new(f, our_back_rank))
                {
                    let blockers = board.occupied() ^ src ^ *dest ^ Bitboard(0x6).relative_to(stm);
                    if !blockers.has(duck) {
                        Some(MoveFlag::LongCastling)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                Some(MoveFlag::Capture)
            };
        }

        let castling_src = Square::new(File::E, our_back_rank);
        if src == castling_src {
            let rights = board.castling_rights(stm);
            let short_dest = Square::new(File::G, our_back_rank);
            let long_dest = Square::new(File::C, our_back_rank);

            if let Some(rook_src) = rights.get(CastlingDirection::Short)
                && *dest == short_dest
            {
                *dest = Square::new(rook_src, our_back_rank);

                let blockers = board.occupied() ^ src ^ *dest ^ Bitboard(0x60).relative_to(stm);
                return if !blockers.has(duck) {
                    Some(MoveFlag::ShortCastling)
                } else {
                    None
                };
            } else if let Some(rook_src) = rights.get(CastlingDirection::Long)
                && *dest == long_dest
            {
                *dest = Square::new(rook_src, our_back_rank);

                let blockers = board.occupied() ^ src ^ *dest ^ Bitboard(0x6).relative_to(stm);
                return if !blockers.has(duck) {
                    Some(MoveFlag::LongCastling)
                } else {
                    None
                };
            }
        }

        Some(MoveFlag::Normal)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    Normal = 0x0,
    DoublePush = 0x1,
    LongCastling = 0x2,
    ShortCastling = 0x3,
    PromotionQueen = 0x4,
    PromotionRook = 0x5,
    PromotionBishop = 0x6,
    PromotionKnight = 0x7,
    Capture = 0x8,
    EnPassant = 0x9,
    CapturePromotionQueen = 0xC,
    CapturePromotionRook = 0xD,
    CapturePromotionBishop = 0xE,
    CapturePromotionKnight = 0xF,
}

impl MoveFlag {
    #[inline]
    pub const fn new_castling(dir: CastlingDirection) -> MoveFlag {
        match dir {
            CastlingDirection::Long => MoveFlag::LongCastling,
            CastlingDirection::Short => MoveFlag::ShortCastling,
        }
    }

    #[inline]
    pub const fn new_promotion(piece: Piece) -> Option<MoveFlag> {
        match piece {
            Piece::Knight => Some(MoveFlag::PromotionKnight),
            Piece::Bishop => Some(MoveFlag::PromotionBishop),
            Piece::Rook => Some(MoveFlag::PromotionRook),
            Piece::Queen => Some(MoveFlag::PromotionQueen),
            _ => None,
        }
    }

    #[inline]
    pub const fn new_capture_promotion(piece: Piece) -> Option<MoveFlag> {
        match piece {
            Piece::Knight => Some(MoveFlag::CapturePromotionKnight),
            Piece::Bishop => Some(MoveFlag::CapturePromotionBishop),
            Piece::Rook => Some(MoveFlag::CapturePromotionRook),
            Piece::Queen => Some(MoveFlag::CapturePromotionQueen),
            _ => None,
        }
    }

    #[inline]
    pub const fn promotion(self) -> Option<Piece> {
        if !self.is_promotion() {
            return None;
        }

        const LOOKUP: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

        Some(LOOKUP[self as usize & 0x3])
    }

    #[inline]
    pub const fn is_castling(self) -> bool {
        matches!(self, MoveFlag::LongCastling | MoveFlag::ShortCastling)
    }

    #[inline]
    pub const fn is_promotion(self) -> bool {
        (self as u8 & 0x4) != 0
    }

    #[inline]
    pub const fn is_capture_promotion(self) -> bool {
        (self as u8 & 0xC) != 0
    }

    #[inline]
    pub const fn is_capture(self) -> bool {
        (self as u8 & 0x8) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DuckMoves {
    pub src: Square,
    pub dest: Square,
    pub flag: MoveFlag,
    pub duck: Bitboard,
}

impl DuckMoves {
    #[inline]
    pub fn new(src: Square, dest: Square, flag: MoveFlag, duck: Bitboard) -> Self {
        Self {
            src,
            dest,
            duck,
            flag,
        }
    }

    #[inline]
    pub fn has(&self, mv: Move) -> bool {
        self.src == mv.src()
            && self.dest == mv.dest()
            && self.duck.has(mv.duck())
            && self.flag == mv.flag()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.duck.popcnt()
    }

    #[inline]
    pub fn is_nonempty(&self) -> bool {
        self.duck.is_nonempty()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.duck.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> DuckMovesIter {
        self.into_iter()
    }
}

impl IntoIterator for DuckMoves {
    type Item = Move;
    type IntoIter = DuckMovesIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        DuckMovesIter {
            src: self.src,
            dest: self.dest,
            flag: self.flag,
            duck: self.duck.iter(),
        }
    }
}

pub struct DuckMovesIter {
    src: Square,
    dest: Square,
    flag: MoveFlag,
    duck: BitboardIter,
}

impl Iterator for DuckMovesIter {
    type Item = Move;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let duck = self.duck.next()?;
        Some(Move::new(self.src, self.dest, duck, self.flag))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for DuckMovesIter {
    #[inline]
    fn len(&self) -> usize {
        self.duck.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{MoveFlag, Piece};

    #[test]
    fn move_flags() {
        let flags = [
            MoveFlag::Normal,
            MoveFlag::DoublePush,
            MoveFlag::LongCastling,
            MoveFlag::ShortCastling,
            MoveFlag::PromotionQueen,
            MoveFlag::PromotionRook,
            MoveFlag::PromotionBishop,
            MoveFlag::PromotionKnight,
            MoveFlag::Capture,
            MoveFlag::EnPassant,
            MoveFlag::CapturePromotionQueen,
            MoveFlag::CapturePromotionRook,
            MoveFlag::CapturePromotionBishop,
            MoveFlag::CapturePromotionKnight,
        ];

        let expected_promo = [
            None,
            None,
            None,
            None,
            Some(Piece::Queen),
            Some(Piece::Rook),
            Some(Piece::Bishop),
            Some(Piece::Knight),
            None,
            None,
            Some(Piece::Queen),
            Some(Piece::Rook),
            Some(Piece::Bishop),
            Some(Piece::Knight),
        ];

        for (&flag, &expected_promo) in flags.iter().zip(expected_promo.iter()) {
            assert_eq!(flag.promotion(), expected_promo);
            assert_eq!(flag.is_promotion(), expected_promo.is_some());
        }

        let expected_capture = [
            false, false, false, false, false, false, false, false, true, true, true, true, true,
            true,
        ];

        for (&flag, &expected_capture) in flags.iter().zip(expected_capture.iter()) {
            assert_eq!(flag.is_capture(), expected_capture);
        }
    }
}
