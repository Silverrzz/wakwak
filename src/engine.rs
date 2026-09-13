use crate::board::Board;
use crate::common::Move;
use crate::position::Position;
use crate::search::{DEFAULT_OVERHEAD, SearchInfo, Searcher};
use crate::uci::{SearchLimit, UciCommand, UciParseError};
use crate::util::{Abort, EPOCH};
use std::io;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Engine {
    pub position: Position,
    pub searcher: Searcher,
    pub options: EngineOptions,
}

impl Engine {
    #[inline]
    pub fn new() -> Self {
        Self {
            position: Position::new(Board::startpos()),
            searcher: Searcher::default(),
            options: EngineOptions::default(),
        }
    }

    #[inline]
    pub fn run(&mut self) {
        LazyLock::force(&EPOCH);

        let mut buffer = String::new();
        let args = std::env::args().skip(1).collect::<Vec<String>>();

        if !args.is_empty() {
            if args[0] == "perft" {
                self.handle_input(&args.join(" "));
                return;
            }

            for cmd in args {
                if self.handle_input(cmd.trim()) == Abort::Yes {
                    return;
                }
            }

            return;
        }

        loop {
            buffer.clear();
            match io::stdin().read_line(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if buffer.trim().is_empty() {
                continue;
            }

            if self.handle_input(buffer.trim()) == Abort::Yes {
                break;
            }
        }
    }

    #[inline]
    pub fn handle_input(&mut self, input: &str) -> Abort {
        let cmd = match UciCommand::parse(input, self.options.dumb_interface, self.options.frc) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("info string {e}");
                return Abort::No;
            }
        };

        match cmd {
            UciCommand::Uci => Self::uci(),
            UciCommand::NewGame => self.newgame(),
            UciCommand::IsReady => Self::isready(),
            UciCommand::Display => self.display(),
            UciCommand::Bench { depth } => self.bench(depth),
            UciCommand::Search(limits) => self.search(limits),
            UciCommand::Perft { depth, bulk } => self.perft(depth, bulk),
            UciCommand::SplitPerft { depth, bulk } => self.split_perft(depth, bulk),
            UciCommand::Position { board, moves } => self.set_position(board, moves),
            UciCommand::SetOption { name, value } => self.set_option(name, value),
            UciCommand::Stop => self.stop(),
            UciCommand::Quit => return self.quit(),
        }

        Abort::No
    }

    #[inline]
    fn uci() {
        println!("id name wakwak v{ENGINE_VERSION}");
        println!("id author Drexell, Kelseyde, ptsouchlos, Silverrzz, Sp00ph and Tecci");
        println!("option name Threads type spin default 1 min 1 max 1024");
        println!("option name MoveOverhead type spin default {DEFAULT_OVERHEAD} min 0 max 5000");
        println!("option name Minimal type check default false");
        println!("option name SoftTarget type check default false");
        println!("option name UseDumbInterface type check default true");
        println!("option name UCI_Chess960 type check default false");
        println!("option name UCI_Variant type combo default duck var duck");
        println!("uciok");
    }

    #[inline]
    fn newgame(&mut self) {
        self.searcher.newgame();
    }

    #[inline]
    fn isready() {
        println!("readyok");
    }

    #[inline]
    fn display(&self) {
        self.position.board().display(self.options.frc);
    }

    #[inline]
    fn search(&mut self, limits: Vec<SearchLimit>) {
        self.searcher.search(
            self.position.clone(),
            self.options,
            limits,
            if self.options.minimal {
                SearchInfo::Minimal
            } else {
                SearchInfo::Full
            },
        );
    }

    #[inline]
    fn perft(&self, depth: u8, bulk: bool) {
        let start = Instant::now();
        let nodes = if bulk {
            self.position.board().perft::<true>(depth)
        } else {
            self.position.board().perft::<false>(depth)
        };
        let elapsed = start.elapsed();
        let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;
        println!(
            "info string perft depth {depth} nodes {nodes} time {} nps {nps}",
            elapsed.as_millis()
        );
    }

    #[inline]
    fn split_perft(&self, depth: u8, bulk: bool) {
        if depth == 0 {
            eprintln!("info string `splitperft` depth cannot be 0");
            return;
        }

        let mut perft_data = Vec::new();
        let mut total_time = Duration::ZERO;
        let mut total_nodes = 0;

        self.position.board().gen_moves(|moves| {
            for mv in moves {
                let mut board = *self.position.board();
                board.make_move(mv);

                let start = Instant::now();
                let nodes = if bulk {
                    board.perft::<true>(depth - 1)
                } else {
                    board.perft::<false>(depth - 1)
                };

                total_time += start.elapsed();
                total_nodes += nodes;

                perft_data.push((mv, nodes));
            }

            Abort::No
        });

        for (mv, nodes) in perft_data {
            println!(
                "{:<5}: {nodes}",
                mv.display(self.options.dumb_interface, self.options.frc)
            );
        }

        let nps = (total_nodes as f64 / total_time.as_secs_f64()) as u64;
        println!(
            "info string splitperft depth {depth} nodes {total_nodes} time {} nps {nps}",
            total_time.as_millis()
        );
    }

    #[inline]
    fn set_position(&mut self, board: Board, moves: Vec<Move>) {
        self.position.reset(board);
        for mv in moves {
            self.position.make_move(mv);
        }
    }

    #[inline]
    fn set_option(&mut self, name: String, value: String) {
        match name.as_str() {
            "MoveOverhead" => {
                let value = match value.parse::<u64>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("info string {:?}", UciParseError::InvalidInteger(e));
                        return;
                    }
                };

                self.options.overhead = value;
                println!("info string Set MoveOverhead to {value}");
            }
            "Minimal" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.minimal = value;
                println!("info string Set Minimal to {value}");
            }
            "SoftTarget" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.soft_target = value;
                println!("info string Set SoftTarget to {value}");
            }
            "UseDumbInterface" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.dumb_interface = value;
                println!("info string Set UseDumbInterface to {value}");
            }
            "UCI_Chess960" => {
                let value = match value.parse::<bool>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("info string {:?}", UciParseError::InvalidBoolean(e));
                        return;
                    }
                };

                self.options.frc = value;
                println!("info string Set UCI_Chess960 to {value}");
            }
            "UCI_Variant" => {
                let variant = match value.as_str() {
                    "duck" => Variant::Duck,
                    _ => {
                        eprintln!("info string Invalid UCI_Variant: `{value}` (expected duck)");
                        return;
                    }
                };

                self.options.variant = variant;
                println!("info string Set UCI_Variant to {value}");
            }
            _ => eprintln!("info string Unknown Option: `{name}`"),
        }
    }

    #[inline]
    fn stop(&mut self) {
        self.searcher.stop();
    }

    #[inline]
    fn quit(&mut self) -> Abort {
        self.searcher.quit();
        Abort::Yes
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EngineOptions {
    pub overhead: u64,
    pub minimal: bool,
    pub soft_target: bool,
    pub dumb_interface: bool,
    pub frc: bool,
    pub variant: Variant,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Variant {
    Duck,
}

impl Default for EngineOptions {
    #[inline]
    fn default() -> Self {
        Self {
            overhead: DEFAULT_OVERHEAD,
            minimal: false,
            soft_target: false,
            dumb_interface: true,
            frc: false,
            variant: Variant::Duck,
        }
    }
}
