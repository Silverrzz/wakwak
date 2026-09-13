use crate::common::Move;
use crate::engine::EngineOptions;
use crate::search::MAX_PLY;
use arrayvec::ArrayVec;
use std::fmt::Write;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct PrincipalVariation {
    moves: ArrayVec<Move, { MAX_PLY + 1 }>,
}

impl PrincipalVariation {
    #[inline]
    pub fn display(&self, options: EngineOptions) -> String {
        let mut f = String::new();
        for mv in self.moves.iter() {
            write!(f, "{} ", mv.display(options.dumb_interface, options.frc)).unwrap();
        }

        f
    }
}

impl Default for PrincipalVariation {
    #[inline]
    fn default() -> Self {
        Self {
            moves: ArrayVec::new(),
        }
    }
}

impl Deref for PrincipalVariation {
    type Target = ArrayVec<Move, { MAX_PLY + 1 }>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

impl DerefMut for PrincipalVariation {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves
    }
}
