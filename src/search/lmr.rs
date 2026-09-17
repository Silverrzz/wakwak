use crate::search::Params;

pub struct LmrTable {
    base: [[[i32; 2]; 64]; 256],
}

impl LmrTable {
    pub fn init(&mut self) {
        for depth in 1..256 {
            for move_count in 1..64 {
                for is_quiet in [true, false] {
                    let base = if is_quiet {
                        Params::lmr_quiet_base() as f32 / 100.0
                    } else {
                        Params::lmr_noisy_base() as f32 / 100.0
                    };
                    let divisor = if is_quiet {
                        Params::lmr_quiet_div() as f32 / 100.0
                    } else {
                        Params::lmr_noisy_div() as f32 / 100.0
                    };
                    let ln_depth = (depth as f32).ln();
                    let ln_move_count = (move_count as f32).ln();
                    let reduction = (base + (ln_depth * ln_move_count / divisor)) as i32;
                    self.base[depth as usize][move_count as usize][is_quiet as usize] = reduction;
                }
            }
        }
    }

    #[inline]
    pub fn base(&self, depth: i32, move_count: i32, is_quiet: bool) -> i32 {
        self.base[depth.min(255) as usize][move_count.min(63) as usize][is_quiet as usize]
    }
}
