//! Gain records: per run of an agent-loop command, the bytes it printed against the raw bytes it
//! replaces. JSON Lines in `.specjutsu/ledger/gain.jsonl`, next to the evidence ledger.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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

/// Append one record with a single write.
pub fn append(root: &Path, record: &Record) -> Result<()> {
    let p = path(root);
    std::fs::create_dir_all(p.parent().unwrap())
        .with_context(|| format!("creating {}", p.parent().unwrap().display()))?;
    let mut line = serde_json::to_string(record)?;
    line.push('\n');
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .with_context(|| format!("opening {}", p.display()))?;
    f.write_all(line.as_bytes())
        .with_context(|| format!("writing {}", p.display()))
}

/// Record one run when [`enabled`]; returns whether a record was written.
pub fn record(root: &Path, no_ledger: bool, command: &str, printed: u64, raw: u64) -> Result<bool> {
    if !enabled(root, no_ledger) {
        return Ok(false);
    }
    append(root, &Record::now(command, printed, raw))?;
    Ok(true)
}

/// Records in file order, plus the number of lines that could not be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loaded {
    pub records: Vec<Record>,
    pub skipped: usize,
}

pub fn load(root: &Path) -> Result<Loaded> {
    let p = path(root);
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::default()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", p.display())),
    };
    let mut out = Loaded::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match serde_json::from_str::<Record>(line) {
            Ok(r) if r.v == FORMAT => out.records.push(r),
            _ => out.skipped += 1,
        }
    }
    Ok(out)
}
