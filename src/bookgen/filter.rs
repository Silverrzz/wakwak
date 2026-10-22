use super::Analysis;
use super::config::Range;
use crate::board::Board;
use crate::common::{Bitboard, Color, Piece, Square};
use serde::Deserialize;
use std::collections::BTreeMap;

const PIECES: [(Piece, &str, usize); 6] = [
    (Piece::Pawn, "pawn", 1),
    (Piece::Knight, "knight", 3),
    (Piece::Bishop, "bishop", 3),
    (Piece::Rook, "rook", 5),
    (Piece::Queen, "queen", 9),
    (Piece::King, "king", 0),
];

#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Filter {
    pub cp: Option<Range<f64>>,
    pub abs_cp: Option<Range<f64>>,
    pub ranges: BTreeMap<String, Range<f64>>,
    pub squares: BTreeMap<String, String>,
    pub side_to_move: Option<String>,
}

fn color_name(color: Color) -> &'static str {
    if color == Color::White {
        "white"
    } else {
        "black"
    }
}

fn occupant(board: &Board, square: Square) -> String {
    if board.duck() == Some(square) {
        return "duck".into();
    }
    match (board.color_on(square), board.piece_on(square)) {
        (Some(color), Some(piece)) => {
            let name = PIECES.iter().find(|&&(p, _, _)| p == piece).unwrap().1;
            format!("{}_{name}", color_name(color))
        }
        _ => "empty".into(),
    }
}

fn value(board: &Board, analysis: Option<&Analysis>, name: &str) -> Option<f64> {
    let material = |color| {
        PIECES
            .iter()
            .map(|&(piece, _, weight)| board.colored_pieces(color, piece).popcnt() * weight)
            .sum::<usize>() as f64
    };
    let sign = f64::from(board.stm().signum());
    Some(match name {
        "cp" => analysis?.cp(0)? as f64 * sign,
        "stm_cp" => analysis?.cp(0)? as f64,
        "abs_cp" => analysis?.cp(0)?.abs() as f64,
        "gap_cp" => analysis?.gap()? as f64,
        "depth" => analysis?.roots[0].searched_depth as f64,
        "mate_plies" => analysis?.roots[0].score.mate_in()? as f64,
        "win_rate" | "draw_rate" | "loss_rate" => {
            let index = match name {
                "draw_rate" => 1,
                "win_rate" => usize::from(sign < 0.0) * 2,
                _ => usize::from(sign > 0.0) * 2,
            };
            analysis?.roots[0].score.wdl(board)[index] as f64 / 1000.0
        }
        "material" => material(Color::White) + material(Color::Black),
        "material_difference" => material(Color::White) - material(Color::Black),
        "abs_material_difference" => (material(Color::White) - material(Color::Black)).abs(),
        "halfmove_clock" => board.hmc() as f64,
        "fullmove" => board.fmc() as f64,
        "pieces" => board.occupied_duckless().popcnt() as f64,
        "mobility" | "piece_mobility" => {
            let mut count = 0;
            board.gen_all_moves(|moves| {
                count += if name == "mobility" {
                    moves.len()
                } else {
                    usize::from(moves.is_nonempty())
                };
                crate::util::Abort::No
            });
            count as f64
        }
        _ => {
            let (color, property) = name.split_once('_')?;
            let color = match color {
                "white" => Color::White,
                "black" => Color::Black,
                _ => return None,
            };
            match property {
                "material" => material(color),
                "king_exposed" => {
                    let blocks = board.king_capture_blocks(color);
                    u8::from(
                        blocks != Bitboard::FULL && board.duck().is_none_or(|sq| !blocks.has(sq)),
                    ) as f64
                }
                _ => {
                    let piece = PIECES
                        .iter()
                        .find(|&&(_, name, _)| Some(name) == property.strip_suffix('s'))?
                        .0;
                    board.colored_pieces(color, piece).popcnt() as f64
                }
            }
        }
    })
}

impl Filter {
    pub fn matches(&self, board: &Board, analysis: Option<&Analysis>) -> bool {
        let ranges = self
            .ranges
            .iter()
            .map(|(name, range)| (name.as_str(), range))
            .chain(self.cp.as_ref().map(|range| ("cp", range)))
            .chain(self.abs_cp.as_ref().map(|range| ("abs_cp", range)));
        ranges.into_iter().all(|(name, range)| {
            value(board, analysis, name).map_or(analysis.is_none(), |value| range.contains(value))
        }) && self
            .side_to_move
            .as_ref()
            .is_none_or(|side| side == color_name(board.stm()))
            && self.squares.iter().all(|(square, piece)| {
                square
                    .parse()
                    .is_ok_and(|square| occupant(board, square) == *piece)
            })
    }
}
