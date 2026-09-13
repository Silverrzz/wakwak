use crate::common::Color;
use crate::engine::EngineOptions;
use crate::search::{MAX_DEPTH, ThreadData};
use crate::uci::SearchLimit;
use crate::util::AtomicInstant;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const DEFAULT_OVERHEAD: u64 = 50;

pub struct TimeManager {
    start: AtomicInstant,
    infinite: AtomicBool,    // Search lasts indefinitely
    manage_time: AtomicBool, // Manage the soft time limit every depth
    check_time: AtomicBool,  // Check the hard time limit every 1024 nodes
    stop: AtomicU32,

    base_time: AtomicU64,
    soft_time: AtomicU64,
    hard_time: AtomicU64,

    soft_nodes: AtomicU64,
    hard_nodes: AtomicU64,
    depth: AtomicU8,
}

impl TimeManager {
    #[inline]
    pub fn init(&self, stm: Color, limits: &[SearchLimit], options: EngineOptions) {
        use SearchLimit::*;

        let mut inc = [0; Color::COUNT];
        let mut time = [u64::MAX; Color::COUNT];
        let mut move_time = None;
        let mut nodes = u64::MAX;
        let mut depth = MAX_DEPTH;
        let mut infinite = true;
        let mut manage_time = true;
        let mut check_time = false;

        for &limit in limits {
            match limit {
                WhiteTime(t) => time[Color::White] = t,
                BlackTime(t) => time[Color::Black] = t,
                WhiteInc(i) => inc[Color::White] = i,
                BlackInc(i) => inc[Color::Black] = i,
                MoveTime(t) => move_time = Some(t),
                Nodes(n) => nodes = nodes.min(n),
                Depth(d) => depth = depth.min(d),
            }

            infinite = false;
            if matches!(limit, WhiteTime(_) | BlackTime(_) | MoveTime(_)) {
                check_time = true;
            }

            if let MoveTime(_) = limit {
                manage_time = false;
            }
        }

        self.set_stop(false);
        self.infinite.store(infinite, Ordering::Relaxed);
        self.manage_time.store(manage_time, Ordering::Relaxed);
        self.check_time.store(check_time, Ordering::Relaxed);

        self.depth.store(depth, Ordering::Relaxed);
        self.soft_nodes.store(nodes, Ordering::Relaxed);

        if options.soft_target {
            self.hard_nodes.store(nodes * 2000, Ordering::Relaxed);
        } else {
            self.hard_nodes.store(nodes, Ordering::Relaxed);
        }

        if let Some(time) = move_time {
            self.base_time.store(time, Ordering::Relaxed);
            self.soft_time.store(time, Ordering::Relaxed);

            if options.soft_target {
                self.hard_time
                    .store((time * 2).max(time + 2000), Ordering::Relaxed);
            } else {
                self.hard_time.store(time, Ordering::Relaxed);
            }
        } else {
            let (time, inc) = (time[stm].saturating_sub(options.overhead), inc[stm]);
            let hard_time = (time / 3 + inc).min(time);
            let soft_time = (time / 20 + inc / 2).min(hard_time);

            self.base_time.store(soft_time, Ordering::Relaxed);
            self.soft_time.store(soft_time, Ordering::Relaxed);
            self.hard_time.store(hard_time, Ordering::Relaxed);
        }

        self.start.store(Instant::now(), Ordering::Relaxed);
    }

    #[inline]
    pub fn deepen(&self, depth: u8) {
        if depth < 4 || !self.manage_time.load(Ordering::Relaxed) {
            #[allow(clippy::needless_return)]
            return;
        }

        //This is where most TM patches will be implemented.
    }

    #[inline]
    pub fn set_stop(&self, stop: bool) {
        self.stop.store(stop as u32, Ordering::Relaxed);

        if self.infinite() {
            atomic_wait::wake_all(&self.stop);
        }
    }

    #[inline]
    pub fn wait_for_stop(&self) {
        while !self.stop() {
            atomic_wait::wait(&self.stop, 0);
        }
    }

    #[inline]
    pub fn stop_search(&self, thread: &ThreadData) -> bool {
        self.stop()
            || thread.nodes.global() >= self.hard_nodes.load(Ordering::Relaxed)
            || (thread.nodes.local().is_multiple_of(1024)
                && self.check_time.load(Ordering::Relaxed)
                && self.elapsed().as_millis() as u64 > self.hard_time.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn stop_id(&self, depth: u8, nodes: u64) -> bool {
        self.stop()
            || depth >= self.depth.load(Ordering::Relaxed)
            || nodes >= self.soft_nodes.load(Ordering::Relaxed)
            || (self.check_time.load(Ordering::Relaxed)
                && self.elapsed().as_millis() as u64 > self.soft_time.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.load(Ordering::Relaxed).elapsed()
    }

    #[inline]
    pub fn infinite(&self) -> bool {
        self.infinite.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn stop(&self) -> bool {
        self.stop.load(Ordering::Relaxed) != 0
    }
}

impl Default for TimeManager {
    #[inline]
    fn default() -> Self {
        Self {
            start: AtomicInstant::now(),
            infinite: AtomicBool::new(true),
            manage_time: AtomicBool::new(true),
            check_time: AtomicBool::new(false),
            stop: AtomicU32::new(0),

            base_time: AtomicU64::new(0),
            soft_time: AtomicU64::new(0),
            hard_time: AtomicU64::new(0),

            soft_nodes: AtomicU64::new(u64::MAX),
            hard_nodes: AtomicU64::new(u64::MAX),
            depth: AtomicU8::new(MAX_DEPTH),
        }
    }
}
