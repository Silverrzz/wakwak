use crate::{board::Board, common::{Bitboard, Color, East, Move, MoveFlag, North, Piece, Rank, South, Square, West, bitboard}};

//lazy upper bound to include duck, could be trimmed
const MAX_MOVES: usize = 218 * Square::COUNT; 

pub struct MoveList {
    list: [Option<Move>; MAX_MOVES], 
    len: usize
}

impl MoveList {
    fn empty() -> MoveList {
        MoveList { list: [None; MAX_MOVES], len: 0 }
    }
    fn add(&mut self, mv: Move) {
        self.list[self.len] = Some(mv);
        self.len += 1;
    }
}

#[inline]
fn duck_bb(mut empty_square_bb: Bitboard, src: Square, dest: Square, flag: MoveFlag) -> Bitboard {
    match flag {
        MoveFlag::Capture |
        MoveFlag::Normal => {
            empty_square_bb.0 |= 0 << src as usize;
            empty_square_bb.0 |= 1 << dest as usize;
        }
        _ => todo!()
    }
    empty_square_bb
}

pub fn get_legal_moves(board: &Board) -> Move {
    let mut list = MoveList::empty();
    let friendly_bb = board.colors(board.stm());
    let enemy_bb = board.colors(!board.stm());
    let filled_square_bb = enemy_bb | friendly_bb;
    let empty_square_bb = !filled_square_bb;
    let friendly_pawns = board.pieces(Piece::Pawn) & friendly_bb;
    let enemy_pawns = board.pieces(Piece::Pawn) & enemy_bb;

    /*
        Pawns
    */

    // Pawn Forward
    let shifted_pawns = match board.stm() {
        Color::Black => friendly_pawns.shift::<South>(1),
        Color::White => friendly_pawns.shift::<North>(1),
    };
    let valid_pawn_forward = shifted_pawns & empty_square_bb;
    valid_pawn_forward.iter().for_each(|dest|{
        let flag = MoveFlag::Normal;
        let src = match board.stm() {
            Color::Black => dest.offset(0, -1),
            Color::White => dest.offset(0,  1),
        };
        duck_bb(empty_square_bb, src, dest, flag).iter().for_each(|duck|{
            list.add(Move::new(src, dest, duck, flag));
        });
    });

    //Pawn Attack Left
    let attack_left  = shifted_pawns.shift::<West>(1);
    let valid_attack_left = attack_left & filled_square_bb;
    valid_attack_left.iter().for_each(|dest|{
        let flag = MoveFlag::Capture;
        let src = match board.stm() {
            Color::Black => dest.offset(1, -1),
            Color::White => dest.offset(1,  1),
        };
        duck_bb(empty_square_bb, src, dest, flag).iter().for_each(|duck|{
            list.add(Move::new(src, dest, duck, flag));
        });
    });

    //Pawn Attack Right
    let attack_right = shifted_pawns.shift::<East>(1);
    let valid_attack_right = attack_right & filled_square_bb;
    valid_attack_right.iter().for_each(|dest|{
        let flag = MoveFlag::Capture;
        let src = match board.stm() {
            Color::Black => dest.offset(-1, -1),
            Color::White => dest.offset(-1,  1),
        };
        duck_bb(empty_square_bb, src, dest, flag).iter().for_each(|duck|{
            list.add(Move::new(src, dest, duck, flag));
        });
    });

    //En Crossiant (work already done for us)
    if let Some(en_passant) = board.en_passant() {
        let flag = MoveFlag::EnPassant;
        let dest = Square::new(
            en_passant.file(), 
            Rank::Sixth.relative_to(board.stm())
        );

        'left: {
            if !en_passant.left() {break 'left;}
            let Some(left) = en_passant.file().try_offset(-1) else {break 'left;};
            let src_left = Square::new(
                left, 
                Rank::Fifth.relative_to(board.stm())
            );
            duck_bb(empty_square_bb, src_left, dest, flag).iter().for_each(|duck|{
                list.add(Move::new(src_left, dest, duck, flag));
            });
        }

        'right: {
            if !en_passant.right() {break 'right;}
            let Some(left) = en_passant.file().try_offset(1) else {break 'right;};
            let src_left = Square::new(
                left, 
                Rank::Fifth.relative_to(board.stm())
            );
            duck_bb(empty_square_bb, src_left, dest, flag).iter().for_each(|duck|{
                list.add(Move::new(src_left, dest, duck, flag));
            });
        }
    }

    todo!("pawns only movegen (incomplete)")
}