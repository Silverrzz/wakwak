use crate::board::Board;
use crate::common::Move;
use std::num::ParseIntError;
use std::str::{FromStr, ParseBoolError, SplitWhitespace};

#[derive(Clone)]
pub enum UciCommand {
    Uci,
    NewGame,
    IsReady,
    Display,
    Search(Vec<SearchLimit>),
    Perft { depth: u8, bulk: bool },
    SplitPerft { depth: u8, bulk: bool },
    Position { board: Board, moves: Vec<Move> },
    SetOption { name: String, value: String },
    Stop,
    Quit,
}

impl UciCommand {
    #[inline]
    pub fn parse(
        input: &str,
        dumb_interface: bool,
        frc: bool,
    ) -> Result<UciCommand, UciParseError> {
        use UciCommand::*;
        use UciParseError::*;

        let mut reader = input.split_whitespace();
        let cmd = reader.next().ok_or(MissingCommand)?;

        match cmd {
            "uci" => Ok(Uci),
            "ucinewgame" => Ok(NewGame),
            "isready" => Ok(IsReady),
            "display" | "d" => Ok(Display),
            "perft" => {
                let depth = reader.next().ok_or(MissingPerftDepth)?.parse()?;
                let bulk = reader.next().ok_or(MissingPerftBulk)?.parse()?;

                if let Some(token) = reader.next() {
                    return Err(UnexpectedPerftArgument(token.to_string()));
                }

                Ok(Perft { depth, bulk })
            }
            "splitperft" => {
                let depth = reader.next().ok_or(MissingSplitPerftDepth)?.parse()?;
                let bulk = reader.next().ok_or(MissingSplitPerftBulk)?.parse()?;

                if let Some(token) = reader.next() {
                    return Err(UnexpectedSplitPerftArgument(token.to_string()));
                }

                Ok(Perft { depth, bulk })
            }
            "stop" => Ok(Stop),
            "quit" | "q" => Ok(Quit),
            "go" => parse_search_cmd(reader),
            "position" | "pos" => parse_position_cmd(reader, dumb_interface, frc),
            "setoption" => {
                if reader.next() != Some("name") {
                    return Err(MissingOptionNameToken);
                }

                let name = reader.next().ok_or(MissingOptionName)?.to_string();
                if reader.next() != Some("value") {
                    return Err(MissingOptionValueToken);
                }

                let value = reader.next().ok_or(MissingOptionValue)?.to_string();
                Ok(SetOption { name, value })
            }
            _ => Err(UnknownCommand(cmd.to_string())),
        }
    }
}

fn parse_search_cmd(mut reader: SplitWhitespace) -> Result<UciCommand, UciParseError> {
    use SearchLimit::*;
    use UciCommand::*;
    use UciParseError::*;

    #[inline]
    fn parse_int<T: FromStr<Err = ParseIntError>>(
        reader: &mut SplitWhitespace,
        token: &str,
    ) -> Result<T, UciParseError> {
        Ok(reader
            .next()
            .ok_or_else(|| MissingLimitValue(token.to_string()))?
            .parse::<T>()?)
    }

    let mut limits = Vec::new();
    while let Some(token) = reader.next() {
        match token {
            "infinite" => {}
            "wtime" => limits.push(WhiteTime(
                parse_int::<i64>(&mut reader, token)?.max(0) as u64
            )),
            "btime" => limits.push(BlackTime(
                parse_int::<i64>(&mut reader, token)?.max(0) as u64
            )),
            "winc" => limits.push(WhiteInc(parse_int(&mut reader, token)?)),
            "binc" => limits.push(BlackInc(parse_int(&mut reader, token)?)),
            "movetime" => limits.push(MoveTime(parse_int(&mut reader, token)?)),
            "nodes" => limits.push(Nodes(parse_int(&mut reader, token)?)),
            "depth" => limits.push(Depth(parse_int(&mut reader, token)?)),
            _ => return Err(UnknownLimit(token.to_string())),
        }
    }

    Ok(Search(limits))
}

fn parse_position_cmd(
    mut reader: SplitWhitespace,
    dumb_interface: bool,
    frc: bool,
) -> Result<UciCommand, UciParseError> {
    use UciCommand::*;
    use UciParseError::*;

    let startpos = match reader.next() {
        Some("startpos") => Board::startpos(),
        Some("frc") => {
            if !frc {
                return Err(FrcNotEnabled);
            }

            let scharnagl: u16 = reader.next().ok_or(MissingScharnagl)?.parse()?;
            if scharnagl >= 960 {
                return Err(InvalidScharnagl(scharnagl));
            }

            Board::frc_startpos(scharnagl)
        }
        Some("dfrc") => {
            if !frc {
                return Err(FrcNotEnabled);
            }

            let white_scharnagl: u16 = reader.next().ok_or(MissingScharnagl)?.parse()?;
            let black_scharnagl: u16 = reader.next().ok_or(MissingScharnagl)?.parse()?;
            let max = white_scharnagl.max(black_scharnagl);

            if max >= 960 {
                return Err(InvalidScharnagl(max));
            }

            Board::dfrc_startpos(white_scharnagl, black_scharnagl)
        }
        Some("fen") => {
            let mut fen = String::new();

            for part in reader.by_ref().take(6) {
                if !fen.is_empty() {
                    fen.push(' ');
                }

                fen.push_str(part);
            }

            Board::from_fen(&fen).ok_or(InvalidFen(fen))?
        }
        _ => return Err(MissingPositionType),
    };

    if reader.next().is_some_and(|token| token != "moves") {
        return Err(MissingPositionMovesToken);
    }

    let mut current = startpos;
    let mut moves = Vec::new();

    for token in reader {
        let mv = Move::parse(&current, dumb_interface, token.trim())
            .ok_or_else(|| InvalidMove(token.to_string()))?;

        if !current.gen_moves().contains(&mv) {
            return Err(InvalidMove(token.to_string()));
        }

        moves.push(mv);
        current.make_move(mv);
    }

    Ok(Position {
        board: startpos,
        moves,
    })
}

#[derive(Debug, Copy, Clone)]
pub enum SearchLimit {
    WhiteTime(u64),
    BlackTime(u64),
    WhiteInc(u64),
    BlackInc(u64),
    MoveTime(u64),
    Nodes(u64),
    Depth(u8),
}

#[derive(thiserror::Error, Debug)]
pub enum UciParseError {
    #[error("Missing command")]
    MissingCommand,
    #[error("Unknown command: `{0}`")]
    UnknownCommand(String),

    #[error("Missing perft depth (usage: perft <depth> <bulk: true|false>)")]
    MissingPerftDepth,
    #[error("Missing perft bulk (usage: perft <depth> <bulk: true|false>)")]
    MissingPerftBulk,
    #[error("Unexpected perft argument: `{0}` (usage: perft <depth> <bulk: true|false>)")]
    UnexpectedPerftArgument(String),

    #[error("Missing perft depth (usage: splitperft <depth> <bulk: true|false>)")]
    MissingSplitPerftDepth,
    #[error("Missing perft bulk (usage: splitperft <depth> <bulk: true|false>)")]
    MissingSplitPerftBulk,
    #[error("Unexpected splitperft argument: `{0}` (usage: splitperft <depth> <bulk: true|false>)")]
    UnexpectedSplitPerftArgument(String),

    #[error("FRC not enabled in `position frc/dfrc` command")]
    FrcNotEnabled,
    #[error("Missing Scharnagl number in `position frc/dfrc` command")]
    MissingScharnagl,
    #[error("Invalid Scharnagl number in `position frc/dfrc` command: `{0}`")]
    InvalidScharnagl(u16),

    #[error("Missing position type (e.g. startpos, fen) in `position` command")]
    MissingPositionType,
    #[error("Invalid FEN in `position fen` command: `{0}`")]
    InvalidFen(String),

    #[error("Unknown search limit: `{0}`")]
    UnknownLimit(String),
    #[error("Missing value for search limit: `{0}`")]
    MissingLimitValue(String),

    #[error("Missing `moves` token in `position` command")]
    MissingPositionMovesToken,
    #[error("Invalid move in `position``command: `{0}`")]
    InvalidMove(String),

    #[error("Missing `name` token in `setoption` command")]
    MissingOptionNameToken,
    #[error("Missing `value` token in `setoption` command")]
    MissingOptionValueToken,
    #[error("Missing option name in `setoption` command")]
    MissingOptionName,
    #[error("Missing option value in `setoption` command")]
    MissingOptionValue,

    #[error("Error parsing integer: `{0}`")]
    InvalidInteger(#[from] ParseIntError),
    #[error("Error parsing boolean: `{0}`")]
    InvalidBoolean(#[from] ParseBoolError),
}
