use super::{Result, ensure};
use crate::board::SplitMix64;
use serde::Deserialize;
use std::path::{Path, PathBuf};

macro_rules! settings {
    ($name:ident { $($field:ident: $ty:ty = $default:expr),* $(,)? }) => {
        #[derive(Clone, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }

        impl Default for $name {
            fn default() -> Self { Self { $($field: $default),* } }
        }
    };
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: PartialOrd + Copy> Range<T> {
    pub fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn validate(&self) -> Result<()> {
        ensure(self.min <= self.max, "range min must not exceed max")
    }
}

impl Range<usize> {
    pub fn sample(&self, rng: &mut SplitMix64) -> usize {
        self.min + (rng.next() % (self.max - self.min + 1) as u64) as usize
    }
}

settings! {
    Config {
        count: usize = 1000,
        seed: u64 = 1,
        max_candidates: u64 = 100000,
        max_seconds: Option<u64> = None,
        hash_mb: usize = 16,
        threads: u32 = 1,
        workers: usize = 0,
        source: Source = Source::default(),
        sources: Vec<Source> = Vec::new(),
        walk: Walk = Walk::default(),
        filter: super::filter::Filter = Default::default(),
        verify: Verify = Verify::default(),
        diversity: Diversity = Diversity::default(),
        output: Output = Output::default(),
    }
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    Start,
    Fen,
    Epd,
    Pgn,
}

settings! {
    Source {
        kind: SourceKind = SourceKind::Start,
        path: Option<PathBuf> = None,
        weight: u32 = 1,
        white_ids: Range<usize> = Range { min: 0, max: 959 },
        black_ids: Range<usize> = Range { min: 0, max: 959 },
        shared_id: bool = false,
        plies: Range<usize> = Range { min: 0, max: 40 },
    }
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    Random,
    Best,
    #[default]
    Weighted,
    Mixed,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Grouping {
    #[default]
    Turn,
    Piece,
}

settings! {
    Walk {
        plies: Range<usize> = Range { min: 12, max: 24 },
        policy: Policy = Policy::Weighted,
        nodes: u64 = 20000,
        multipv: usize = 8,
        random_probability: f64 = 0.15,
        max_loss_cp: i32 = 80,
        temperature_cp: f64 = 30.0,
        duck_sampling: Grouping = Grouping::Piece,
    }
}

settings! {
    Verify {
        nodes: Vec<u64> = vec![20000, 200000],
        multipv: usize = 1,
        min_depth: usize = 3,
        max_cp_shift: Option<i32> = None,
        min_gap_cp: Option<i32> = None,
        allow_mate: bool = false,
        rollout_plies: usize = 0,
        rollout_nodes: u64 = 20000,
        max_rollout_cp_shift: Option<i32> = None,
    }
}

#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Diversity {
    pub per_start: Option<usize>,
    pub per_source: Option<usize>,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    Epd,
    Fen,
    Pgn,
}

settings! {
    Output {
        path: PathBuf = "book.epd".into(),
        format: Format = Format::Epd,
        chess960: bool = true,
    }
}

fn document(path: &Path, depth: usize) -> Result<toml::Value> {
    let path = path.canonicalize()?;
    ensure(depth < 16, "excessive recipe inheritance")?;
    let mut child: toml::Value = toml::from_str(&std::fs::read_to_string(&path)?)?;
    let parent = child
        .as_table_mut()
        .ok_or("recipe must be a TOML table")?
        .remove("extends");
    if let Some(parent) = parent {
        let parent = parent.as_str().ok_or("extends must be a path")?;
        let mut base = document(&path.parent().unwrap().join(parent), depth + 1)?;
        merge(&mut base, child);
        child = base;
    }
    Ok(child)
}

fn merge(base: &mut toml::Value, child: toml::Value) {
    if let (Some(base), Some(child)) = (base.as_table_mut(), child.as_table()) {
        for (key, value) in child {
            if let Some(base) = base.get_mut(key) {
                merge(base, value.clone());
            } else {
                base.insert(key.clone(), value.clone());
            }
        }
    } else {
        *base = child;
    }
}

impl Config {
    pub fn worker_count(&self) -> usize {
        if self.workers == 0 {
            std::thread::available_parallelism()
                .map_or(1, usize::from)
                .div_euclid(self.threads as usize)
                .clamp(1, 1024)
        } else {
            self.workers
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut config: Self = document(path, 0)?.try_into()?;
        let base = path.canonicalize()?.parent().unwrap().to_path_buf();
        if config.sources.is_empty() {
            config.sources.push(config.source.clone());
        }
        for source in &mut config.sources {
            source.white_ids.validate()?;
            source.black_ids.validate()?;
            source.plies.validate()?;
            ensure(
                source.white_ids.max < 960 && source.black_ids.max < 960 && source.weight > 0,
                "source needs valid starting IDs and a positive weight",
            )?;
            if source.shared_id {
                ensure(
                    source.white_ids.min.max(source.black_ids.min)
                        <= source.white_ids.max.min(source.black_ids.max),
                    "shared starting ID ranges must overlap",
                )?;
            }
            if source.kind != SourceKind::Start {
                source.path =
                    Some(base.join(source.path.as_ref().ok_or("file source needs path")?));
            }
        }
        config.walk.plies.validate()?;
        ensure(
            config.count > 0
                && config.max_candidates > 0
                && config.threads > 0
                && config.hash_mb > 0,
            "count, candidate limit, threads and hash must be positive",
        )?;
        ensure(
            config.walk.nodes > 0 && config.walk.multipv > 0 && config.walk.temperature_cp > 0.0,
            "walk budget, MultiPV and temperature must be positive",
        )?;
        ensure(
            !config.verify.nodes.is_empty()
                && config.verify.nodes.iter().all(|&nodes| nodes > 0)
                && config.verify.multipv > 0,
            "verification needs positive budgets and MultiPV",
        )?;
        config.output.path = base.join(&config.output.path);
        Ok(config)
    }
}
