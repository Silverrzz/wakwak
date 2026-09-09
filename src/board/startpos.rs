use crate::board::{Board, CastlingDirection, sliders};
use crate::common::{Color, File, Piece, Rank, Square};

impl Board {
    #[inline]
    pub fn startpos() -> Board {
        Board::frc_startpos(518)
    }

    #[inline]
    pub fn frc_startpos(scharnagl: u16) -> Board {
        Board::dfrc_startpos(scharnagl, scharnagl)
    }

    #[inline]
    pub fn dfrc_startpos(white_scharnagl: u16, black_scharnagl: u16) -> Board {
        assert!(white_scharnagl < 960);
        assert!(black_scharnagl < 960);

        #[inline]
        fn write_scharnagl(board: &mut Board, color: Color, scharnagl: u16) {
            let n = scharnagl;
            let (n, light_bishop) = (n / 4, n % 4);
            let (n, dark_bishop) = (n / 4, n % 4);
            let (n, queen) = (n / 6, n % 6);
            let knights = n;

            let backrank = Rank::First.relative_to(color);
            let mut free_squares = backrank.bitboard();

            let light_bishop = match light_bishop {
                0 => File::B,
                1 => File::D,
                2 => File::F,
                3 => File::H,
                _ => unreachable!(),
            };
            let dark_bishop = match dark_bishop {
                0 => File::A,
                1 => File::C,
                2 => File::E,
                3 => File::G,
                _ => unreachable!(),
            };

            let light_bishop = Square::new(light_bishop, backrank);
            let dark_bishop = Square::new(dark_bishop, backrank);

            free_squares ^= light_bishop;
            free_squares ^= dark_bishop;

            let queen = free_squares.iter().nth(queen as usize).unwrap();
            free_squares ^= queen;

            let (left_knight, right_knight) = match knights {
                0 => (0, 1),
                1 => (0, 2),
                2 => (0, 3),
                3 => (0, 4),

                4 => (1, 2),
                5 => (1, 3),
                6 => (1, 4),

                7 => (2, 3),
                8 => (2, 4),

                9 => (3, 4),

                _ => unreachable!(),
            };

            let left_knight = free_squares.iter().nth(left_knight).unwrap();
            let right_knight = free_squares.iter().nth(right_knight).unwrap();

            free_squares ^= left_knight;
            free_squares ^= right_knight;

            let left_rook = free_squares.next();
            free_squares ^= left_rook;

            let king = free_squares.next();
            free_squares ^= king;

            let right_rook = free_squares.next();
            free_squares ^= right_rook;

            let pieces: [(Square, Piece); File::COUNT] = [
                (king, Piece::King),
                (left_knight, Piece::Knight),
                (right_knight, Piece::Knight),
                (light_bishop, Piece::Bishop),
                (dark_bishop, Piece::Bishop),
                (left_rook, Piece::Rook),
                (right_rook, Piece::Rook),
                (queen, Piece::Queen),
            ];

            for &(sq, piece) in &pieces {
                board.toggle_square(sq, piece, color);
                board.mailbox[sq] = Some(piece);
            }

            for sq in Rank::Second.relative_to(color).bitboard() {
                board.toggle_square(sq, Piece::Pawn, color);
                board.mailbox[sq] = Some(Piece::Pawn);
            }

            board.set_castling_rights(color, CastlingDirection::Short, Some(right_rook.file()));
            board.set_castling_rights(color, CastlingDirection::Long, Some(left_rook.file()));
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
            fmc: 1,
            hmc: 0,
            slider_tag: sliders::init(),
        };

        write_scharnagl(&mut board, Color::White, white_scharnagl);
        write_scharnagl(&mut board, Color::Black, black_scharnagl);
        board
    }
}
