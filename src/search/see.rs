use crate::board::Board;
use crate::common::{
    Bitboard, Color, Move, MoveFlag, Piece, Square, bishop_rays, king_attacks, knight_attacks,
    pawn_attacks, rook_rays,
};
use crate::search::Params;

const KING_VALUE: i32 = 10_000_000;

pub fn value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => Params::see_pawn_value(),
        Piece::Knight => Params::see_knight_value(),
        Piece::Bishop => Params::see_bishop_value(),
        Piece::Rook => Params::see_rook_value(),
        Piece::Queen => Params::see_queen_value(),
        Piece::King => KING_VALUE,
    }
}

pub fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    let flag = mv.flag();

    let (src, dest) = (mv.src(), mv.dest());

    let next_victim = flag
        .promotion()
        .unwrap_or_else(|| board.piece_on(src).unwrap());

    let mut balance = move_value(board, mv) - threshold;

    if balance < 0 {
        return false;
    }

    balance -= value(next_victim);

    if balance >= 0 {
        return true;
    }

    let mut attackers = attackers_to(board, dest) & !src.bitboard();
    let mut stm = !board.stm();

    loop {
        let our_attackers = attackers & board.colors(stm);
        if our_attackers.is_empty() {
            break;
        }

        let attacker_pc = least_valuable_attacker(board, our_attackers);
        let attacker_sq = board
            .colored_pieces(stm, attacker_pc)
            .try_next()
            .expect("No attacker found!");

        if attacker_pc == Piece::King && (attackers & board.colors(!stm)).is_nonempty() {
            break;
        }

        attackers ^= attacker_sq.bitboard();
        stm = !stm;

        balance = -balance - 1 - value(attacker_pc);
        if balance >= 0 {
            break;
        }
    }

    stm != board.stm()
}

fn move_value(board: &Board, mv: Move) -> i32 {
    let flag = mv.flag();

    let captured = if flag == MoveFlag::EnPassant {
        value(Piece::Pawn)
    } else {
        board.piece_on(mv.dest()).map_or(0, value)
    };

    let promotion = flag
        .promotion()
        .map_or(0, |piece| value(piece) - value(Piece::Pawn));

    captured + promotion
}

fn least_valuable_attacker(board: &Board, attackers: Bitboard) -> Piece {
    if !(attackers & board.pieces(Piece::Pawn)).is_empty() {
        return Piece::Pawn;
    }
    if !(attackers & board.pieces(Piece::Knight)).is_empty() {
        return Piece::Knight;
    }
    if !(attackers & board.pieces(Piece::Bishop)).is_empty() {
        return Piece::Bishop;
    }
    if !(attackers & board.pieces(Piece::Rook)).is_empty() {
        return Piece::Rook;
    }
    if !(attackers & board.pieces(Piece::Queen)).is_empty() {
        return Piece::Queen;
    }
    if !(attackers & board.pieces(Piece::King)).is_empty() {
        return Piece::King;
    }
    panic!("No attackers found");
}

fn attackers_to(board: &Board, square: Square) -> Bitboard {
    let king_attacks = king_attacks(square);
    let king_attackers = king_attacks & board.pieces(Piece::King);

    let w_pawn_attacks = pawn_attacks(square, Color::White);
    let b_pawn_attacks = pawn_attacks(square, Color::Black);

    let w_pawn_attackers = b_pawn_attacks & board.colored_pieces(Color::White, Piece::Pawn);
    let b_pawn_attackers = w_pawn_attacks & board.colored_pieces(Color::Black, Piece::Pawn);

    let knight_attacks = knight_attacks(square);
    let knight_attackers = knight_attacks & board.pieces(Piece::Knight);

    // Only consider directly adjacent sliders (since distant sliders can be blocked by duck)
    let diag_neighbours = king_attacks & bishop_rays(square);
    let diag_attackers = diag_neighbours & board.diag_sliders();

    let orth_neighbours = king_attacks & rook_rays(square);
    let orth_attackers = orth_neighbours & board.orth_sliders();

    w_pawn_attackers
        | b_pawn_attackers
        | knight_attackers
        | king_attackers
        | diag_attackers
        | orth_attackers
}
