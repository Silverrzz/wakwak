use super::config::Config;
use super::output;
use super::source::Input;
use super::{Record, Result, choose, ensure, identity, verify, walk};
use crate::board::SplitMix64;
use crate::search::Searcher;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

type Outcome = Option<Box<Record>>;

fn candidate(
    config: &Config,
    inputs: &[Input],
    attempt: u64,
    searcher: &mut Searcher,
    stopped: &AtomicBool,
) -> Outcome {
    let mut rng = SplitMix64::new(
        config
            .seed
            .wrapping_add(attempt.wrapping_mul(0x9e3779b97f4a7c15)),
    );
    let choices: Vec<_> = inputs
        .iter()
        .map(|source| (source, source.config.weight as f64))
        .collect();
    let input = choose(&mut rng, &choices)?;
    searcher.newgame();
    let candidate = walk(
        &config.walk,
        input.sample(&mut rng),
        searcher,
        &mut rng,
        stopped,
    )?;
    verify(config, &candidate, searcher, stopped)?;
    Some(Box::new(Record {
        board: *candidate.position.board(),
        initial_fen: candidate.initial_fen,
        moves: candidate.moves,
        source: candidate.source,
    }))
}

fn quota_keys(config: &Config, record: &Record) -> Vec<(String, usize)> {
    let mut keys = Vec::new();
    let limits = &config.diversity;
    if let Some(limit) = limits.per_start {
        keys.push((format!("start:{}", identity(&record.initial_fen)), limit));
    }
    if let Some(limit) = limits.per_source {
        keys.push((format!("source:{}", record.source), limit));
    }
    keys
}

pub fn run(config: &Config, inputs: &[Input]) -> Result<()> {
    ensure(
        !config.output.path.exists(),
        "output exists; choose a new output path",
    )?;
    let mut records = Vec::new();
    let mut attempts = 0;
    let mut seen = BTreeSet::new();
    let mut quotas = BTreeMap::<String, usize>::new();
    let workers = config.worker_count();
    let searchers: Vec<_> = (0..workers)
        .map(|_| {
            let mut searcher = Searcher::default();
            searcher.resize_tt(config.hash_mb);
            if config.threads != 1 {
                searcher.set_threads(config.threads);
            }
            searcher
        })
        .collect();
    let progress = ProgressBar::new(config.count as u64);
    progress.set_style(ProgressStyle::with_template(
        "  {prefix}\n  [{bar:28.cyan/blue}] {pos}/{len} ({percent}%)\n  Elapsed {elapsed_precise}   ETA {eta_precise}   {per_sec} positions/s\n  {msg}"
    )?.progress_chars("##-"));
    progress.set_prefix(
        config
            .output
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    );
    let shared: Vec<_> = searchers
        .iter()
        .map(|searcher| searcher.shared.clone())
        .collect();
    let stopped = AtomicBool::new(false);
    let active = AtomicUsize::new(0);
    let cancel = || {
        stopped.store(true, Ordering::Relaxed);
        for shared in &shared {
            shared.time_man.set_stop(true);
        }
    };
    let started = Instant::now();
    let next = AtomicU64::new(0);
    let result = std::thread::scope(|scope| -> Result<()> {
        let (send, results) = mpsc::sync_channel::<Outcome>(workers * 2);
        let mut handles = Vec::new();
        for mut searcher in searchers {
            let send = send.clone();
            let stopped = &stopped;
            let active = &active;
            let next = &next;
            handles.push(scope.spawn(move || {
                while !stopped.load(Ordering::Relaxed) {
                    let attempt = next.fetch_add(1, Ordering::Relaxed);
                    if attempt >= config.max_candidates {
                        break;
                    }
                    active.fetch_add(1, Ordering::Relaxed);
                    let outcome = candidate(config, inputs, attempt, &mut searcher, stopped);
                    active.fetch_sub(1, Ordering::Relaxed);
                    if send.send(outcome).is_err() {
                        break;
                    }
                }
                searcher.quit();
            }));
        }
        drop(send);
        while records.len() < config.count
            && attempts < config.max_candidates
            && config
                .max_seconds
                .is_none_or(|limit| started.elapsed().as_secs() < limit)
        {
            match results.recv_timeout(Duration::from_millis(200)) {
                Ok(outcome) => {
                    attempts += 1;
                    if let Some(record) = outcome {
                        let key = record.board.hash();
                        let keys = quota_keys(config, &record);
                        if !seen.contains(&key)
                            && keys
                                .iter()
                                .all(|(key, limit)| quotas.get(key).copied().unwrap_or(0) < *limit)
                        {
                            records.push(*record);
                            seen.insert(key);
                            for (key, _) in keys {
                                *quotas.entry(key).or_default() += 1;
                            }
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            progress.set_position(records.len() as u64);
            progress.set_message(format!(
                "{}/{} workers active | {attempts} candidates",
                active.load(Ordering::Relaxed),
                workers
            ));
        }
        cancel();
        drop(results);
        while handles.iter().any(|handle| !handle.is_finished()) {
            cancel();
            std::thread::sleep(Duration::from_millis(10));
        }
        for handle in handles {
            ensure(handle.join().is_ok(), "book generation worker panicked")?;
        }
        Ok(())
    });
    progress.set_message("saving book");
    let saved = output::write(config, &records);
    let result = result.and(saved).and_then(|()| {
        ensure(
            records.len() >= config.count,
            "generation stopped before filling the requested book; partial output saved",
        )
    });
    progress.finish_with_message(if result.is_ok() {
        "complete"
    } else {
        "stopped"
    });
    println!(
        "\n  Saved {} positions to {} ({} candidates).",
        records.len(),
        config.output.path.display(),
        attempts
    );
    result
}
