//! This was adapted directly from [Icarus](https://github.com/Sp00ph/icarus/).

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use crate::tagged_cell;

/// Wrapper type to atomically store and load `Instant`s. Internally, we store
/// in a `AtomicU64` the duration in nanoseconds since `EPOCH`. Note that this
/// breaks after running the program for ~584 years at a time.
pub struct AtomicInstant(AtomicU64);

tagged_cell! {
    /// The epoch used for `AtomicInstant`.
    static EPOCH: TaggedCell<Instant, pub EpochTag> = TaggedCell::new();
}

pub fn init_epoch() -> EpochTag {
    EPOCH.init(Instant::now)
}

fn instant_to_bits(instant: Instant, tag: EpochTag) -> u64 {
    (instant - *EPOCH.get(tag)).as_nanos().try_into().unwrap()
}

fn bits_to_instant(bits: u64, tag: EpochTag) -> Instant {
    *EPOCH.get(tag) + Duration::from_nanos(bits)
}

impl AtomicInstant {
    fn new(instant: Instant, tag: EpochTag) -> Self {
        Self(AtomicU64::new(instant_to_bits(instant, tag)))
    }

    pub fn now(tag: EpochTag) -> Self {
        Self::new(Instant::now(), tag)
    }

    pub fn load(&self, order: Ordering, tag: EpochTag) -> Instant {
        bits_to_instant(self.0.load(order), tag)
    }

    pub fn store(&self, instant: Instant, order: Ordering, tag: EpochTag) {
        self.0.store(instant_to_bits(instant, tag), order);
    }
}
