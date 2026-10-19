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
    minor_corr:       i32 => 65;
    major_corr:       i32 => 62;
    nonpawn_corr:     i32 => 66;
    cont1_corr:       i32 => 62;
    cont2_corr:       i32 => 64;
    corr_bonus_scale: i64 => 129;

    quiet_bonus_base:  i32 => 128;
    quiet_bonus_scale: i32 => 126;
    quiet_bonus_max:   i32 => 2152;
    quiet_malus_base:  i32 => 132;
    quiet_malus_scale: i32 => 130;
    quiet_malus_max:   i32 => 2076;

    noisy_bonus_base:  i32 => 127;
    noisy_bonus_scale: i32 => 131;
    noisy_bonus_max:   i32 => 2136;
    noisy_malus_base:  i32 => 126;
    noisy_malus_scale: i32 => 120;
    noisy_malus_max:   i32 => 2069;

    pawn_bonus_base:  i32 => 125;
    pawn_bonus_scale: i32 => 128;
    pawn_bonus_max:   i32 => 2095;
    pawn_malus_base:  i32 => 127;
    pawn_malus_scale: i32 => 135;
    pawn_malus_max:   i32 => 2063;

    duck_bonus_base:  i32 => 129;
    duck_bonus_scale: i32 => 128;
    duck_bonus_max:   i32 => 2012;
    duck_malus_base:  i32 => 127;
    duck_malus_scale: i32 => 125;
    duck_malus_max:   i32 => 2143;

    quiet_duck_bonus_base:  i32 => 123;
    quiet_duck_bonus_scale: i32 => 127;
    quiet_duck_bonus_max:   i32 => 2053;
    quiet_duck_malus_base:  i32 => 128;
    quiet_duck_malus_scale: i32 => 125;
    quiet_duck_malus_max:   i32 => 2082;

    cont1_bonus_base:  i32 => 131;
    cont1_bonus_scale: i32 => 129;
    cont1_bonus_max:   i32 => 2013;
    cont1_malus_base:  i32 => 139;
    cont1_malus_scale: i32 => 126;
    cont1_malus_max:   i32 => 2024;

    cont2_bonus_base:  i32 => 127;
    cont2_bonus_scale: i32 => 120;
    cont2_bonus_max:   i32 => 2031;
    cont2_malus_base:  i32 => 127;
    cont2_malus_scale: i32 => 123;
    cont2_malus_max:   i32 => 2123;

    cont4_bonus_base:  i32 => 128;
    cont4_bonus_scale: i32 => 127;
    cont4_bonus_max:   i32 => 2026;
    cont4_malus_base:  i32 => 124;
    cont4_malus_scale: i32 => 131;
    cont4_malus_max:   i32 => 2003;

    hindsight_ext_red:  i32 => 3066;

    rfp_base:       i32 => -5;
    rfp_scale:      i32 => 47;
    rfp_imp_base:   i32 => -42;
    rfp_imp_scale:  i32 => 50;
    rfp_corr_scale: i32 => 243;
    rfp_lerp:       i32 => 500;

    razor_base:  i32 => 317;
    razor_scale: i32 => 253;

    nmr_margin:          i32 => 19;
    nmr_reduction_base:  i32 => 5098;
    nmr_reduction_scale: i32 => 355;

    iid_depth_scale:     i32 => 733;
    iid_depth_reduction: i32 => 1539;

    mvv_pawn:   i32 => 94;
    mvv_knight: i32 => 328;
    mvv_bishop: i32 => 333;
    mvv_rook:   i32 => 501;
    mvv_queen:  i32 => 906;

    see_pawn:   i32 => 102;
    see_knight: i32 => 331;
    see_bishop: i32 => 331;
    see_rook:   i32 => 501;
    see_queen:  i32 => 908;

    quiet_hp_base:        i32 => -59;
    quiet_hp_scale:       i32 => -2508;
    quiet_hp_quiet_scale: i32 => 1025;
    quiet_hp_duck_scale:  i32 => 949;
    quiet_hp_cont1_scale: i32 => 980;
    quiet_hp_cont2_scale: i32 => 1080;

    quiet_ldp_imp_threshold_base:  i32 => 2110;
    quiet_ldp_imp_threshold_scale: i32 => 2050;
    quiet_ldp_threshold_base:      i32 => 1040;
    quiet_ldp_threshold_scale:     i32 => 1125;
    quiet_ldp_history_offset:      i32 => -4003;
    quiet_ldp_history_div:         i32 => 4114;
    quiet_ldp_history_min:         i32 => -2058;
    quiet_ldp_history_max:         i32 => 2043;

    noisy_ldp_imp_threshold_base:  i32 => 4111;
    noisy_ldp_imp_threshold_scale: i32 => 4054;
    noisy_ldp_threshold_base:      i32 => 4199;
    noisy_ldp_threshold_scale:     i32 => 3938;

    dcp_threshold_imp_base:  i32 => 2031;
    dcp_threshold_imp_scale: i32 => 1040;
    dcp_threshold_base:      i32 => 4132;
    dcp_threshold_scale:     i32 => 2076;
    dcp_history_offset:      i32 => -3844;
    dcp_history_div:         i32 => 3973;
    dcp_history_min:         i32 => -2211;
    dcp_history_max:         i32 => 1923;

    udp_threshold_base:  i32 => 10571;
    udp_threshold_scale: i32 => 3980;
    udp_history_offset:  i32 => -4228;
    udp_history_div:     i32 => 4129;
    udp_history_min:     i32 => -2062;
    udp_history_max:     i32 => 2157;

    ump_threshold_base:  i32 => 3994;
    ump_threshold_scale: i32 => 1412;

    quiet_see_base:             i32 => 10;
    quiet_see_scale:            i32 => -73;
    noisy_see_base:             i32 => -7;
    noisy_see_scale:            i32 => -73;
    noisy_see_hist_offset:      i32 => 42;
    noisy_see_hist_div:         i32 => 65;
    noisy_see_hist_min:         i32 => -521;
    noisy_see_hist_max:         i32 => 551;
    noisy_see_hist_noisy_scale: i32 => 976;
    noisy_see_hist_duck_scale:  i32 => 975;

    se_beta:       i32 => 126;
    se_depth_lerp: i32 => 528;
    se_dext:       i32 => 61;

    mp_see_threshold:       i32 => -28;
    mp_qs_see_threshold:    i32 => 9;
    mp_quiet_neutral_malus: i32 => 5079;

    asp_delta:       i32 => 21;
    asp_beta_lerp:   i32 => 515;
    asp_widen_scale: i32 => 127;

    soft_time_div: u64 => 101087;
    soft_time_inc: u64 => 2015;
    hard_time_div: u64 => 12271;
    hard_time_inc: u64 => 4062;

    duck_stability_base:  u128 => 5210;
    duck_stability_scale: u128 => 477;
    duck_stability_min:   u128 => 2839;

    move_stability_base:  u128 => 5470;
    move_stability_scale: u128 => 505;
    move_stability_min:   u128 => 2881;

    quiet_lmr_base:  i32 => 1011;
    quiet_lmr_scale: i32 => 97;
    lmr_pv:          i32 => 1030;
    lmr_imp:         i32 => 1053;
    lmr_exact:       i32 => 1034;
    lmr_in_check:    i32 => 1057;
    lmr_history:     i32 => 67;
    lmr_corr:        i32 => 3018;
    lmr_duck_count:  i32 => 1024;

    fp_base:             i32 => 99;
    fp_scale:            i32 => 93;
    fp_hist_offset:      i32 => 3956;
    fp_hist_div:         i32 => 61;
    fp_hist_min:         i32 => -419;
    fp_hist_max:         i32 => 359;
    fp_hist_quiet_scale: i32 => 1087;
    fp_hist_duck_scale:  i32 => 1125;
    fp_hist_cont1_scale: i32 => 980;
    fp_hist_cont2_scale: i32 => 1110;

    noisy_lmr_noisy_scale: i32 => 136;
    noisy_lmr_duck_scale:  i32 => 109;

    quiet_lmr_quiet_scale: i32 => 1057;
    quiet_lmr_duck_scale:  i32 => 917;
    quiet_lmr_cont1_scale: i32 => 1001;
    quiet_lmr_cont2_scale: i32 => 1010;

    quiet_mp_quiet_scale:       i32 => 986;
    quiet_mp_duck_scale:        i32 => 982;
    quiet_mp_pawn_scale:        i32 => 1024;
    quiet_mp_quiet_duck_scale:  i32 => 946;
    quiet_mp_cont1_scale:       i32 => 1012;
    quiet_mp_cont2_scale:       i32 => 1055;
    quiet_mp_cont4_scale:       i32 => 1035;

    noisy_mp_noisy_scale: i32 => 144;
    noisy_mp_duck_scale:  i32 => 146;

    duck_count_hist_scale: i32 => 32;
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
    pub const fn rfp_margin(depth: i32, corr: i32, improving: bool) -> i32 {
        let (base, scale) = if improving {
            (Self::rfp_imp_base(), Self::rfp_imp_scale())
        } else {
            (Self::rfp_base(), Self::rfp_scale())
        };

        base + scale * depth + Self::rfp_corr_scale() * corr / 1024
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
            threshold += Self::frac_hist_adjustment(
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
        threshold += Self::frac_hist_adjustment(
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
        threshold += Self::frac_hist_adjustment(
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
        let threshold = Self::ump_threshold_base() + Self::ump_threshold_scale() * depth * depth;
        threshold / 1024
    }

    #[inline]
    pub fn see_margin(depth: i32, history: i32, is_quiet: bool) -> i32 {
        if is_quiet {
            Self::quiet_see_base() + Self::quiet_see_scale() * depth
        } else {
            Self::noisy_see_base() + Self::noisy_see_scale() * depth
                - Self::history_adjustment(
                    history,
                    Self::noisy_see_hist_offset(),
                    Self::noisy_see_hist_div(),
                    Self::noisy_see_hist_min(),
                    Self::noisy_see_hist_max(),
                )
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
    pub fn see_history(
        thread: &ThreadData,
        pos: &Position,
        _indices: ContIndices,
        mv: Move,
    ) -> i32 {
        let board = pos.board();
        let mut history = 0;

        if mv.flag().is_noisy() {
            history += thread.history.noisy(board, mv) * Self::noisy_see_hist_noisy_scale();
            history += thread.history.duck(board, mv) * Self::noisy_see_hist_duck_scale();
        }

        history / 1024
    }

    #[inline]
    pub fn fp_history(thread: &ThreadData, pos: &Position, indices: ContIndices, mv: Move) -> i32 {
        let board = pos.board();
        let mut history = 0;

        history += thread.history.quiet(board, mv) * Self::fp_hist_quiet_scale();
        history += thread.history.duck(board, mv) * Self::fp_hist_duck_scale();
        history += thread.history.cont1(board, indices, mv) * Self::fp_hist_cont1_scale();
        history += thread.history.cont2(board, indices, mv) * Self::fp_hist_cont2_scale();

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
    pub fn fp_margin(depth: i32, history: i32) -> i32 {
        Self::fp_base()
            + Self::fp_scale() * depth
            + Self::history_adjustment(
                history,
                Self::fp_hist_offset(),
                Self::fp_hist_div(),
                Self::fp_hist_min(),
                Self::fp_hist_max(),
            )
    }

    #[inline]
    pub fn lerp(a: i32, b: i32, t: i32) -> i32 {
        let a = a as i64;
        let b = b as i64;
        let t = t as i64;

        ((a * (1024 - t) + b * t) / 1024) as i32
    }

    #[inline]
    fn frac_hist_adjustment(history: i32, offset: i32, divisor: i32, min: i32, max: i32) -> i32 {
        ((history + offset) * 1024 / divisor).clamp(min, max)
    }

    #[inline]
    fn history_adjustment(history: i32, offset: i32, divisor: i32, min: i32, max: i32) -> i32 {
        ((history + offset) / divisor).clamp(min, max)
    }
}
