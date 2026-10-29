use crate::bench::DEFAULT_BENCH_DEPTH;
use crate::board::Board;
use crate::common::Move;
use std::num::ParseIntError;
use std::str::{FromStr, ParseBoolError, SplitWhitespace};

mod move_pattern;
use move_pattern::MovePattern;

pub const GENFENS_USAGE: &str = "genfens <count> seed <u64> book None [dfrc <bool>] [moves <n>]";

#[derive(Clone)]
pub enum UciCommand {
    Uci,
    NewGame,
    IsReady,
    Display,
    Eval,
    Bench {
        depth: u8,
    },
    GenFens {
        count: usize,
        seed: u64,
        dfrc: bool,
        plies: u16,
    },
    GenFensHelp,
    #[cfg(not(target_os = "emscripten"))]
    BookGen(Vec<String>),
    Relabel {
        input: String,
        output: String,
        nodes: u64,
        threads: usize,
    },
    Search {
        limits: Vec<SearchLimit>,
        root_moves: Option<Vec<Move>>,
    },
    Perft {
        depth: u8,
        bulk: bool,
    },
    SplitPerft {
        depth: u8,
        bulk: bool,
    },
    Position {
        board: Board,
        moves: Vec<Move>,
    },
    SetOption {
        name: String,
        value: String,
    },
    Stop,
    Wait,
    Quit,
}

impl UciCommand {
    #[inline]
    pub fn parse(
        input: &str,
        board: &Board,
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
            "eval" => Ok(Eval),
            "genfens" => parse_genfens_cmd(reader),
            #[cfg(not(target_os = "emscripten"))]
            "bookgen" => Ok(BookGen(reader.map(str::to_owned).collect())),
            "relabel" => parse_relabel_cmd(reader),
            "bench" => {
                let depth = reader.next().map_or(Ok(DEFAULT_BENCH_DEPTH), str::parse)?;

                Ok(Bench { depth })
            }
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

                Ok(SplitPerft { depth, bulk })
            }
            "stop" => Ok(Stop),
            "quit" | "q" => Ok(Quit),
            "wait" => Ok(Wait),
            "go" => parse_search_cmd(reader, board, dumb_interface, frc),
            "position" | "pos" => parse_position_cmd(reader, dumb_interface, frc),
            "setoption" => {
                if reader.next() != Some("name") {
                    return Err(MissingOptionNameToken);
                }

                let name = reader.next().ok_or(MissingOptionName)?.to_string();
                if reader.next() != Some("value") {
                    return Err(MissingOptionValueToken);
                }

                let mut remaining = input.trim_start();
                for token in [cmd, "name", name.as_str(), "value"] {
                    remaining = remaining.strip_prefix(token).unwrap().trim_start();
                }
                let value = remaining.trim_end().to_string();
                if value.is_empty() {
                    return Err(MissingOptionValue);
                }
                Ok(SetOption { name, value })
            }
            _ => Err(UnknownCommand(cmd.to_string())),
        }
    }
}

fn parse_relabel_cmd(reader: SplitWhitespace) -> Result<UciCommand, UciParseError> {
    use UciParseError::InvalidRelabelArguments;

    let tokens = reader.collect::<Vec<_>>();
    let [input, output, extra @ ..] = tokens.as_slice() else {
        return Err(InvalidRelabelArguments);
    };
    let mut nodes = 0;
    let mut threads = std::thread::available_parallelism().map_or(1, usize::from);
    for pair in extra.chunks(2) {
        match pair {
            ["nodes", value] => nodes = value.parse::<u64>()?,
            ["threads", value] => threads = value.parse::<usize>()?,
            _ => return Err(InvalidRelabelArguments),
        }
    }
    if threads == 0 || nodes > u64::MAX / 2000 {
        return Err(InvalidRelabelArguments);
    }
    Ok(UciCommand::Relabel {
        input: input.to_string(),
        output: output.to_string(),
        nodes,
        threads,
    })
}

fn parse_genfens_cmd(reader: SplitWhitespace) -> Result<UciCommand, UciParseError> {
    use UciParseError::*;

    let tokens = reader.collect::<Vec<_>>();
    if matches!(tokens.as_slice(), ["help" | "--help"]) {
        return Ok(UciCommand::GenFensHelp);
    }
    let [count, "seed", seed, "book", book, extra @ ..] = tokens.as_slice() else {
        return Err(InvalidGenFensArguments);
    };
    let count = count.parse()?;
    let seed = seed.parse()?;
    if !book.eq_ignore_ascii_case("none") {
        return Err(UnsupportedGenFensBook);
    }
    let (mut dfrc, mut plies) = (true, 8);
    for pair in extra.chunks(2) {
        match pair {
            ["dfrc", value] => dfrc = value.parse()?,
            ["moves", value] => plies = value.parse()?,
            _ => return Err(InvalidGenFensArguments),
        }
    }
    Ok(UciCommand::GenFens {
        count,
        seed,
        dfrc,
        plies,
    })
}

fn parse_search_cmd(
    reader: SplitWhitespace,
    board: &Board,
    dumb_interface: bool,
    frc: bool,
) -> Result<UciCommand, UciParseError> {
    use SearchLimit::*;
    use UciCommand::*;
    use UciParseError::*;

    #[inline]
    fn parse_int<'a, T: FromStr<Err = ParseIntError>>(
        reader: &mut impl Iterator<Item = &'a str>,
        token: &str,
    ) -> Result<T, UciParseError> {
        Ok(reader
            .next()
            .ok_or_else(|| MissingLimitValue(token.to_string()))?
            .parse::<T>()?)
    }

    let mut limits = Vec::new();
    let mut searchmoves = None;
    let mut excludemoves = Vec::new();
    let mut reader = reader.peekable();
    while let Some(token) = reader.next() {
        match token {
            "searchmoves" | "excludemoves" => {
                let moves = if token == "searchmoves" {
                    searchmoves.get_or_insert_with(Vec::new)
                } else {
                    &mut excludemoves
                };
                while let Some(token) = reader.next_if(|&token| {
                    !matches!(
                        token,
                        "infinite"
                            | "wtime"
                            | "btime"
                            | "winc"
                            | "binc"
                            | "movetime"
                            | "nodes"
                            | "depth"
                            | "searchmoves"
                            | "excludemoves"
                    )
                }) {
                    if token.contains('*') {
                        let pattern = MovePattern::parse(token, dumb_interface)
                            .ok_or_else(|| InvalidMove(token.to_string()))?;
                        board.gen_all_moves(|group| {
                            moves.extend(group.iter().filter(|&mv| pattern.matches(mv, frc)));
                            crate::util::Abort::No
                        });
                        continue;
                    }
                    let mv = Move::parse(board, dumb_interface, token)
                        .filter(|&mv| board.any_moves(|moves| moves.has(mv)))
                        .ok_or_else(|| InvalidMove(token.to_string()))?;
                    moves.push(mv);
                }
            }
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

    let root_moves = if searchmoves.is_some() || !excludemoves.is_empty() {
        let mut root_moves = Vec::new();
        board.gen_all_moves(|moves| {
            root_moves.extend(moves.iter().filter(|mv| {
                searchmoves.as_ref().is_none_or(|moves| moves.contains(mv))
                    && !excludemoves.contains(mv)
            }));
            crate::util::Abort::No
        });
        Some(root_moves)
    } else {
        None
    };
    Ok(Search { limits, root_moves })
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

        if !current.any_moves(|moves| moves.has(mv)) {
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

    #[error("Invalid genfens arguments (usage: {GENFENS_USAGE})")]
    InvalidGenFensArguments,
    #[error("Opening books are not supported; use book None")]
    UnsupportedGenFensBook,

    #[error(
        "Invalid relabel arguments (usage: relabel <input.wf> <output.wf> [nodes <n>] [threads <n>])"
    )]
    InvalidRelabelArguments,

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
    #[error("Invalid move: `{0}`")]
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
