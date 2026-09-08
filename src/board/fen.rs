use crate::board::{Board, CastlingDirection, ZOBRIST, sliders};
use crate::common::{Color, File, Piece, Rank, Square, between};
use std::fmt::Write;

impl Board {
    pub fn from_fen(fen: &str) -> Option<Self> {
        /*
        TODO: Return a String or custom Err result to see why the fen is invalid
        */
        let mut parts = fen.split_whitespace();

        let pieces = parts.next()?;
        let stm = parts.next()?;
        let castling_rights = parts.next()?;
        let en_passant = parts.next()?;
        let hmc = parts.next()?;
        let fmc = parts.next()?;

        if parts.next().is_some() {
            return None;
        }

        let mut board = Board {
            pieces: Default::default(),
            colors: Default::default(),
            mailbox: Default::default(),
            castling_rights: Default::default(),
            en_passant: None,
            duck: None,
            hash: 0,
            stm: Color::White,
            fmc: 0,
            hmc: 0,
            _slider_tag: sliders::init(),
        };

        //Parse board
        for (rank, row) in pieces.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank)?;
            let mut file = 0;

            for p in row.chars() {
                if let Some(empty) = p.to_digit(10) {
                    file += empty as usize;
                } else {
                    if p == '*' {
                        let sq = Square::new(File::try_index(file)?, rank);
                        if board.duck.is_some() {
                            return None;
                        }

                        board.set_duck(Some(sq));
                    } else {
                        let piece = p.try_into().ok()?;
                        let color = Color::index(p.is_ascii_lowercase() as usize);
                        let sq = Square::new(File::try_index(file)?, rank);

                        if board.mailbox[sq].is_some() {
                            return None;
                        }

                        board.toggle_square(sq, piece, color);
                    }

                    file += 1;
                }
            }

            if file != File::COUNT {
                return None;
            }
        }

        if board.colored_pieces(Color::White, Piece::King).popcnt() != 1
            && board.colored_pieces(Color::Black, Piece::King).popcnt() != 1
        {
            return None;
        }

        //Parse stm
        if stm.len() != 1 {
            return None;
        } else {
            board.stm = stm.chars().next().unwrap().try_into().ok()?;

            if board.stm == Color::Black {
                board.hash ^= ZOBRIST.stm;
            }
        }

        //Parse castling rights
        if castling_rights.len() > 4 {
            return None;
        }

        if castling_rights != "-" {
            for c in castling_rights.chars() {
                let color = Color::index(c.is_ascii_lowercase() as usize);
                let our_back_rank = Rank::First.relative_to(color);
                let our_king = board.king(color);

                if our_king.rank() != our_back_rank {
                    return None;
                }

                let rook_file = match c.to_ascii_lowercase() {
                    'a'..='h' => c.try_into().ok()?,
                    'k' => {
                        let corner_rook = Square::new(File::H, our_back_rank);
                        let rook_mask = between(our_king, corner_rook) | corner_rook;
                        let valid_rooks = board.colored_pieces(color, Piece::Rook) & rook_mask;

                        valid_rooks.try_next_back().map(Square::file)?
                    }
                    'q' => {
                        let corner_rook = Square::new(File::A, our_back_rank);
                        let rook_mask = between(our_king, corner_rook) | corner_rook;
                        let valid_rooks = board.colored_pieces(color, Piece::Rook) & rook_mask;

                        valid_rooks.try_next().map(Square::file)?
                    }
                    _ => return None,
                };

                let dir = if rook_file > our_king.file() {
                    CastlingDirection::Short
                } else {
                    CastlingDirection::Long
                };

                board.set_castling_rights(color, dir, Some(rook_file));
            }
        }

        //Parse en passant
        if en_passant != "-" {
            let ep_sq = en_passant.parse::<Square>().ok()?;
            if ep_sq.rank() != Rank::Sixth.relative_to(board.stm) {
                return None;
            }

            board.calc_en_passant(ep_sq.file());
        }

        //Parse halfmove clock and fullmove count
        board.hmc = hmc.parse::<u8>().ok()?.min(100);
        board.fmc = fmc.parse::<u16>().ok()?.max(1);

        Some(board)
    }

    #[inline]
    pub fn to_fen(&self, frc: bool) -> String {
        let mut fen = String::new();

        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;

            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 {
                        write!(fen, "{}", empty).unwrap();
                        empty = 0;
                    }

                    let mut piece: char = piece.into();
                    if self.color_on(sq).unwrap() == Color::White {
                        piece = piece.to_ascii_uppercase();
                    }

                    write!(fen, "{}", piece).unwrap();
                } else if self.duck == Some(sq) {
                    if empty > 0 {
                        write!(fen, "{}", empty).unwrap();
                        empty = 0;
                    }

                    write!(fen, "*").unwrap();
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                write!(fen, "{}", empty).unwrap();
            }

            if rank > Rank::First {
                write!(fen, "/").unwrap();
            }
        }

        write!(fen, " {}", char::from(self.stm)).unwrap();

        let mut castling_rights = String::new();
        if let Some(file) = self.castling_rights[Color::White].get(CastlingDirection::Short) {
            castling_rights.push(if frc {
                char::from(file).to_ascii_uppercase()
            } else {
                'K'
            });
        }
        if let Some(file) = self.castling_rights[Color::White].get(CastlingDirection::Long) {
            castling_rights.push(if frc {
                char::from(file).to_ascii_uppercase()
            } else {
                'Q'
            });
        }
        if let Some(file) = self.castling_rights[Color::Black].get(CastlingDirection::Short) {
            castling_rights.push(if frc {
                char::from(file).to_ascii_lowercase()
            } else {
                'k'
            });
        }
        if let Some(file) = self.castling_rights[Color::Black].get(CastlingDirection::Long) {
            castling_rights.push(if frc {
                char::from(file).to_ascii_lowercase()
            } else {
                'q'
            });
        }

        if castling_rights.is_empty() {
            castling_rights.push('-');
        }

        write!(fen, " {}", castling_rights).unwrap();

        if let Some(ep) = self.en_passant() {
            write!(
                fen,
                " {}",
                Square::new(ep.file(), Rank::Sixth.relative_to(self.stm))
            )
            .unwrap();
        } else {
            write!(fen, " -").unwrap();
        }

        write!(fen, " {} {}", self.hmc, self.fmc).unwrap();

        fen
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, CastlingRights};
    use crate::common::{Bitboard, Color, File, Piece, Square};

    #[test]
    fn from_fen() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/4*3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(board.is_some());

        let board = board.unwrap();
        assert_eq!(board.pieces(Piece::Pawn), Bitboard(0xFF00000000FF00));
        assert_eq!(board.pieces(Piece::Knight), Bitboard(0x4200000000000042));
        assert_eq!(board.pieces(Piece::Bishop), Bitboard(0x2400000000000024));
        assert_eq!(board.pieces(Piece::Rook), Bitboard(0x8100000000000081));
        assert_eq!(board.pieces(Piece::Queen), Bitboard(0x800000000000008));
        assert_eq!(board.pieces(Piece::King), Bitboard(0x1000000000000010));
        assert_eq!(board.colors(Color::White), Bitboard(0xFFFF));
        assert_eq!(board.colors(Color::Black), Bitboard(0xFFFF000000000000));
        assert_eq!(
            board.castling_rights(Color::White),
            CastlingRights::new(Some(File::A), Some(File::H))
        );
        assert_eq!(
            board.castling_rights(Color::Black),
            CastlingRights::new(Some(File::A), Some(File::H))
        );
        assert_eq!(board.en_passant(), None);
        assert_eq!(board.duck(), Some(Square::E4));
        assert_eq!(board.stm, Color::White);
        assert_eq!(board.fmc, 1);
        assert_eq!(board.hmc, 0);
    }

    #[test]
    fn to_fen() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/4*3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(board.is_some());

        assert_eq!(
            board.unwrap().to_fen(false),
            "rnbqkbnr/pppppppp/8/8/4*3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
