use crate::board::{CastlingDirection, CastlingRights, EnPassant, ZOBRIST};
use crate::common::{
    Bitboard, Color, East, File, Move, MoveFlag, MoveList, North, Piece, Rank, South, Square, West,
    between, bishop_rays, pawn_attacks, rook_rays,
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
    pub(super) stm: Color,
    pub(super) fmc: u16,
    pub(super) hmc: u8,
}

impl Board {
    #[inline]
    pub fn occupied(&self) -> Bitboard {
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
    pub fn calc_en_passant(&mut self, file: File) {
        let victim = Square::new(file, Rank::Fifth.relative_to(self.stm));
        let attacker_dest = Square::new(file, Rank::Sixth.relative_to(self.stm));
        let our_pawns = self.colored_pieces(self.stm, Piece::Pawn);
        let our_king = self.king(self.stm);

        let attackers = our_pawns & pawn_attacks(attacker_dest, !self.stm);
        if attackers.is_empty() {
            return;
        }

        let (mut left, mut right) = (false, false);
        let orth = self.colored_orth_sliders(!self.stm);
        let diag = self.colored_diag_sliders(!self.stm);
        let sliders = (bishop_rays(our_king) & diag) | (rook_rays(our_king) & orth);

        'attackers: for attacker in attackers {
            let blockers = self.occupied() ^ attacker ^ attacker_dest ^ victim;
            for slider in sliders {
                if (blockers & between(our_king, slider)).is_empty() {
                    continue 'attackers;
                }
            }

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
        self.mailbox[sq] = self.pieces[piece].has(sq).then_some(piece);

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

    #[inline]
    fn duck_bb(&self, src: Square, dest: Square, flag: MoveFlag) -> Bitboard {
        let mut empty_square_bb = !self.colors(Color::White) & !self.colors(Color::Black);

        match flag {
            MoveFlag::Capture | MoveFlag::DoublePush | MoveFlag::Normal => {
                empty_square_bb.0 |= 0 << src as usize;
                empty_square_bb.0 |= 1 << dest as usize;
            }
            _ => todo!(),
        }
        empty_square_bb
    }

    #[inline]
    pub fn get_legal_moves(&self) -> MoveList {
        let mut list = MoveList::default();
        let friendly_bb = self.colors(self.stm());
        let enemy_bb = self.colors(!self.stm());
        let filled_square_bb = enemy_bb | friendly_bb;
        let empty_square_bb = !filled_square_bb;

        /*
        Pawns
        */

        let friendly_pawns = self.pieces(Piece::Pawn) & friendly_bb;
        let pawns_forward_1 = match self.stm() {
            Color::Black => friendly_pawns.shift::<South>(1),
            Color::White => friendly_pawns.shift::<North>(1),
        };
        let pawns_forward_2 = match self.stm() {
            Color::Black => friendly_pawns.shift::<South>(2),
            Color::White => friendly_pawns.shift::<North>(2),
        };

        // Pawn Forward
        let valid_pawn_forward = pawns_forward_1 & empty_square_bb;
        valid_pawn_forward.iter().for_each(|dest| {
            let flag = MoveFlag::Normal;
            let src = match self.stm() {
                Color::White => dest.offset_dir::<North>(-1),
                Color::Black => dest.offset_dir::<South>(-1),
            };
            self.duck_bb(src, dest, flag).iter().for_each(|duck| {
                list.add(Move::new(src, dest, duck, flag));
            });
        });

        //Pawn Double
        let start_rank = Rank::Fourth.relative_to(self.stm()).bitboard();
        let valid_pawn_double = pawns_forward_2 & start_rank;
        valid_pawn_double.iter().for_each(|dest| {
            let flag = MoveFlag::DoublePush;
            let src = match self.stm() {
                Color::White => dest.offset_dir::<North>(-2),
                Color::Black => dest.offset_dir::<South>(-2),
            };
            self.duck_bb(src, dest, flag).iter().for_each(|duck| {
                list.add(Move::new(src, dest, duck, flag));
            });
        });

        //Pawn Attack Left
        let attack_left = pawns_forward_1.shift::<West>(1);
        let valid_attack_left = attack_left & filled_square_bb;
        valid_attack_left.iter().for_each(|dest| {
            let flag = MoveFlag::Capture;
            let src = match self.stm() {
                Color::Black => dest.offset(1, -1),
                Color::White => dest.offset(1, 1),
            };
            self.duck_bb(src, dest, flag).iter().for_each(|duck| {
                list.add(Move::new(src, dest, duck, flag));
            });
        });

        //Pawn Attack Right
        let attack_right = pawns_forward_1.shift::<East>(1);
        let valid_attack_right = attack_right & filled_square_bb;
        valid_attack_right.iter().for_each(|dest| {
            let flag = MoveFlag::Capture;
            let src = match self.stm() {
                Color::Black => dest.offset(-1, -1),
                Color::White => dest.offset(-1, 1),
            };
            self.duck_bb(src, dest, flag).iter().for_each(|duck| {
                list.add(Move::new(src, dest, duck, flag));
            });
        });

        //En Passant (work already done for us)
        if let Some(en_passant) = self.en_passant() {
            let flag = MoveFlag::EnPassant;
            let dest = Square::new(en_passant.file(), Rank::Sixth.relative_to(self.stm()));

            'left: {
                if !en_passant.left() {
                    break 'left;
                }
                let Some(left) = en_passant.file().try_offset(-1) else {
                    break 'left;
                };
                let src_left = Square::new(left, Rank::Fifth.relative_to(self.stm()));
                self.duck_bb(src_left, dest, flag).iter().for_each(|duck| {
                    list.add(Move::new(src_left, dest, duck, flag));
                });
            }

            'right: {
                if !en_passant.right() {
                    break 'right;
                }
                let Some(left) = en_passant.file().try_offset(1) else {
                    break 'right;
                };
                let src_left = Square::new(left, Rank::Fifth.relative_to(self.stm()));
                self.duck_bb(src_left, dest, flag).iter().for_each(|duck| {
                    list.add(Move::new(src_left, dest, duck, flag));
                });
            }
        }

        //rook

        list
    }
}

#[test]
fn perft_depth_1() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/4*3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("board couldnt parse fen string");
    assert_eq!(board.get_legal_moves().len(), 640);
}

#[test]
fn pawn_attack() {
    let board = Board::from_fen("4k3/8/8/2p1p3/3P4/8/8/3K4 w - - 0 1")
        .expect("board couldnt parse fen string");
    assert_eq!(board.get_legal_moves().len(), 179);
}

#[test]
fn en_passant() {
    let board = Board::from_fen("4k3/8/8/2p1p3/3P4/8/8/3K4 w - - 0 1")
        .expect("board couldnt parse fen string");
    assert_eq!(board.get_legal_moves().len(), 179);
}
