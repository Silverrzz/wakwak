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
    ($($name:ident : $ty:ty => $default:literal, $min:literal, $max:literal, $step:literal;)*) => {
        pub struct Params;

        $(
            #[allow(non_upper_case_globals)]
            #[cfg(feature = "tune")]
            pub static $name: SyncUnsafeCell<$ty> = SyncUnsafeCell(UnsafeCell::new($default));
        )*

        impl Params {
            pub fn print_spsa() {
                $(
                    println!("{}, int, {}, {}, {}, {}, 0.002",
                        stringify!($name), Self::$name(), $min, $max, $step);
                )*
            }

            #[cfg(feature = "tune")]
            pub fn print_uci_options() {
                $(
                    println!("option name {} type spin default {} min {} max {}",
                        stringify!($name), $default, $min, $max);
                )*
            }

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

                            if !($min..=$max).contains(&value) {
                                eprintln!("info string {} must be between {} and {}",
                                    name, $min, $max);
                                return;
                            }

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
    pawn_corr:        i32 => 64, 0, 128, 1.6;
    minor_corr:       i32 => 66, 0, 128, 1.6;
    major_corr:       i32 => 67, 0, 128, 1.6;
    nonpawn_corr:     i32 => 66, 0, 128, 1.6;
    cont1_corr:       i32 => 64, 0, 128, 1.6;
    cont2_corr:       i32 => 62, 0, 128, 1.6;
    corr_bonus_scale: i64 => 131, 0, 256, 3.2;

    quiet_bonus_base:  i32 => 127, 0, 256, 3.2;
    quiet_bonus_scale: i32 => 126, 0, 256, 3.2;
    quiet_bonus_max:   i32 => 2044, 0, 4096, 51.2;
    quiet_malus_base:  i32 => 125, 0, 256, 3.2;
    quiet_malus_scale: i32 => 127, 0, 256, 3.2;
    quiet_malus_max:   i32 => 2117, 0, 4096, 51.2;

    noisy_bonus_base:  i32 => 128, 0, 256, 3.2;
    noisy_bonus_scale: i32 => 127, 0, 256, 3.2;
    noisy_bonus_max:   i32 => 2021, 0, 4096, 51.2;
    noisy_malus_base:  i32 => 131, 0, 256, 3.2;
    noisy_malus_scale: i32 => 131, 0, 256, 3.2;
    noisy_malus_max:   i32 => 2025, 0, 4096, 51.2;

    pawn_bonus_base:  i32 => 134, 0, 256, 3.2;
    pawn_bonus_scale: i32 => 129, 0, 256, 3.2;
    pawn_bonus_max:   i32 => 2042, 0, 4096, 51.2;
    pawn_malus_base:  i32 => 131, 0, 256, 3.2;
    pawn_malus_scale: i32 => 122, 0, 256, 3.2;
    pawn_malus_max:   i32 => 2041, 0, 4096, 51.2;

    duck_bonus_base:  i32 => 129, 0, 256, 3.2;
    duck_bonus_scale: i32 => 132, 0, 256, 3.2;
    duck_bonus_max:   i32 => 1985, 0, 4096, 51.2;
    duck_malus_base:  i32 => 126, 0, 256, 3.2;
    duck_malus_scale: i32 => 128, 0, 256, 3.2;
    duck_malus_max:   i32 => 2016, 0, 4096, 51.2;

    quiet_duck_bonus_base:  i32 => 126, 0, 256, 3.2;
    quiet_duck_bonus_scale: i32 => 124, 0, 256, 3.2;
    quiet_duck_bonus_max:   i32 => 2025, 0, 4096, 51.2;
    quiet_duck_malus_base:  i32 => 129, 0, 256, 3.2;
    quiet_duck_malus_scale: i32 => 119, 0, 256, 3.2;
    quiet_duck_malus_max:   i32 => 2061, 0, 4096, 51.2;

    cont1_bonus_base:  i32 => 125, 0, 256, 3.2;
    cont1_bonus_scale: i32 => 127, 0, 256, 3.2;
    cont1_bonus_max:   i32 => 2010, 0, 4096, 51.2;
    cont1_malus_base:  i32 => 124, 0, 256, 3.2;
    cont1_malus_scale: i32 => 129, 0, 256, 3.2;
    cont1_malus_max:   i32 => 2001, 0, 4096, 51.2;

    cont2_bonus_base:  i32 => 130, 0, 256, 3.2;
    cont2_bonus_scale: i32 => 125, 0, 256, 3.2;
    cont2_bonus_max:   i32 => 2062, 0, 4096, 51.2;
    cont2_malus_base:  i32 => 129, 0, 256, 3.2;
    cont2_malus_scale: i32 => 124, 0, 256, 3.2;
    cont2_malus_max:   i32 => 2065, 0, 4096, 51.2;

    cont4_bonus_base:  i32 => 129, 0, 256, 3.2;
    cont4_bonus_scale: i32 => 132, 0, 256, 3.2;
    cont4_bonus_max:   i32 => 2099, 0, 4096, 51.2;
    cont4_malus_base:  i32 => 123, 0, 256, 3.2;
    cont4_malus_scale: i32 => 124, 0, 256, 3.2;
    cont4_malus_max:   i32 => 2032, 0, 4096, 51.2;

    rfp_base:      i32 => -20, -100, 100, 10;
    rfp_scale:     i32 => 51, 0, 100, 1.3;
    rfp_imp_base:  i32 => -51, -100, 0, 1.3;
    rfp_imp_scale: i32 => 51, 0, 100, 1.3;
    rfp_lerp:      i32 => 496, 0, 1024, 12.8;

    razor_base:  i32 => 318, 0, 640, 8;
    razor_scale: i32 => 265, 0, 500, 6.3;

    nmr_margin:          i32 => 20, 0, 40, 0.5;
    nmr_reduction_base:  i32 => 5113, 1024, 10240, 128;
    nmr_reduction_scale: i32 => 337, 0, 682, 8.6;

    iid_depth_scale:     i32 => 785, 384, 1024, 19.2;
    iid_depth_reduction: i32 => 1530, 1024, 3072, 38.4;

    mvv_pawn:   i32 => 101, 50, 200, 2.5;
    mvv_knight: i32 => 308, 160, 640, 8;
    mvv_bishop: i32 => 323, 165, 660, 8.3;
    mvv_rook:   i32 => 512, 250, 1000, 12.5;
    mvv_queen:  i32 => 916, 450, 1800, 22.5;

    see_pawn:   i32 => 103, 50, 200, 2.5;
    see_knight: i32 => 323, 160, 640, 8;
    see_bishop: i32 => 319, 165, 660, 8.3;
    see_rook:   i32 => 499, 250, 1000, 12.5;
    see_queen:  i32 => 933, 450, 1800, 22.5;

    quiet_hp_base:  i32 => 0, -4000, 4000, 10;
    quiet_hp_scale: i32 => -2437, -5000, 0, 62.5;

    quiet_ldp_imp_threshold_base:  i32 => 2307, 0, 4096, 102.4;
    quiet_ldp_imp_threshold_scale: i32 => 2108, 0, 4096, 102.4;
    quiet_ldp_threshold_base:      i32 => 1072, 0, 2048, 51.2;
    quiet_ldp_threshold_scale:     i32 => 1079, 0, 2048, 51.2;
    quiet_ldp_history_offset:      i32 => -3981, -8000, 0, 100;
    quiet_ldp_history_div:         i32 => 3884, 2000, 8000, 100;
    quiet_ldp_history_min:         i32 => -2111, -4096, 0, 102.4;
    quiet_ldp_history_max:         i32 => 2211, 0, 4096, 102.4;

    noisy_ldp_imp_threshold_base:  i32 => 4165, 0, 8192, 102.4;
    noisy_ldp_imp_threshold_scale: i32 => 4189, 0, 8192, 102.4;
    noisy_ldp_threshold_base:      i32 => 4026, 0, 8192, 102.4;
    noisy_ldp_threshold_scale:     i32 => 4094, 0, 8192, 102.4;

    dcp_threshold_imp_base:  i32 => 2107, 0, 4096, 102.4;
    dcp_threshold_imp_scale: i32 => 1050, 0, 2048, 51.2;
    dcp_threshold_base:      i32 => 4105, 0, 8192, 102.4;
    dcp_threshold_scale:     i32 => 2168, 0, 4096, 102.4;
    dcp_history_offset:      i32 => -3921, -8000, 0, 100;
    dcp_history_div:         i32 => 4001, 2000, 8000, 100;
    dcp_history_min:         i32 => -2101, -4096, 0, 102.4;
    dcp_history_max:         i32 => 2037, 0, 4096, 102.4;

    udp_threshold_base:  i32 => 10349, 0, 20480, 307.2;
    udp_threshold_scale: i32 => 4043, 0, 8192, 102.4;
    udp_history_offset:  i32 => -3892, -8000, 0, 100;
    udp_history_div:     i32 => 3964, 2000, 8000, 100;
    udp_history_min:     i32 => -2119, -4096, 0, 102.4;
    udp_history_max:     i32 => 2112, 0, 4096, 102.4;

    ump_threshold_base:         i64 => 4223, 0, 8192, 102.4;
    ump_threshold_numerator:    i64 => 3042, 0, 6144, 102.4;
    ump_threshold_denominator:  i64 => 1948, 1024, 4096, 102.4;

    quiet_see_base:  i32 => 2, -100, 100, 10;
    quiet_see_scale: i32 => -78, -160, 0, 2;
    noisy_see_base:  i32 => -6, -100, 100, 10;
    noisy_see_scale: i32 => -77, -160, 0, 2;

    se_beta:       i32 => 127, 0, 256, 3.2;
    se_depth_lerp: i32 => 518, 256, 768, 12.8;

    mp_see_threshold: i32 => -21, -100, 100, 10;
    mp_qs_see_threshold: i32 => 3, -100, 100, 10;
    mp_quiet_neutral_malus: i32 => 4874, 0, 10000, 125;

    asp_delta:       i32 => 20, 1, 40, 0.5;
    asp_beta_lerp:   i32 => 533, 0, 1024, 12.8;
    asp_widen_scale: i32 => 129, 64, 256, 3.2;

    soft_time_div: u64 => 101793, 49152, 196608, 2457.6;
    soft_time_inc: u64 => 1981, 0, 4096, 51.2;
    hard_time_div: u64 => 11987, 6144, 24576, 307.2;
    hard_time_inc: u64 => 4247, 0, 8192, 102.4;

    duck_stability_base:  u128 => 5582, 2662, 10650, 133.2;
    duck_stability_scale: u128 => 423, 0, 820, 10.3;
    duck_stability_min:   u128 => 2730, 1433, 5734, 71.7;

    move_stability_base:  u128 => 5346, 2662, 10650, 133.2;
    move_stability_scale: u128 => 396, 0, 820, 10.3;
    move_stability_min:   u128 => 2864, 1433, 5734, 71.7;

    quiet_lmr_base:  i32 => 1025, 0, 2048, 25.6;
    quiet_lmr_scale: i32 => 96, 0, 192, 2.4;
    lmr_exact:       i32 => 984, 0, 2048, 25.6;
    lmr_imp:         i32 => 1019, 0, 2048, 25.6;
    lmr_pv:          i32 => 1051, 0, 2048, 25.6;
    lmr_in_check:    i32 => 526, 0, 1024, 12.8;
    lmr_history:     i32 => 65, 0, 128, 1.6;
    lmr_corr:        i32 => 3052, 0, 6144, 76.8;

    fp_base:  i32 => 255, 0, 512, 6.4;
    fp_scale: i32 => 129, 0, 256, 3.2;

    noisy_lmr_noisy_scale: i32 => 127, 0, 256, 3.2;
    noisy_lmr_duck_scale:  i32 => 129, 0, 256, 3.2;

    quiet_lmr_quiet_scale: i32 => 1032, 0, 2048, 25.6;
    quiet_lmr_duck_scale:  i32 => 1030, 0, 2048, 25.6;
    quiet_lmr_cont1_scale: i32 => 1015, 0, 2048, 25.6;
    quiet_lmr_cont2_scale: i32 => 1005, 0, 2048, 25.6;

    quiet_mp_quiet_scale:       i32 => 1003, 0, 2048, 25.6;
    quiet_mp_duck_scale:        i32 => 1026, 0, 2048, 25.6;
    quiet_mp_pawn_scale:        i32 => 1064, 0, 2048, 25.6;
    quiet_mp_quiet_duck_scale:  i32 => 1040, 0, 2048, 25.6;
    quiet_mp_cont1_scale:       i32 => 1003, 0, 2048, 25.6;
    quiet_mp_cont2_scale:       i32 => 1037, 0, 2048, 25.6;
    quiet_mp_cont4_scale:       i32 => 1021, 0, 2048, 25.6;

    noisy_mp_noisy_scale: i32 => 131, 0, 256, 3.2;
    noisy_mp_duck_scale:  i32 => 126, 0, 256, 3.2;

    quiet_hp_quiet_scale: i32 => 1060, 0, 2048, 25.6;
    quiet_hp_duck_scale:  i32 => 1031, 0, 2048, 25.6;
    quiet_hp_cont1_scale: i32 => 1020, 0, 2048, 25.6;
    quiet_hp_cont2_scale: i32 => 1062, 0, 2048, 25.6;
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
    pub fn quiet_duck_bonus(depth: i32) -> i32 {
        (Self::quiet_duck_bonus_base() + Self::quiet_duck_bonus_scale() * depth)
            .min(Self::quiet_duck_bonus_max())
    }

    #[inline]
    pub fn quiet_duck_malus(depth: i32) -> i32 {
        -(Self::quiet_duck_malus_base() + Self::quiet_duck_malus_scale() * depth)
            .min(Self::quiet_duck_malus_max())
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
    pub const fn quiet_hp_margin(depth: i32) -> i32 {
        Self::quiet_hp_base() + Self::quiet_hp_scale() * depth * depth
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
            threshold += Self::history_adjustment(
                duck_history,
                Params::quiet_ldp_history_offset(),
                Params::quiet_ldp_history_div(),
                Params::quiet_ldp_history_min(),
                Params::quiet_ldp_history_max(),
            );
        }
        threshold / 1024
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
        threshold += Self::history_adjustment(
            duck_history,
            Params::dcp_history_offset(),
            Params::dcp_history_div(),
            Params::dcp_history_min(),
            Params::dcp_history_max(),
        );

        threshold / 1024
    }

    #[inline]
    pub fn udp_threshold(depth: i32, duck_history: i32) -> i32 {
        let mut threshold = Self::udp_threshold_base() + Self::udp_threshold_scale() * depth;
        threshold += Self::history_adjustment(
            duck_history,
            Self::udp_history_offset(),
            Self::udp_history_div(),
            Self::udp_history_min(),
            Self::udp_history_max(),
        );

        threshold / 1024
    }

    #[inline]
    pub fn ump_threshold(depth: i32) -> i32 {
        let depth = depth as i64;

        let mut threshold = Self::ump_threshold_base();
        threshold += Self::ump_threshold_numerator() * depth * depth * 1024
            / Self::ump_threshold_denominator();

        (threshold / 1024) as i32
    }

    #[inline]
    pub fn see_margin(depth: i32, is_quiet: bool) -> i32 {
        if is_quiet {
            Self::quiet_see_base() + Self::quiet_see_scale() * depth
        } else {
            Self::noisy_see_base() + Self::noisy_see_scale() * depth
        }
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
    pub fn lmr(depth: i32, unique_moves: i32) -> i32 {
        let log_depth = depth.ilog2() as i32;
        let log_moves = (unique_moves + 1).ilog2() as i32;

        Self::quiet_lmr_base() + Self::quiet_lmr_scale() * log_depth * log_moves
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
    pub fn quiet_lmr_history(
        thread: &ThreadData,
        pos: &Position,
        indices: ContIndices,
        mv: Move,
    ) -> i32 {
        let board = pos.board();
        let mut history = 0;

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
        history_score += history.quiet_duck(board, mv) * Self::quiet_mp_quiet_duck_scale();
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
    pub fn quiet_hp_history(
        thread: &ThreadData,
        pos: &Position,
        indices: ContIndices,
        mv: Move,
    ) -> i32 {
        let board = pos.board();
        let mut history = 0;

        history += thread.history.quiet(board, mv) * Self::quiet_hp_quiet_scale();
        history += thread.history.duck(board, mv) * Self::quiet_hp_duck_scale();
        history += thread.history.cont1(board, indices, mv) * Self::quiet_hp_cont1_scale();
        history += thread.history.cont2(board, indices, mv) * Self::quiet_hp_cont2_scale();

        history / 1024
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

    #[inline]
    pub fn lerp(a: i32, b: i32, t: i32) -> i32 {
        let a = a as i64;
        let b = b as i64;
        let t = t as i64;

        ((a * (1024 - t) + b * t) / 1024) as i32
    }

    #[inline]
    fn history_adjustment(history: i32, offset: i32, divisor: i32, min: i32, max: i32) -> i32 {
        ((history + offset) * 1024 / divisor).clamp(min, max)
    }
}
