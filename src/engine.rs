use crate::board::Board;
use crate::common::Move;
use crate::position::Position;
use crate::uci::{UciCommand, UciParseError};
use std::io;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Engine {
    pub position: Position,
    pub options: EngineOptions,
}

impl Engine {
    #[inline]
    pub fn new() -> Self {
        Self {
            position: Position::new(Board::startpos()),
            options: EngineOptions::default(),
        }
    }

    #[inline]
    pub fn run(&mut self) {
        let mut buffer = String::new();
        let args = std::env::args().skip(1).collect::<Vec<String>>();

        if !args.is_empty() {
            for cmd in args {
                if self.handle_input(cmd.trim()) == Abort::Yes {
                    return;
                }
            }

            return;
        }

        while io::stdin().read_line(&mut buffer).is_ok() {
            if buffer.trim().is_empty() {
                continue;
            }

            if self.handle_input(buffer.trim()) == Abort::Yes {
                break;
            }

            buffer.clear();
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
        println!("option name UseDumbInterface type check default true");
        println!("option name UCI_Chess960 type check default false");
        println!("option name UCI_Variant type combo default duck var duck");
        println!("uciok");
    }

    #[inline]
    fn newgame(&mut self) {}

    #[inline]
    fn isready() {
        println!("readyok");
    }

    #[inline]
    fn display(&self) {
        self.position.board().display(self.options.frc);
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
    fn stop(&mut self) {}

    #[inline]
    fn quit(&mut self) -> Abort {
        Abort::Yes
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Abort {
    Yes,
    No,
}

#[derive(Debug, Copy, Clone)]
pub struct EngineOptions {
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
            dumb_interface: true,
            frc: false,
            variant: Variant::Duck,
        }
    }
}
