//! Gain records: per run of an agent-loop command, the bytes it printed against the raw bytes it
//! replaces. JSON Lines in `.specjutsu/ledger/gain.jsonl`, next to the evidence ledger.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::git::Git;

/// Record format version. Lines with any other `v` are skipped on load.
pub const FORMAT: u32 = 1;

/// One run of one command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub v: u32,
    /// `test`, `verify`, `next` or `status`.
    pub command: String,
    /// RFC 3339 UTC, to the second (`2026-11-22T10:00:00Z`).
    pub ts: String,
    /// Bytes the command wrote to stdout.
    pub printed: u64,
    /// Raw-equivalent bytes: what the agent would otherwise have read.
    pub raw: u64,
}

impl Record {
    /// A record stamped with the current time.
    pub fn now(command: &str, printed: u64, raw: u64) -> Self {
        Record {
            v: FORMAT,
            command: command.to_string(),
            ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            printed,
            raw,
        }
    }
}

/// `<root>/.specjutsu/ledger/gain.jsonl`.
pub fn path(root: &Path) -> PathBuf {
    root.join(".specjutsu").join("ledger").join("gain.jsonl")
}

/// Whether a run should be recorded: recording is on (no `--no-ledger`) and `root` is in a git
/// repository, the same switches as the evidence ledger.
pub fn enabled(root: &Path, no_ledger: bool) -> bool {
    !no_ledger && Git::new(root).is_repo()
}

/// Append one record with a single write. Callers check [`enabled`] first, or reuse what the
/// evidence ledger already found (see `sj test`).
pub fn append(root: &Path, record: &Record) -> Result<()> {
    let mut line = serde_json::to_string(record)?;
    line.push('\n');
    crate::ledger::append_lines(&path(root), &line)
}

/// Total size in bytes of every file under `dir`, subfolders included: the raw side of a
/// command that stands in for reading a change folder. Unreadable entries count as 0.
pub fn dir_bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_bytes(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

/// Records in file order, plus the number of lines that could not be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loaded {
    pub records: Vec<Record>,
    pub skipped: usize,
}

/// Every readable record. A missing file is empty; a line that isn't JSON, or has another
/// `v` than [`FORMAT`], is counted in `skipped`.
pub fn load(root: &Path) -> Result<Loaded> {
    let (records, skipped) = crate::ledger::load_lines(&path(root), |r: &Record| r.v == FORMAT)?;
    Ok(Loaded { records, skipped })
}

/// Runs and bytes for one command, or for all of them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Totals {
    pub runs: u64,
    pub printed: u64,
    pub raw: u64,
}

impl Totals {
    fn add(&mut self, r: &Record) {
        self.runs += 1;
        self.printed += r.printed;
        self.raw += r.raw;
    }

    /// Share of `raw` not printed, as a whole percent rounded down; 0 when `raw` is 0.
    pub fn saved_percent(&self) -> u64 {
        if self.raw == 0 {
            return 0;
        }
        let saved = u128::from(self.raw.saturating_sub(self.printed));
        (saved * 100 / u128::from(self.raw)) as u64
    }

    /// Estimated tokens saved: (raw − printed) / 4, rounded down.
    pub fn tokens_saved(&self) -> u64 {
        self.raw.saturating_sub(self.printed) / 4
    }
}

/// The report: one entry per recorded command, then the overall total.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    /// `test`, `verify`, `next`, `status` in that order, then any other command
    /// alphabetically; only commands with records.
    pub commands: Vec<(String, Totals)>,
    pub total: Totals,
}

const ORDER: [&str; 4] = ["test", "verify", "next", "status"];

pub fn summarize(records: &[Record]) -> Summary {
    let mut by: std::collections::BTreeMap<String, Totals> = Default::default();
    let mut total = Totals::default();
    for r in records {
        by.entry(r.command.clone()).or_default().add(r);
        total.add(r);
    }
    let mut commands: Vec<(String, Totals)> = by.into_iter().collect();
    // Stable sort keeps the alphabetical order of the commands not in ORDER.
    commands.sort_by_key(|(c, _)| ORDER.iter().position(|o| o == c).unwrap_or(ORDER.len()));
    Summary { commands, total }
}
