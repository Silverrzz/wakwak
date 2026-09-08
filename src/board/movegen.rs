use crate::board::Board;
use crate::common::{
    Bitboard, Color, East, Move, MoveFlag, MoveList, North, Piece, Rank, South, Square, West,
    king_attacks, knight_attacks,
};

impl Board {
    #[inline]
    fn duck_moves(
        &self,
        mut empty_square_bb: Bitboard,
        src: Square,
        dest: Square,
        flag: MoveFlag,
    ) -> Bitboard {
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
    pub fn gen_moves(&self) -> MoveList {
        let mut list = MoveList::default();
        let friendly_bb = self.colors(self.stm());
        let enemy_bb = self.colors(!self.stm());
        let duck_bb = self.duck().map_or(Bitboard::EMPTY, Square::bitboard);
        let filled_square_bb = enemy_bb | friendly_bb | duck_bb;
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
            self.duck_moves(empty_square_bb, src, dest, flag)
                .iter()
                .for_each(|duck| {
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
            self.duck_moves(empty_square_bb, src, dest, flag)
                .iter()
                .for_each(|duck| {
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
            self.duck_moves(empty_square_bb, src, dest, flag)
                .iter()
                .for_each(|duck| {
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
            self.duck_moves(empty_square_bb, src, dest, flag)
                .iter()
                .for_each(|duck| {
                    list.add(Move::new(src, dest, duck, flag));
                });
        });

        //En Passant
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
                self.duck_moves(empty_square_bb, src_left, dest, flag)
                    .iter()
                    .for_each(|duck| {
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
                self.duck_moves(empty_square_bb, src_left, dest, flag)
                    .iter()
                    .for_each(|duck| {
                        list.add(Move::new(src_left, dest, duck, flag));
                    });
            }
        }

        /*
        Knights
        */

        // Knights cant end on a friendly piece or the duck, uses precomputed attacks (ty tecci)
        let knight_targets = !friendly_bb & !duck_bb;
        for src in self.pieces(Piece::Knight) & friendly_bb {
            // The compile-time lookup table already handles board edges.
            for dest in knight_attacks(src) & knight_targets {
                let flag = if enemy_bb.has(dest) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Normal
                };
                for duck in self.duck_moves(empty_square_bb, src, dest, flag) {
                    list.add(Move::new(src, dest, duck, flag));
                }
            }
        }

        /*
        King
        */
        let king_sq = (self.pieces(Piece::King) & friendly_bb).next();
        let king_attacks = king_attacks(king_sq);
        king_attacks.iter().for_each(|dest| {
            let flag = match enemy_bb.has(dest) {
                true => MoveFlag::Capture,
                false => MoveFlag::Normal,
            };
            self.duck_moves(empty_square_bb, king_sq, dest, flag)
                .iter()
                .for_each(|duck| {
                    list.add(Move::new(king_sq, dest, duck, flag));
                });
        });

        list
    }
}

#[test]
fn pawn_attack() {
    const EXPECTED_RESULT: usize = 8 * (64 - 6) + 2 * (64 - 5);
    let board = Board::from_fen("8/3k4/8/2p*p3/3P4/8/3K4/8 w - - 0 1")
        .expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}

#[test]
fn knight_corners() {
    const EXPECTED_RESULT: usize = 13 * (64 - 7);
    let board = Board::from_fen("N3k2N/8/8/4*3/8/8/8/N3K2N w - - 0 1")
        .expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}

#[test]
fn knight_captures_and_blockers() {
    const EXPECTED_RESULT: usize = 2 * (64 - 6) + 15 * (64 - 7);
    let board = Board::from_fen("7k/7n/4*3/1r3R2/3N4/8/8/K7 w - - 0 1")
        .expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}

#[test]
fn black_knight_captures_and_blockers() {
    const EXPECTED_RESULT: usize = 2 * (64 - 6) + 15 * (64 - 7);
    let board: Board = Board::from_fen("7K/7N/4*3/1R3r2/3n4/8/8/k7 b - - 0 1")
        .expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}

#[test]
fn king_empty() {
    const EXPECTED_RESULT: usize = 8 * (64 - 3);
    let board =
        Board::from_fen("8/4K3/8/4*3/8/8/4k3/8 b - - 0 1").expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}

#[test]
fn king_capture() {
    const EXPECTED_RESULT: usize = 8 * (64 - 10);
    let board = Board::from_fen("7*/8/8/3PPP2/3PkP2/3PPP2/8/K7 b - - 0 1")
        .expect("board couldnt parse fen string");
    board.display(true);
    assert_eq!(board.gen_moves().len(), EXPECTED_RESULT);
}
