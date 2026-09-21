use crate::board::{
    CastlingDirection, CastlingRights, EnPassant, SliderTag, ZOBRIST, bishop_attacks, rook_attacks,
};
use crate::common::{
    Bitboard, Color, File, Move, MoveFlag, North, NorthEast, NorthWest, Piece, Rank, Square,
    between, king_attacks, knight_attacks, pawn_attacks,
};
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
    pub(super) pawn_hash: u64,
    pub(super) minor_hash: u64,
    pub(super) major_hash: u64,
    pub(super) white_hash: u64,
    pub(super) black_hash: u64,
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
    pub fn occupied_duckless(&self) -> Bitboard {
        self.colors[Color::White] | self.colors[Color::Black]
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
    pub fn pawn_hash(&self) -> u64 {
        self.pawn_hash
    }

    #[inline]
    pub fn minor_hash(&self) -> u64 {
        self.minor_hash
    }

    #[inline]
    pub fn major_hash(&self) -> u64 {
        self.major_hash
    }

    #[inline]
    pub fn white_hash(&self) -> u64 {
        self.white_hash
    }

    #[inline]
    pub fn black_hash(&self) -> u64 {
        self.black_hash
    }

    #[inline]
    pub fn duckless_hash(&self) -> u64 {
        self.hash ^ self.duck.map_or(0, |sq| ZOBRIST.duck(sq))
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
    pub fn in_check(&self) -> bool {
        // TODO: maybe make it incremental (?) idk
        let blocks = self.king_capture_blocks(self.stm);

        blocks != Bitboard::FULL && self.duck.is_none_or(|sq| !blocks.has(sq))
    }

    #[inline]
    pub fn king_capture_blocks(&self, color: Color) -> Bitboard {
        if self.try_king(!color).is_none() {
            return Bitboard::FULL;
        }
        let king = self.king(color);
        let unblockable = (pawn_attacks(king, color) & self.colored_pieces(!color, Piece::Pawn))
            | (knight_attacks(king) & self.colored_pieces(!color, Piece::Knight))
            | (king_attacks(king) & self.king(!color));
        if unblockable.is_nonempty() {
            return Bitboard::EMPTY;
        }
        let blockers = self.colors(color) | self.colors(!color);
        let sliders = (bishop_attacks(blockers, king, self.slider_tag)
            & self.colored_diag_sliders(!color))
            | (rook_attacks(blockers, king, self.slider_tag) & self.colored_orth_sliders(!color));
        let mut safe = Bitboard::FULL;
        for attacker in sliders {
            safe &= between(king, attacker);
        }
        safe
    }

    #[inline]
    pub fn king_capture_blocks_after(&self, mv: Move) -> Bitboard {
        let mut colors = self.colors;
        let mut pieces = self.pieces;
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());

        let mut toggle_square = |piece: Piece, color: Color, sq: Square| {
            colors[color] ^= sq;
            pieces[piece] ^= sq;
        };

        let piece = self
            .piece_on(src)
            .expect("Board::king_capture_blocks_after(): Empty source square");
        if let Some(dir) = flag.castling_dir() {
            let rank = Rank::First.relative_to(self.stm);
            let king_dest = Square::new(dir.king_dest(), rank);
            let rook_dest = Square::new(dir.rook_dest(), rank);

            toggle_square(Piece::King, self.stm, src);
            toggle_square(Piece::Rook, self.stm, dest);
            toggle_square(Piece::King, self.stm, king_dest);
            toggle_square(Piece::Rook, self.stm, rook_dest);
        } else if let Some(promo) = flag.promotion() {
            toggle_square(Piece::Pawn, self.stm, src);
            toggle_square(promo, self.stm, dest);
        } else {
            toggle_square(piece, self.stm, src);
            toggle_square(piece, self.stm, dest);
        }

        if flag == MoveFlag::EnPassant {
            let victim = Square::new(dest.file(), src.rank());
            toggle_square(Piece::Pawn, !self.stm, victim);
        } else if flag.is_capture()
            && let Some(piece) = self.piece_on(dest)
        {
            toggle_square(piece, !self.stm, dest);
        }

        let colored_pieces = |color: Color, piece: Piece| colors[color] & pieces[piece];
        let colored_diag_sliders =
            |color: Color| colors[color] & (pieces[Piece::Bishop] | pieces[Piece::Queen]);
        let colored_orth_sliders =
            |color: Color| colors[color] & (pieces[Piece::Rook] | pieces[Piece::Queen]);

        let Some(their_king) = colored_pieces(!self.stm, Piece::King).try_next() else {
            return Bitboard::FULL;
        };

        let our_king = colored_pieces(self.stm, Piece::King).next();
        let unblockable = (pawn_attacks(our_king, self.stm)
            & colored_pieces(!self.stm, Piece::Pawn))
            | (knight_attacks(our_king) & colored_pieces(!self.stm, Piece::Knight))
            | (king_attacks(our_king) & their_king);
        if unblockable.is_nonempty() {
            return Bitboard::EMPTY;
        }

        let blockers = colors[Color::White] | colors[Color::Black];
        let sliders = (bishop_attacks(blockers, our_king, self.slider_tag)
            & colored_diag_sliders(!self.stm))
            | (rook_attacks(blockers, our_king, self.slider_tag) & colored_orth_sliders(!self.stm));

        let mut safe = Bitboard::FULL;
        for attacker in sliders {
            safe &= between(our_king, attacker);
        }

        safe
    }

    #[inline]
    pub fn attackers_to(&self, dest: Square, occupied: Bitboard) -> Bitboard {
        let orth = self.orth_sliders();
        let diag = self.diag_sliders();

        occupied
            & ((pawn_attacks(dest, Color::White) & self.colored_pieces(Color::Black, Piece::Pawn))
                | (pawn_attacks(dest, Color::Black)
                    & self.colored_pieces(Color::White, Piece::Pawn))
                | (knight_attacks(dest) & self.pieces(Piece::Knight))
                | (bishop_attacks(occupied, dest, self.slider_tag) & diag)
                | (rook_attacks(occupied, dest, self.slider_tag) & orth)
                | (king_attacks(dest) & self.pieces(Piece::King)))
    }

    #[inline]
    pub fn neutral_ducks(&self) -> Bitboard {
        let them = !self.stm;
        let blockers = self.colors(them);
        let pawns = self.colored_pieces(them, Piece::Pawn);
        let mut relevant = pawns.shift::<North>(them.signum())
            | (pawns & Rank::Second.relative_to(them)).shift::<North>(2 * them.signum())
            | pawns.shift::<NorthEast>(them.signum())
            | pawns.shift::<NorthWest>(them.signum())
            | king_attacks(self.king(them));

        for square in self.colored_pieces(them, Piece::Knight) {
            relevant |= knight_attacks(square);
        }
        for square in self.colored_diag_sliders(them) {
            relevant |= bishop_attacks(blockers, square, self.slider_tag);
        }
        for square in self.colored_orth_sliders(them) {
            relevant |= rook_attacks(blockers, square, self.slider_tag);
        }
        if CastlingDirection::ALL
            .iter()
            .any(|&dir| self.castling_rights(them).get(dir).is_some())
        {
            relevant |= Rank::First.relative_to(them);
        }
        !relevant
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

        let value = ZOBRIST.piece(sq, piece, color);
        self.hash ^= value;

        match piece {
            Piece::Pawn => self.pawn_hash ^= value,
            Piece::Knight => self.minor_hash ^= value,
            Piece::Bishop => self.minor_hash ^= value,
            Piece::Rook => self.major_hash ^= value,
            Piece::Queen => self.major_hash ^= value,
            Piece::King => {
                self.minor_hash ^= value;
                self.major_hash ^= value;
            }
        }

        if piece != Piece::Pawn {
            match color {
                Color::White => self.white_hash ^= value,
                Color::Black => self.black_hash ^= value,
            }
        }
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

    #[inline]
    pub fn slider_tag(&self) -> SliderTag {
        self.slider_tag
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TerminalState {
    Victory(Color),
    Stalemate(Color),
    Draw,
}
