use crate::common::Move;
use crate::engine::EngineOptions;
use crate::search::MAX_PLY;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct PrincipalVariation {
    pub moves: [Option<Move>; MAX_PLY + 1],
    pub len: usize,
}

impl PrincipalVariation {
    #[inline]
    pub fn update(&mut self, mv: Move, child_pv: &PrincipalVariation) {
        self.moves[0] = Some(mv);
        self.len = child_pv.len + 1;
        self.moves[1..self.len].copy_from_slice(&child_pv.moves[..child_pv.len]);
    }

    #[inline]
    pub fn display(&self, options: EngineOptions) -> String {
        let mut f = String::new();
        if self.len != 0 {
            for &mv in self.moves[..self.len].iter() {
                if let Some(mv) = mv {
                    write!(f, "{} ", mv.display(options.dumb_interface, options.frc)).unwrap();
                } else {
                    break;
                }
            }
        }

        f
    }
}

impl Default for PrincipalVariation {
    #[inline]
    fn default() -> Self {
        Self {
            moves: [None; MAX_PLY + 1],
            len: 0,
        }
    }
}
