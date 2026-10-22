use super::config::{Range, Source, SourceKind};
use super::{Result, ensure};
use crate::board::{Board, CastlingDirection, SplitMix64};
use crate::common::{Move, Piece, Square};
use crate::position::Position;
use crate::util::Abort;

#[derive(Clone)]
pub struct Candidate {
    pub initial_fen: String,
    pub moves: Vec<Move>,
    pub position: Position,
    pub source: String,
}

impl Candidate {
    pub fn play(&mut self, mv: Move) {
        self.position.make_move(mv);
        self.position.reset_nnue();
        self.moves.push(mv);
    }

    pub fn ended(&self) -> bool {
        let board = self.position.board();
        board.try_king(board.stm()).is_none()
            || board.try_king(!board.stm()).is_none()
            || board.terminal_state().is_some()
            || self.position.repetition(0)
    }
}

struct Seed {
    board: Board,
    moves: Vec<Move>,
    label: String,
}

pub struct Input {
    pub config: Source,
    seeds: Vec<Seed>,
}

pub fn board(fen: &str) -> Result<Board> {
    let placement = fen.split_whitespace().next().ok_or("empty FEN")?;
    ensure(
        placement.matches('K').count() == 1 && placement.matches('k').count() == 1,
        "source FEN needs exactly one king per side",
    )?;
    Board::from_fen(fen).ok_or_else(|| format!("invalid FEN: {fen}").into())
}

fn parse_move(board: &Board, token: &str) -> Result<Move> {
    for dumb in [false, true] {
        if let Some(mv) = Move::parse(board, dumb, token)
            && board.any_moves(|moves| moves.has(mv))
        {
            return Ok(mv);
        }
    }
    let token = token
        .trim_end_matches(['+', '#', '!', '?'])
        .replace('0', "O");
    let (san, duck) = token
        .rsplit_once('@')
        .map_or((token.as_str(), None), |(san, duck)| (san, Some(duck)));
    let duck = duck
        .map(str::parse::<Square>)
        .transpose()
        .map_err(|_| "invalid duck square")?;
    let (body, promotion) = san
        .split_once('=')
        .map_or((san, None), |(body, piece)| (body, Some(piece)));
    let promotion = promotion
        .map(|piece| -> Result<Piece> {
            ensure(piece.len() == 1, "invalid promotion")?;
            Piece::try_from(piece.chars().next().unwrap()).map_err(|_| "invalid promotion".into())
        })
        .transpose()?;
    let piece = body
        .chars()
        .next()
        .filter(char::is_ascii_uppercase)
        .and_then(|c| Piece::try_from(c).ok())
        .unwrap_or(Piece::Pawn);
    let mut found = Vec::new();
    board.gen_all_moves(|moves| {
        let Some(mv) = moves
            .iter()
            .find(|mv| duck.is_none_or(|duck| mv.duck() == duck))
        else {
            return Abort::No;
        };
        if duck.is_none() && board.piece_on(mv.dest()) != Some(Piece::King) {
            return Abort::No;
        }
        let matches = if body == "O-O" || body == "O-O-O" {
            mv.flag()
                .castling_dir()
                .is_some_and(|dir| (dir == CastlingDirection::Short) == (body == "O-O"))
        } else if body.len() >= 2 && body.is_ascii() {
            board.piece_on(mv.src()) == Some(piece)
                && mv.dest().to_string() == body[body.len() - 2..]
                && mv.flag().promotion() == promotion
                && mv.flag().is_capture() == body.contains('x')
                && body[usize::from(piece != Piece::Pawn)..body.len() - 2]
                    .chars()
                    .filter(|&c| c != 'x')
                    .all(|c| mv.src().to_string().contains(c))
                && mv.flag().castling_dir().is_none()
        } else {
            false
        };
        if matches {
            found.push(mv);
        }
        Abort::No
    });
    ensure(
        found.len() == 1,
        &format!("illegal or ambiguous move: {token}"),
    )?;
    Ok(found[0])
}

pub fn san(board: &Board, mv: Move) -> String {
    let mut result = String::new();
    if let Some(dir) = mv.flag().castling_dir() {
        result.push_str(if dir == CastlingDirection::Short {
            "O-O"
        } else {
            "O-O-O"
        });
    } else {
        let piece = board.piece_on(mv.src()).unwrap();
        if piece != Piece::Pawn {
            result.push(char::from(piece).to_ascii_uppercase());
            let mut alternatives = Vec::new();
            board.gen_all_moves(|moves| {
                if moves.src != mv.src()
                    && moves.dest == mv.dest()
                    && board.piece_on(moves.src) == Some(piece)
                    && moves.is_nonempty()
                {
                    alternatives.push(moves.src);
                }
                Abort::No
            });
            if !alternatives.is_empty() {
                if alternatives.iter().all(|sq| sq.file() != mv.src().file()) {
                    result.push(char::from(mv.src().file()));
                } else if alternatives.iter().all(|sq| sq.rank() != mv.src().rank()) {
                    result.push(char::from(mv.src().rank()));
                } else {
                    result.push_str(&mv.src().to_string());
                }
            }
        } else if mv.flag().is_capture() {
            result.push(char::from(mv.src().file()));
        }
        if mv.flag().is_capture() {
            result.push('x');
        }
        result.push_str(&mv.dest().to_string());
        if let Some(piece) = mv.flag().promotion() {
            result.push('=');
            result.push(char::from(piece).to_ascii_uppercase());
        }
    }
    format!("{result}@{}", mv.duck())
}

fn pgn(text: &str, label: &str) -> Result<Vec<Seed>> {
    let mut text = text.trim_start_matches('\u{feff}');
    let mut seeds = Vec::new();
    let (mut variation, mut started, mut captured) = (0usize, false, false);
    let mut seed = Seed {
        board: Board::startpos(),
        moves: Vec::new(),
        label: String::new(),
    };
    let mut current = seed.board;
    let finish = |seed: &mut Seed, seeds: &mut Vec<Seed>| {
        let next = Seed {
            board: Board::startpos(),
            moves: Vec::new(),
            label: String::new(),
        };
        seed.label = format!("{label}:{}", seeds.len() + 1);
        seeds.push(std::mem::replace(seed, next));
    };
    let mut headers = false;
    while !text.is_empty() {
        text = text.trim_start();
        let Some(first) = text.chars().next() else {
            break;
        };
        match first {
            '{' | ';' | '%' => {
                let delimiter = if first == '{' { '}' } else { '\n' };
                let end = text.find(delimiter);
                ensure(end.is_some() || first != '{', "unterminated PGN comment")?;
                text = end.map_or("", |end| &text[end + 1..]);
            }
            '(' | ')' => {
                if first == '(' {
                    variation += 1;
                } else {
                    ensure(variation > 0, "unmatched PGN variation")?;
                    variation -= 1;
                }
                text = &text[1..];
            }
            '[' => {
                let (mut quote, mut escape) = (false, false);
                let end = text
                    .char_indices()
                    .skip(1)
                    .find_map(|(index, ch)| {
                        if ch == ']' && !quote {
                            return Some(index);
                        }
                        if ch == '"' && !escape {
                            quote = !quote;
                        }
                        escape = ch == '\\' && !escape;
                        None
                    })
                    .ok_or("unterminated PGN header")?;
                if variation == 0 {
                    if started {
                        finish(&mut seed, &mut seeds);
                        started = false;
                        captured = false;
                    }
                    let (key, value) = text[1..end]
                        .split_once(char::is_whitespace)
                        .ok_or("invalid PGN header")?;
                    if key == "FEN" {
                        seed.board = board(value.trim().trim_matches('"'))?;
                    }
                    headers = true;
                }
                text = &text[end + 1..];
            }
            '$' => {
                text = text[1..].trim_start_matches(|c: char| c.is_ascii_digit());
            }
            _ => {
                let end = text
                    .find(|c: char| c.is_whitespace() || "{}()[;$".contains(c))
                    .unwrap_or(text.len());
                ensure(end > 0, "unexpected PGN delimiter")?;
                let token = &text[..end];
                text = &text[end..];
                if variation > 0 {
                    continue;
                }
                if ["1-0", "0-1", "1/2-1/2", "*"].contains(&token) {
                    if started || headers {
                        finish(&mut seed, &mut seeds);
                    }
                    (started, captured, headers) = (false, false, false);
                    continue;
                }
                let token = token.rsplit('.').next().unwrap();
                if token.is_empty() {
                    continue;
                }
                ensure(!captured, "PGN continues after king capture")?;
                if !started {
                    current = seed.board;
                    started = true;
                }
                let mv = parse_move(&current, token).map_err(|error| {
                    format!(
                        "{label}: game {}, move {}: {error}",
                        seeds.len() + 1,
                        seed.moves.len() + 1
                    )
                })?;
                captured = current.piece_on(mv.dest()) == Some(Piece::King);
                if !captured {
                    ensure(
                        current.terminal_state().is_none(),
                        "PGN continues after game end",
                    )?;
                    current.make_move(mv);
                    seed.moves.push(mv);
                }
            }
        }
    }
    ensure(variation == 0, "unterminated PGN variation")?;
    if started || headers {
        finish(&mut seed, &mut seeds);
    }
    Ok(seeds)
}

impl Input {
    pub fn load(config: Source) -> Result<Self> {
        let mut seeds = Vec::new();
        if let Some(path) = &config.path {
            let text = std::fs::read_to_string(path)?;
            let label = path.display().to_string();
            if config.kind == SourceKind::Pgn {
                seeds = pgn(&text, &label)?;
                seeds.retain(|seed| seed.moves.len() >= config.plies.min);
            } else {
                for (index, line) in text
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
                {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    ensure(fields.len() >= 4, "position needs at least four FEN fields")?;
                    let words: Vec<_> = line.split(';').flat_map(str::split_whitespace).collect();
                    let counter = |name| {
                        words
                            .windows(2)
                            .find(|pair| pair[0] == name)
                            .map(|pair| pair[1])
                    };
                    let counters = if fields.len() >= 6
                        && fields[4].parse::<u8>().is_ok()
                        && fields[5].parse::<u16>().is_ok()
                    {
                        format!("{} {}", fields[4], fields[5])
                    } else {
                        format!(
                            "{} {}",
                            counter("hmvc").unwrap_or("0"),
                            counter("fmvn").or_else(|| counter("fmvc")).unwrap_or("1")
                        )
                    };
                    seeds.push(Seed {
                        board: board(&format!("{} {counters}", fields[..4].join(" ")))?,
                        moves: Vec::new(),
                        label: format!("{label}:{}", index + 1),
                    });
                }
            }
            ensure(
                !seeds.is_empty(),
                &format!("no usable positions in {label}"),
            )?;
        }
        Ok(Self { config, seeds })
    }

    pub fn sample(&self, rng: &mut SplitMix64) -> Candidate {
        let (board, moves, source) = if self.config.kind == SourceKind::Start {
            let white = if self.config.shared_id {
                Range {
                    min: self.config.white_ids.min.max(self.config.black_ids.min),
                    max: self.config.white_ids.max.min(self.config.black_ids.max),
                }
                .sample(rng)
            } else {
                self.config.white_ids.sample(rng)
            };
            let black = if self.config.shared_id {
                white
            } else {
                self.config.black_ids.sample(rng)
            };
            (
                Board::dfrc_startpos(white as u16, black as u16),
                &[][..],
                format!("start:{white}:{black}"),
            )
        } else {
            let seed = &self.seeds[(rng.next() % self.seeds.len() as u64) as usize];
            let count = if self.config.kind == SourceKind::Pgn {
                Range {
                    min: self.config.plies.min,
                    max: self.config.plies.max.min(seed.moves.len()),
                }
                .sample(rng)
            } else {
                0
            };
            (seed.board, &seed.moves[..count], seed.label.clone())
        };
        let mut candidate = Candidate {
            initial_fen: board.to_fen(true),
            moves: Vec::new(),
            position: Position::new(board),
            source,
        };
        for &mv in moves {
            candidate.play(mv);
        }
        candidate
    }
}
