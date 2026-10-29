use crate::common::Move;

pub(super) struct MovePattern<'a> {
    piece: &'a str,
    duck: &'a str,
}

impl<'a> MovePattern<'a> {
    pub(super) fn parse(token: &'a str, dumb_interface: bool) -> Option<Self> {
        let (piece, duck) = if token == "*" {
            ("*", "*")
        } else if dumb_interface {
            let (piece, duck) = token.split_once(',')?;
            let duck = if duck == "*" {
                duck
            } else {
                duck.get(duck.len().checked_sub(2)?..)?
            };
            (piece, duck)
        } else {
            token.split_once('@')?
        };
        if piece.is_empty()
            || duck.is_empty()
            || !piece
                .bytes()
                .all(|c| matches!(c, b'a'..=b'h' | b'1'..=b'8' | b'n' | b'q' | b'r' | b'*'))
            || !duck
                .bytes()
                .all(|c| matches!(c, b'a'..=b'h' | b'1'..=b'8' | b'*'))
        {
            return None;
        }
        Some(Self { piece, duck })
    }

    pub(super) fn matches(&self, mv: Move, frc: bool) -> bool {
        let notation = mv.display(false, frc);
        let (piece, duck) = notation.split_once('@').unwrap();
        matches_pattern(piece, self.piece) && matches_pattern(duck, self.duck)
    }
}

fn matches_pattern(value: &str, pattern: &str) -> bool {
    let mut parts = pattern.split('*').peekable();
    let Some(mut remaining) = value.strip_prefix(parts.next().unwrap()) else {
        return false;
    };
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            return remaining.ends_with(part);
        }
        let Some(index) = remaining.find(part) else {
            return false;
        };
        remaining = &remaining[index + part.len()..];
    }
    remaining.is_empty()
}
