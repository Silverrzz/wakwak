#![allow(dead_code)]

use std::sync::Mutex;
#[cfg(feature = "stat")]
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

#[cfg(feature = "stat")]
use dashmap::DashMap;

#[macro_export]
macro_rules! debug_hit {
    ($key:expr, $value:expr) => {
        #[cfg(feature = "stat")]
        {
            $crate::tools::stat::stat_map().add($key.to_string(), $value as i64, true);
        }
    };
}

#[macro_export]
macro_rules! debug_stat {
    ($key:expr, $value:expr) => {
        #[cfg(feature = "stat")]
        {
            $crate::tools::stat::stat_map().add($key.to_string(), $value as i64, false);
        }
    };
}

#[cfg(feature = "stat")]
static STAT_MAP: OnceLock<StatMap> = OnceLock::new();

#[cfg(feature = "stat")]
pub fn stat_map() -> &'static StatMap {
    STAT_MAP.get_or_init(StatMap::default)
}

#[cfg(feature = "stat")]
pub struct StatMap {
    map: DashMap<String, Entry>,
}

#[cfg(feature = "stat")]
impl StatMap {
    #[inline]
    pub fn add(&self, key: String, value: i64, hit: bool) {
        let entry = self.map.entry(key).or_default();
        entry.hit.store(hit, Ordering::Relaxed);
        entry.add(value);
        entry.inc();
    }

    #[inline]
    pub fn clear(&self) {
        self.map.clear();
    }

    #[inline]
    pub fn print(&self) {
        self.map.iter().for_each(|r| {
            print!("{}: ", r.key());
            r.value().print();
        });
    }
}

#[cfg(feature = "stat")]
impl Default for StatMap {
    #[inline]
    fn default() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

#[derive(Default)]
struct Entry {
    count: AtomicU64,
    sum: AtomicI64,
    sum_sqr: AtomicU64,
    values: Mutex<Vec<i64>>,
    hit: AtomicBool,
}

impl Entry {
    #[inline]
    pub fn inc(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn add(&self, value: i64) {
        self.sum.fetch_add(value, Ordering::Relaxed);
        self.sum_sqr
            .fetch_add((value * value) as u64, Ordering::Relaxed);
        self.values.lock().unwrap().push(value);
    }

    #[inline]
    pub fn print(&self) {
        let count = self.count();
        if count == 0 {
            return;
        }

        if self.hit.load(Ordering::Relaxed) {
            let sum = self.sum();
            let rate = sum as f64 / count as f64 * 100.0;

            println!("Total {count}, Hits {sum}, Hit Rate (%) {rate:.3}");
        } else {
            let mean = self.mean();
            let median = self.median();
            let std_dev = self.std_dev();
            let min = self.min();
            let max = self.max();

            println!(
                "Total {count}, Mean {mean:.3}, Median {median:.3}, StdDev {std_dev:.3}, Min {min:.3} Max {max:.3}"
            );
        }
    }

    #[inline]
    pub fn variance(&self) -> f64 {
        if self.count() == 0 {
            return 0.0;
        }

        let sum = self.sum() as f64;
        let sum_sqr = self.sum_sqr() as f64;
        let count = self.count() as f64;

        (sum_sqr - sum * sum / count) / count
    }

    #[inline]
    pub fn median(&self) -> f64 {
        let mut values = self.values.lock().unwrap();
        values.sort_unstable();

        // Naive median finding algorithm
        match values.len() {
            0 => 0.0,
            n if n % 2 == 1 => values[n / 2] as f64,
            n if n % 2 == 0 => (values[n / 2 - 1] + values[n / 2]) as f64 / 2.0,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn mean(&self) -> f64 {
        if self.count() == 0 {
            return 0.0;
        }

        self.sum() as f64 / self.count() as f64
    }

    #[inline]
    pub fn min(&self) -> f64 {
        let values = self.values.lock().unwrap();
        values.iter().min().map(|&x| x as f64).unwrap_or(0.0)
    }

    #[inline]
    pub fn max(&self) -> f64 {
        let values = self.values.lock().unwrap();
        values.iter().max().map(|&x| x as f64).unwrap_or(0.0)
    }

    #[inline]
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    #[inline]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn sum(&self) -> i64 {
        self.sum.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn sum_sqr(&self) -> u64 {
        self.sum_sqr.load(Ordering::Relaxed)
    }
}
