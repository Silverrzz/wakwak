use crate::board::{CastlingDirection, CastlingRights, EnPassant, SliderTag, ZOBRIST};
use crate::common::{Bitboard, Color, File, Piece, Rank, Square, pawn_attacks};
use enum_map::EnumMap;

#[derive(Clone, Copy)]
pub struct Board {
    pub(super) pieces: EnumMap<Piece, Bitboard>,
    pub(super) colors: EnumMap<Color, Bitboard>,
    pub(super) mailbox: EnumMap<Square, Option<Piece>>,
    pub(super) castling_rights: EnumMap<Color, CastlingRights>,
    pub(super) en_passant: Option<EnPassant>,
    pub(super) duck: Option<Square>,
    pub(super) hash: u64,
    pub(super) stm: Color,
    pub(super) fmc: u16,
    pub(super) hmc: u8,
    pub(super) slider_tag: SliderTag,
}

impl Board {
    #[inline]
    pub fn occupied(&self) -> Bitboard {
        self.colors[Color::White]
            | self.colors[Color::Black]
            | self.duck.map_or(Bitboard::EMPTY, Square::bitboard)
    }

    #[inline]
    pub fn colors(&self, color: Color) -> Bitboard {
        self.colors[color]
    }

    #[inline]
    pub fn pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece]
    }

    #[inline]
    pub fn colored_pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.pieces(piece) & self.colors(color)
    }

    #[inline]
    pub fn diag_sliders(&self) -> Bitboard {
        self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
    }

    #[inline]
    pub fn colored_diag_sliders(&self, color: Color) -> Bitboard {
        self.diag_sliders() & self.colors(color)
    }

    #[inline]
    pub fn orth_sliders(&self) -> Bitboard {
        self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
    }

    #[inline]
    pub fn colored_orth_sliders(&self, color: Color) -> Bitboard {
        self.orth_sliders() & self.colors(color)
    }

    #[inline]
    pub fn king(&self, color: Color) -> Square {
        self.colored_pieces(color, Piece::King).next()
    }

    #[inline]
    pub fn try_king(&self, color: Color) -> Option<Square> {
        self.colored_pieces(color, Piece::King).try_next()
    }

    #[inline]
    pub fn castling_rights(&self, color: Color) -> CastlingRights {
        self.castling_rights[color]
    }

    #[inline]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.mailbox[sq]
    }

    #[inline]
    pub fn color_on(&self, sq: Square) -> Option<Color> {
        if self.colors(Color::White).has(sq) {
            Some(Color::White)
        } else if self.colors(Color::Black).has(sq) {
            Some(Color::Black)
        } else {
            None
        }
    }

    #[inline]
    pub fn en_passant(&self) -> Option<EnPassant> {
        self.en_passant
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    pub fn duck(&self) -> Option<Square> {
        self.duck
    }

    #[inline]
    pub fn stm(&self) -> Color {
        self.stm
    }

    #[inline]
    pub fn fmc(&self) -> u16 {
        self.fmc
    }

    #[inline]
    pub fn hmc(&self) -> u8 {
        self.hmc
    }

    #[inline]
    pub fn terminal_state(&self) -> Option<TerminalState> {
        if self.try_king(self.stm).is_none() {
            return Some(TerminalState::Victory(!self.stm));
        }

        if self.any_moves(|_| true) {
            //TODO: Insufficient Material (?)
            if self.hmc >= 100 {
                Some(TerminalState::Draw)
            } else {
                None
            }
        } else {
            Some(TerminalState::Stalemate(self.stm))
        }
    }

    #[inline]
    pub fn calc_en_passant(&mut self, file: Option<File>) {
        let Some(file) = file else {
            self.set_en_passant(None);
            return;
        };

        let victim = Square::new(file, Rank::Fifth.relative_to(self.stm));
        let attacker_dest = Square::new(file, Rank::Sixth.relative_to(self.stm));
        let our_pawns = self.colored_pieces(self.stm, Piece::Pawn);

        let attackers = our_pawns & pawn_attacks(attacker_dest, !self.stm);
        if attackers.is_empty() || self.occupied().has(attacker_dest) {
            self.set_en_passant(None);
            return;
        }

        let (mut left, mut right) = (false, false);
        for attacker in attackers.iter().take(2) {
            if attacker.file() < victim.file() {
                left = true;
            } else {
                right = true;
            }
        }

        self.set_en_passant((left | right).then(|| EnPassant::new(file, left, right)));
    }

    #[inline]
    pub fn toggle_square(&mut self, sq: Square, piece: Piece, color: Color) {
        self.pieces[piece] ^= sq;
        self.colors[color] ^= sq;
        self.hash ^= ZOBRIST.piece(sq, piece, color);
    }

    #[inline]
    pub fn set_castling_rights(
        &mut self,
        color: Color,
        dir: CastlingDirection,
        file: Option<File>,
    ) {
        if let Some(old) = self.castling_rights[color].get(dir) {
            self.hash ^= ZOBRIST.castling_rights(color, old);
        }

        if let Some(new) = file {
            self.hash ^= ZOBRIST.castling_rights(color, new);
        }

        self.castling_rights[color].set(dir, file);
    }

    #[inline]
    pub fn set_en_passant(&mut self, en_passant: Option<EnPassant>) {
        if let Some(prev) = core::mem::replace(&mut self.en_passant, en_passant) {
            self.hash ^= ZOBRIST.en_passant(prev.file());
        }

        if let Some(ep) = en_passant {
            self.hash ^= ZOBRIST.en_passant(ep.file());
        }
    }

    #[inline]
    pub fn set_duck(&mut self, duck: Option<Square>) {
        if let Some(prev) = core::mem::replace(&mut self.duck, duck) {
            self.hash ^= ZOBRIST.duck(prev);
        }

        if let Some(sq) = duck {
            self.hash ^= ZOBRIST.duck(sq);
        }
    }

    #[inline]
    pub fn toggle_stm(&mut self) {
        self.stm = !self.stm;
        self.hash ^= ZOBRIST.stm;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TerminalState {
    Victory(Color),
    Stalemate(Color),
    Draw,
}
