use crate::board::Board;
use crate::common::{Move, Piece};
use crate::position::Position;
use crate::search::{ContIndices, History, ThreadData};
#[cfg(feature = "tune")]
use crate::uci::UciParseError;
use std::cell::UnsafeCell;

// `std::cell::SyncUnsafeCell` is nightly only
pub struct SyncUnsafeCell<T>(pub UnsafeCell<T>);

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

macro_rules! params {
    ($($name:ident : $ty:ty => $default:literal;)*) => {
        pub struct Params;

        $(
            #[allow(non_upper_case_globals)]
            #[cfg(feature = "tune")]
            pub static $name: SyncUnsafeCell<$ty> = SyncUnsafeCell(UnsafeCell::new($default));
        )*

        impl Params {
            $(
                #[cfg(feature = "tune")]
                pub const fn $name() -> $ty { unsafe { *$name.0.get() } }

                #[cfg(not(feature = "tune"))]
                pub const fn $name() -> $ty { $default }
            )*

            #[cfg(feature = "tune")]
            pub fn set_param(name: &str, value: String) {
                match name {
                    $(
                        stringify!($name) => {
                            let value = match value.parse::<$ty>() {
                                Ok(value) => value,
                                Err(e) => {
                                    println!("info string {:?}", UciParseError::InvalidInteger(e));
                                    return;
                                }
                            };

                            unsafe { *$name.0.get() = value };
                        },
                    )*
                    _ => eprintln!("info string Unknown Option: `{name}`"),
                }
            }

            #[cfg(feature = "tune")]
            pub fn is_weight(name: &str) -> bool {
                match name {
                    $(stringify!($name) => true,)*
                    _ => false,
                }
            }
        }
    }
}

params! {
    pawn_corr:        i32 => 64;
    minor_corr:       i32 => 64;
    major_corr:       i32 => 64;
    nonpawn_corr:     i32 => 64;
    corr_bonus_scale: i64 => 128;

    quiet_bonus_base:  i32 => 128;
    quiet_bonus_scale: i32 => 128;
    quiet_bonus_max:   i32 => 2048;
    quiet_malus_base:  i32 => 128;
    quiet_malus_scale: i32 => 128;
    quiet_malus_max:   i32 => 2048;

    noisy_bonus_base:  i32 => 128;
    noisy_bonus_scale: i32 => 128;
    noisy_bonus_max:   i32 => 2048;
    noisy_malus_base:  i32 => 128;
    noisy_malus_scale: i32 => 128;
    noisy_malus_max:   i32 => 2048;

    pawn_bonus_base:  i32 => 128;
    pawn_bonus_scale: i32 => 128;
    pawn_bonus_max:   i32 => 2048;
    pawn_malus_base:  i32 => 128;
    pawn_malus_scale: i32 => 128;
    pawn_malus_max:   i32 => 2048;

    duck_bonus_base:  i32 => 128;
    duck_bonus_scale: i32 => 128;
    duck_bonus_max:   i32 => 2048;
    duck_malus_base:  i32 => 128;
    duck_malus_scale: i32 => 128;
    duck_malus_max:   i32 => 2048;

    cont1_bonus_base:  i32 => 128;
    cont1_bonus_scale: i32 => 128;
    cont1_bonus_max:   i32 => 2048;
    cont1_malus_base:  i32 => 128;
    cont1_malus_scale: i32 => 128;
    cont1_malus_max:   i32 => 2048;

    cont2_bonus_base:  i32 => 128;
    cont2_bonus_scale: i32 => 128;
    cont2_bonus_max:   i32 => 2048;
    cont2_malus_base:  i32 => 128;
    cont2_malus_scale: i32 => 128;
    cont2_malus_max:   i32 => 2048;

    cont4_bonus_base:  i32 => 128;
    cont4_bonus_scale: i32 => 128;
    cont4_bonus_max:   i32 => 2048;
    cont4_malus_base:  i32 => 128;
    cont4_malus_scale: i32 => 128;
    cont4_malus_max:   i32 => 2048;

    rfp_base:      i32 => 0;
    rfp_scale:     i32 => 50;
    rfp_imp_base:  i32 => -50;
    rfp_imp_scale: i32 => 50;

    razor_base:  i32 => 320;
    razor_scale: i32 => 250;

    nmr_margin:          i32 => 20;
    nmr_reduction_base:  i32 => 5120;
    nmr_reduction_scale: i32 => 341;

    iid_depth_scale:     i32 => 768;
    iid_depth_reduction: i32 => 1536;

    mvv_pawn:   i32 => 100;
    mvv_knight: i32 => 320;
    mvv_bishop: i32 => 330;
    mvv_rook:   i32 => 500;
    mvv_queen:  i32 => 900;

    see_pawn:   i32 => 100;
    see_knight: i32 => 320;
    see_bishop: i32 => 330;
    see_rook:   i32 => 500;
    see_queen:  i32 => 900;

    quiet_ldp_imp_threshold_base:  i32 => 2;
    quiet_ldp_imp_threshold_scale: i32 => 2;
    quiet_ldp_threshold_base:      i32 => 1;
    quiet_ldp_threshold_scale:     i32 => 1;
    quiet_ldp_history_offset:      i32 => -4000;
    quiet_ldp_history_div:         i32 => 4000;
    quiet_ldp_history_min:         i32 => -2;
    quiet_ldp_history_max:         i32 => 2;

    noisy_ldp_imp_threshold_base:  i32 => 4;
    noisy_ldp_imp_threshold_scale: i32 => 4;
    noisy_ldp_threshold_base:      i32 => 4;
    noisy_ldp_threshold_scale:     i32 => 4;

    dcp_threshold_imp_base:  i32 => 2;
    dcp_threshold_imp_scale: i32 => 1;
    dcp_threshold_base:      i32 => 4;
    dcp_threshold_scale:     i32 => 2;
    dcp_history_offset:      i32 => -4000;
    dcp_history_div:         i32 => 4000;
    dcp_history_min:         i32 => -2;
    dcp_history_max:         i32 => 2;

    udp_threshold_base:  i32 => 10;
    udp_threshold_scale: i32 => 4;

    see_base:  i32 => 0;
    see_scale: i32 => -80;

    mp_see_threshold: i32 => 0;
    mp_qs_see_threshold: i32 => 0;
    mp_quiet_neutral_malus: i32 => 5000;

    qsldp_threshold: i32 => 2;
    qsdcp_threshold: i32 => 2;

    soft_time_div: u64 => 98304;
    soft_time_inc: u64 => 2048;
    hard_time_div: u64 => 12288;
    hard_time_inc: u64 => 4096;

    duck_stability_base:  u128 => 5325;
    duck_stability_scale: u128 => 410;
    duck_stability_min:   u128 => 2867;

    move_stability_base:  u128 => 5325;
    move_stability_scale: u128 => 410;
    move_stability_min:   u128 => 2867;

    quiet_lmr_base:  i32 => 1024;
    quiet_lmr_scale: i32 => 448;
    lmr_exact:       i32 => 1024;
    lmr_imp:         i32 => 1024;
    lmr_pv:          i32 => 1024;
    lmr_in_check:    i32 => 512;
    lmr_history:     i32 => 64;

    fp_base:  i32 => 256;
    fp_scale: i32 => 128;

    noisy_lmr_noisy_scale: i32 => 128;
    noisy_lmr_duck_scale:  i32 => 128;

    quiet_lmr_quiet_scale: i32 => 1024;
    quiet_lmr_duck_scale:  i32 => 1024;
    quiet_lmr_cont1_scale: i32 => 1024;
    quiet_lmr_cont2_scale: i32 => 1024;

    quiet_mp_quiet_scale: i32 => 1024;
    quiet_mp_duck_scale:  i32 => 1024;
    quiet_mp_pawn_scale:  i32 => 1024;
    quiet_mp_cont1_scale: i32 => 1024;
    quiet_mp_cont2_scale: i32 => 1024;
    quiet_mp_cont4_scale: i32 => 1024;

    noisy_mp_noisy_scale: i32 => 128;
    noisy_mp_duck_scale:  i32 => 128;
}

impl Params {
    #[inline]
    pub fn corr_bonus(depth: i32, diff: i64) -> i32 {
        (diff * depth as i64 * Params::corr_bonus_scale() / 1024) as i32
    }

    #[inline]
    pub fn quiet_bonus(depth: i32) -> i32 {
        (Self::quiet_bonus_base() + Self::quiet_bonus_scale() * depth).min(Self::quiet_bonus_max())
    }

    #[inline]
    pub fn quiet_malus(depth: i32) -> i32 {
        -(Self::quiet_malus_base() + Self::quiet_malus_scale() * depth).min(Self::quiet_malus_max())
    }

    #[inline]
    pub fn noisy_bonus(depth: i32) -> i32 {
        (Self::noisy_bonus_base() + Self::noisy_bonus_scale() * depth).min(Self::noisy_bonus_max())
    }

    #[inline]
    pub fn noisy_malus(depth: i32) -> i32 {
        -(Self::noisy_malus_base() + Self::noisy_malus_scale() * depth).min(Self::noisy_malus_max())
    }

    #[inline]
    pub fn pawn_bonus(depth: i32) -> i32 {
        (Self::pawn_bonus_base() + Self::pawn_bonus_scale() * depth).min(Self::pawn_bonus_max())
    }

    #[inline]
    pub fn pawn_malus(depth: i32) -> i32 {
        -(Self::pawn_malus_base() + Self::pawn_malus_scale() * depth).min(Self::pawn_malus_max())
    }

    #[inline]
    pub fn duck_bonus(depth: i32) -> i32 {
        (Self::duck_bonus_base() + Self::duck_bonus_scale() * depth).min(Self::duck_bonus_max())
    }

    #[inline]
    pub fn duck_malus(depth: i32) -> i32 {
        -(Self::duck_malus_base() + Self::duck_malus_scale() * depth).min(Self::duck_malus_max())
    }

    #[inline]
    pub fn cont_bonus<const PLY: usize>(depth: i32) -> i32 {
        let (base, scale, max) = match PLY {
            1 => (
                Self::cont1_bonus_base(),
                Self::cont1_bonus_scale(),
                Self::cont1_bonus_max(),
            ),
            2 => (
                Self::cont2_bonus_base(),
                Self::cont2_bonus_scale(),
                Self::cont2_bonus_max(),
            ),
            4 => (
                Self::cont4_bonus_base(),
                Self::cont4_bonus_scale(),
                Self::cont4_bonus_max(),
            ),
            _ => unreachable!(),
        };

        (base + scale * depth).min(max)
    }

    #[inline]
    pub fn cont_malus<const PLY: usize>(depth: i32) -> i32 {
        let (base, scale, max) = match PLY {
            1 => (
                Self::cont1_malus_base(),
                Self::cont1_malus_scale(),
                Self::cont1_malus_max(),
            ),
            2 => (
                Self::cont2_malus_base(),
                Self::cont2_malus_scale(),
                Self::cont2_malus_max(),
            ),
            4 => (
                Self::cont4_malus_base(),
                Self::cont4_malus_scale(),
                Self::cont4_malus_max(),
            ),
            _ => unreachable!(),
        };

        -(base + scale * depth).min(max)
    }

    #[inline]
    pub const fn rfp_margin(depth: i32, improving: bool) -> i32 {
        let (base, scale) = if improving {
            (Self::rfp_imp_base(), Self::rfp_imp_scale())
        } else {
            (Self::rfp_base(), Self::rfp_scale())
        };

        base + scale * depth
    }

    #[inline]
    pub const fn razor_margin(depth: i32) -> i32 {
        Self::razor_base() + Self::razor_scale() * depth
    }

    #[inline]
    pub fn ldp_threshold(depth: i32, is_quiet: bool, improving: bool, duck_history: i32) -> i32 {
        let (base, scale) = match (is_quiet, improving) {
            (true, true) => (
                Self::quiet_ldp_imp_threshold_base(),
                Self::quiet_ldp_imp_threshold_scale(),
            ),
            (true, false) => (
                Self::quiet_ldp_threshold_base(),
                Self::quiet_ldp_threshold_scale(),
            ),
            (false, true) => (
                Self::noisy_ldp_imp_threshold_base(),
                Self::noisy_ldp_imp_threshold_scale(),
            ),
            (false, false) => (
                Self::noisy_ldp_threshold_base(),
                Self::noisy_ldp_threshold_scale(),
            ),
        };

        let mut threshold = base + scale * depth;
        if is_quiet {
            let offset = Params::quiet_ldp_history_offset();
            let divisor = Params::quiet_ldp_history_div();
            let min = Params::quiet_ldp_history_min();
            let max = Params::quiet_ldp_history_max();
            threshold += ((duck_history + offset) / divisor).clamp(min, max);
        }
        threshold
    }

    #[inline]
    pub fn dcp_threshold(depth: i32, improving: bool, duck_history: i32) -> i32 {
        let (base, scale) = if improving {
            (
                Self::dcp_threshold_imp_base(),
                Self::dcp_threshold_imp_scale(),
            )
        } else {
            (Self::dcp_threshold_base(), Self::dcp_threshold_scale())
        };
        let mut threshold = base + scale * depth;

        let offset = Params::dcp_history_offset();
        let divisor = Params::dcp_history_div();
        let min = Params::dcp_history_min();
        let max = Params::dcp_history_max();
        threshold += ((duck_history + offset) / divisor).clamp(min, max);

        threshold
    }

    #[inline]
    pub fn udp_threshold(depth: i32) -> i32 {
        Self::udp_threshold_base() + Self::udp_threshold_scale() * depth
    }

    #[inline]
    pub fn see_margin(depth: i32) -> i32 {
        Self::see_base() + Self::see_scale() * depth
    }

    #[inline]
    pub const fn mvv_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => Self::mvv_pawn(),
            Piece::Knight => Self::mvv_knight(),
            Piece::Bishop => Self::mvv_bishop(),
            Piece::Rook => Self::mvv_rook(),
            Piece::Queen => Self::mvv_queen(),
            Piece::King => 20000,
        }
    }

    #[inline]
    pub const fn see_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => Self::see_pawn(),
            Piece::Knight => Self::see_knight(),
            Piece::Bishop => Self::see_bishop(),
            Piece::Rook => Self::see_rook(),
            Piece::Queen => Self::see_queen(),
            Piece::King => 20000,
        }
    }

    #[inline]
    pub fn duck_stability(stability: u16) -> u128 {
        Self::duck_stability_base()
            .saturating_sub(Self::duck_stability_scale() * stability as u128)
            .max(Self::duck_stability_min())
    }

    #[inline]
    pub fn move_stability(stability: u16) -> u128 {
        Self::move_stability_base()
            .saturating_sub(Self::move_stability_scale() * stability as u128)
            .max(Self::move_stability_min())
    }

    #[inline]
    pub fn lmr(depth: i32) -> i32 {
        let log_depth = depth.ilog2() as i32;

        Self::quiet_lmr_base() + Self::quiet_lmr_scale() * log_depth
    }

    #[inline]
    pub fn noisy_lmr_history(thread: &ThreadData, pos: &Position, mv: Move) -> i32 {
        let board = pos.board();
        let mut history = 0;

        history += thread.history.noisy(board, mv) * Self::noisy_lmr_noisy_scale();
        history += thread.history.duck(board, mv) * Self::noisy_lmr_duck_scale();

        history / 1024
    }

    #[inline]
    pub fn quiet_lmr_history(thread: &ThreadData, pos: &Position, mv: Move) -> i32 {
        let board = pos.board();
        let mut history = 0;
        let indices = ContIndices::new(pos);

        history += thread.history.quiet(board, mv) * Self::quiet_lmr_quiet_scale();
        history += thread.history.duck(board, mv) * Self::quiet_lmr_duck_scale();
        history += thread.history.cont1(board, indices, mv) * Self::quiet_lmr_cont1_scale();
        history += thread.history.cont2(board, indices, mv) * Self::quiet_lmr_cont2_scale();

        history / 1024
    }

    #[inline]
    pub fn quiet_mp_history(
        history: &History,
        board: &Board,
        indices: ContIndices,
        mv: Move,
    ) -> i32 {
        let mut history_score = 0;

        history_score += history.quiet(board, mv) * Self::quiet_mp_quiet_scale();
        history_score += history.duck(board, mv) * Self::quiet_mp_duck_scale();
        history_score += history.pawn(board, mv) * Self::quiet_mp_pawn_scale();
        history_score += history.cont1(board, indices, mv) * Self::quiet_mp_cont1_scale();
        history_score += history.cont2(board, indices, mv) * Self::quiet_mp_cont2_scale();
        history_score += history.cont4(board, indices, mv) * Self::quiet_mp_cont4_scale();

        history_score / 1024
    }

    #[inline]
    pub fn noisy_mp_history(history: &History, board: &Board, mv: Move) -> i32 {
        let mut history_score = 0;

        history_score += history.noisy(board, mv) * Self::noisy_mp_noisy_scale() / 1024;
        history_score += history.duck(board, mv) * Self::noisy_mp_duck_scale() / 1024;

        history_score
    }

    #[inline]
    pub fn iid_depth(depth: i32) -> i32 {
        (Params::iid_depth_scale() * depth - Params::iid_depth_reduction()) / 1024
    }

    #[inline]
    pub fn nmr_reduction(depth: i32) -> i32 {
        (Self::nmr_reduction_base() + depth * Self::nmr_reduction_scale()) / 1024
    }

    #[inline]
    pub fn fp_margin(depth: i32) -> i32 {
        Params::fp_base() + Params::fp_scale() * depth
    }
}
