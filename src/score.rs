use crate::board::Board;
use crate::common::Piece;
use crate::search::MAX_PLY;
use std::cmp::Ordering;
use std::fmt;
use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Score(pub i32);

impl Score {
    #[inline]
    pub fn draw(ply: usize, nodes: u64, contempt: i32) -> Self {
        let base = if ply.is_multiple_of(2) {
            -contempt
        } else {
            contempt
        };
        let random = ((nodes % 5) - 2) as i32;

        Self(base + random)
    }

    #[inline]
    pub fn mate(ply: usize) -> Self {
        Self::MIN_MATE - ply as i32
    }

    #[inline]
    pub fn mated(ply: usize) -> Self {
        -Self::MIN_MATE + ply as i32
    }

    #[inline]
    pub fn mate_in(self) -> Option<i16> {
        if self.is_mate() {
            let abs_score = self.0.abs();
            let sign = self.0.signum() as i16;

            Some(sign * (Score::MIN_MATE.0 - abs_score) as i16)
        } else {
            None
        }
    }

    #[inline]
    pub fn clamp_mate(self) -> Self {
        self.clamp(-Score::MAX_MATE + 1, Score::MAX_MATE - 1)
    }

    #[inline]
    pub fn is_mate(self) -> bool {
        let abs_score = Self(self.0.abs());

        abs_score >= Self::MAX_MATE && abs_score <= Self::MIN_MATE
    }

    #[inline]
    pub fn is_win(self) -> bool {
        self >= Self::MAX_MATE
    }

    #[inline]
    pub fn is_loss(self) -> bool {
        self <= -Self::MAX_MATE
    }

    fn wdl_parameters(board: &Board) -> (f64, f64) {
        let material = board.pieces(Piece::Pawn).popcnt()
            + 3 * board.pieces(Piece::Knight).popcnt()
            + 3 * board.pieces(Piece::Bishop).popcnt()
            + 5 * board.pieces(Piece::Rook).popcnt()
            + 9 * board.pieces(Piece::Queen).popcnt();
        let m = material.clamp(17, 78) as f64 / 58.0;
        let polynomial = |[a, b, c, d]: [f64; 4]| ((a * m + b) * m + c) * m + d;
        let a = polynomial([-218.40325114, 751.60081808, -1025.44651252, 746.54265415]);
        let b = polynomial([12.06485395, -79.64734824, 131.01688502, 84.63015106]);
        (a, b)
    }

    pub fn normalized(self, board: &Board) -> Self {
        if self.is_mate() {
            return self;
        }
        let (a, _) = Self::wdl_parameters(board);
        Self((100.0 * f64::from(self.0) / a).round() as i32)
    }

    pub fn wdl(self, board: &Board) -> [i32; 3] {
        if self.is_mate() {
            return if self.0 > 0 {
                [1000, 0, 0]
            } else {
                [0, 0, 1000]
            };
        }
        let (a, b) = Self::wdl_parameters(board);
        let win_rate = |score: f64| (1000.0 / (1.0 + ((a - score) / b).exp())).round() as i32;
        let wins = win_rate(f64::from(self.0));
        let losses = win_rate(-f64::from(self.0));
        [wins, 1000 - wins - losses, losses]
    }

    pub const MIN_MATE: Self = Self(i16::MAX as i32 - MAX_PLY as i32); //Mate in 0
    pub const MAX_MATE: Self = Self(i16::MAX as i32 - (2 * MAX_PLY) as i32); //Mate in MAX_PLY
    pub const INFINITE: Self = Self(i16::MAX as i32);
    pub const ZERO: Self = Score(0);
}

impl fmt::Display for Score {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            if let Some(ply) = self.mate_in() {
                write!(f, "#{}", ply)
            } else {
                write!(f, "{:+.2}", self.0 as f32 / 100.0)
            }
        } else {
            if let Some(ply) = self.mate_in() {
                write!(f, "mate {}", ply)
            } else {
                write!(f, "cp {}", self.0)
            }
        }
    }
}

impl PartialEq<Option<Score>> for Score {
    #[inline]
    fn eq(&self, other: &Option<Score>) -> bool {
        *self == other.unwrap_or(-Score::INFINITE)
    }
}

impl PartialOrd<Option<Score>> for Score {
    #[inline]
    fn partial_cmp(&self, other: &Option<Score>) -> Option<Ordering> {
        self.partial_cmp(&other.unwrap_or(-Score::INFINITE))
    }
}

impl PartialEq<Score> for Option<Score> {
    #[inline]
    fn eq(&self, other: &Score) -> bool {
        *other == self.unwrap_or(-Score::INFINITE)
    }
}

impl PartialOrd<Score> for Option<Score> {
    #[inline]
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        self.unwrap_or(-Score::INFINITE).partial_cmp(other)
    }
}

macro_rules! impl_score_ord {
    ($($trait:ident, $fn:ident, $out:ty;)*) => {$(
        impl $trait<i32> for Score {
            #[inline]
            fn $fn(&self, other: &i32) -> $out {
                self.0.$fn(other)
            }
        }

        impl $trait<Score> for i32 {
            #[inline]
            fn $fn(&self, other: &Score) -> $out {
                self.$fn(&other.0)
            }
        }
    )*}
}

impl_score_ord! {
    PartialEq, eq, bool;
    PartialOrd, partial_cmp, Option<Ordering>;
}

impl Neg for Score {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

macro_rules! impl_score_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Score {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn(rhs.0))
            }
        }

        impl $trait<i32> for Score {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: i32) -> Self::Output {
                Self(self.0.$fn(rhs))
            }
        }

        impl $trait<Score> for i32 {
            type Output = Score;

            #[inline]
            fn $fn(self, rhs: Score) -> Self::Output {
                Score(self.$fn(rhs.0))
            }
        }
    )*}
}

macro_rules! impl_score_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Score {
            #[inline]
            fn $fn(&mut self, rhs: Self) {
                self.0.$fn(rhs.0);
            }
        }

        impl $trait<i32> for Score {
            #[inline]
            fn $fn(&mut self, rhs: i32) {
                self.0.$fn(rhs);
            }
        }
    )*}
}

impl_score_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
}

impl_score_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
    MulAssign, mul_assign;
    DivAssign, div_assign;
}
