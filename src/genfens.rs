use crate::board::{Board, SplitMix64};
use crate::engine::{Engine, EngineOptions};
use crate::position::Position;
use crate::search::SearchInfo;
use crate::uci::SearchLimit;
use crate::util::Abort;
use std::io::{self, Write};
use std::sync::atomic::Ordering;

const USAGE: &str = "genfens <count> seed <u64> book None [dfrc <bool>] [moves <n>]";

impl Engine {
    pub fn gen_fens(&mut self, args: &[String]) -> io::Result<()> {
        let end = args.len() - usize::from(args.last().is_some_and(|arg| arg == "quit"));
        let input = args[..end].join(" ");
        let tokens = input.split_whitespace().collect::<Vec<_>>();
        if matches!(tokens.as_slice(), ["genfens", "help" | "--help"]) {
            println!("info string Usage: {USAGE}");
            println!("info string Defaults: dfrc true, moves 8 (plus 0 or 1 random ply)");
            return Ok(());
        }
        let ["genfens", count, "seed", seed, "book", book, extra @ ..] = tokens.as_slice() else {
            return Err(io::Error::other(USAGE));
        };
        let count = count.parse::<usize>().map_err(io::Error::other)?;
        let seed = seed.parse::<u64>().map_err(io::Error::other)?;
        if !book.eq_ignore_ascii_case("none") {
            return Err(io::Error::other(
                "Opening books are not supported; use book None",
            ));
        }
        let (mut dfrc, mut plies) = (true, 8);
        for pair in extra.chunks(2) {
            match pair {
                ["dfrc", value] => dfrc = value.parse::<bool>().map_err(io::Error::other)?,
                ["moves", value] => plies = value.parse::<u16>().map_err(io::Error::other)?,
                _ => return Err(io::Error::other(USAGE)),
            }
        }
        let mut output = io::stdout();
        let mut rng = SplitMix64::new(seed);
        let options = EngineOptions {
            soft_target: true,
            frc: dfrc,
            ..self.options
        };
        self.searcher.newgame();
        for _ in 0..count {
            loop {
                let Some(position) = random_opening(&mut rng, dfrc, plies) else {
                    continue;
                };
                self.searcher.search(
                    position.clone(),
                    options,
                    vec![SearchLimit::Nodes(1000)],
                    SearchInfo::None,
                );
                self.searcher.wait();
                if self
                    .searcher
                    .shared
                    .best_score
                    .load(Ordering::Relaxed)
                    .abs()
                    >= 1000
                {
                    continue;
                }
                writeln!(
                    output,
                    "info string genfens {}",
                    position.board().to_fen(dfrc)
                )?;
                output.flush()?;
                break;
            }
        }
        Ok(())
    }
}

fn random_opening(rng: &mut SplitMix64, dfrc: bool, plies: u16) -> Option<Position> {
    let board = if dfrc {
        Board::dfrc_startpos((rng.next() % 960) as u16, (rng.next() % 960) as u16)
    } else {
        Board::startpos()
    };
    let mut position = Position::new(board);
    let mut moves = Vec::new();
    let plies = usize::from(plies) + (rng.next() % 2) as usize;
    for _ in 0..plies {
        if position.board().terminal_state().is_some() || position.repetition() {
            return None;
        }
        moves.clear();
        position.board().gen_all_moves(|group| {
            moves.extend(group);
            Abort::No
        });
        position.make_move(moves[(rng.next() % moves.len() as u64) as usize]);
    }
    if position.board().terminal_state().is_some() || position.repetition() {
        return None;
    }
    Some(position)
}
