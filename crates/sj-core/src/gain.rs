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
