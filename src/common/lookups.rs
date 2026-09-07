use crate::common::{
    Bitboard, Color, East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, Square, West,
};

#[inline]
pub const fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    const fn calc_attacks(sq: Square, color: Color) -> Bitboard {
        let bb = sq.bitboard();

        match color {
            Color::White => Bitboard(bb.shift::<NorthEast>(1).0 | bb.shift::<NorthWest>(1).0),
            Color::Black => Bitboard(bb.shift::<SouthEast>(1).0 | bb.shift::<SouthWest>(1).0),
        }
    }

    static PAWN_ATTACKS: [[Bitboard; Square::COUNT]; Color::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Color::COUNT];
        let mut i = 0;
        while i < Color::COUNT {
            let color = Color::index(i);
            let mut j = 0;
            while j < Square::COUNT {
                table[i][j] = calc_attacks(Square::index(j), color);
                j += 1;
            }

            i += 1;
        }

        table
    };

    PAWN_ATTACKS[color as usize][sq as usize]
}

#[inline]
pub const fn knight_attacks(sq: Square) -> Bitboard {
    const fn calc_attacks(sq: Square) -> Bitboard {
        const DELTAS: [(isize, isize); 8] = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];

        let mut bb = Bitboard::EMPTY;
        let mut i = 0;

        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];

            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 |= mv.bitboard().0;
            }

            i += 1;
        }

        bb
    }

    static KNIGHT_ATTACKS: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_attacks(Square::index(i));
            i += 1;
        }

        table
    };

    KNIGHT_ATTACKS[sq as usize]
}

#[inline]
pub const fn bishop_rays(sq: Square) -> Bitboard {
    const fn calc_rays(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<NorthWest>().0;
        bb.0 |= sq.smear::<NorthEast>().0;
        bb.0 |= sq.smear::<SouthWest>().0;
        bb.0 |= sq.smear::<SouthEast>().0;

        bb
    }

    static BISHOP_RAYS: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_rays(Square::index(i));
            i += 1;
        }

        table
    };

    BISHOP_RAYS[sq as usize]
}

#[inline]
pub const fn rook_rays(sq: Square) -> Bitboard {
    const fn calc_rays(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let sq = sq.bitboard();

        bb.0 |= sq.smear::<North>().0;
        bb.0 |= sq.smear::<South>().0;
        bb.0 |= sq.smear::<West>().0;
        bb.0 |= sq.smear::<East>().0;

        bb
    }

    static ROOK_RAYS: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_rays(Square::index(i));
            i += 1;
        }

        table
    };

    ROOK_RAYS[sq as usize]
}

#[inline]
pub const fn king_attacks(sq: Square) -> Bitboard {
    const fn calc_attacks(sq: Square) -> Bitboard {
        const DELTAS: [(isize, isize); 8] = [
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, 1),
        ];

        let mut bb = Bitboard::EMPTY;
        let mut i = 0;

        while i < DELTAS.len() {
            let (dx, dy) = DELTAS[i];

            if let Some(mv) = sq.try_offset(dx, dy) {
                bb.0 |= mv.bitboard().0;
            }

            i += 1;
        }

        bb
    }

    static KING_ATTACKS: [Bitboard; Square::COUNT] = {
        let mut table = [Bitboard::EMPTY; Square::COUNT];
        let mut i = 0;
        while i < Square::COUNT {
            table[i] = calc_attacks(Square::index(i));
            i += 1;
        }

        table
    };

    KING_ATTACKS[sq as usize]
}

#[inline]
pub const fn between(a: Square, b: Square) -> Bitboard {
    const fn calc_between(src: Square, dest: Square) -> Bitboard {
        let dx = dest.file() as isize - src.file() as isize;
        let dy = dest.rank() as isize - src.rank() as isize;

        let diag = dx.abs() == dy.abs();
        let orth = dx == 0 || dy == 0;

        if !(diag ^ orth) {
            return Bitboard::EMPTY;
        }

        let (dx, dy) = (dx.signum(), dy.signum());

        let mut bb = Bitboard::EMPTY;
        let mut sq = src.offset(dx, dy);

        while sq as u8 != dest as u8 {
            bb.0 |= sq.bitboard().0;
            sq = sq.offset(dx, dy)
        }

        bb
    }

    static BETWEEN: [[Bitboard; Square::COUNT]; Square::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
        let mut sq1 = 0;
        while sq1 < Square::COUNT {
            let mut sq2 = 0;
            while sq2 < Square::COUNT {
                table[sq2][sq1] = calc_between(Square::index(sq1), Square::index(sq2));
                sq2 += 1;
            }
            sq1 += 1;
        }

        table
    };

    BETWEEN[a as usize][b as usize]
}

#[inline]
pub const fn line(a: Square, b: Square) -> Bitboard {
    const fn calc_line(src: Square, dest: Square) -> Bitboard {
        let dx = dest.file() as isize - src.file() as isize;
        let dy = dest.rank() as isize - src.rank() as isize;

        let diag = dx.abs() == dy.abs();
        let orth = dx == 0 || dy == 0;

        if !(diag ^ orth) {
            return Bitboard::EMPTY;
        }

        let (dx, dy) = (dx.signum(), dy.signum());

        let mut bb = Bitboard::EMPTY;
        let mut next = src.try_offset(dx, dy);
        while let Some(sq) = next {
            bb.0 |= sq.bitboard().0;
            next = sq.try_offset(dx, dy);
        }

        bb
    }

    static LINE: [[Bitboard; Square::COUNT]; Square::COUNT] = {
        let mut table = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
        let mut sq1 = 0;
        while sq1 < Square::COUNT {
            let mut sq2 = 0;
            while sq2 < Square::COUNT {
                table[sq2][sq1] = calc_line(Square::index(sq1), Square::index(sq2));
                sq2 += 1;
            }
            sq1 += 1;
        }

        table
    };

    LINE[a as usize][b as usize]
}
