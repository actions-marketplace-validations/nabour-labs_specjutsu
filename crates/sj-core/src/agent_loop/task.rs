//! `sj task done`: check one task's box, with proof in strict mode.

use std::path::Path;

use anyhow::{bail, Context, Result};

use super::{task_number, Ledger};
use crate::authoring::{self, blocks};
use crate::config::Mode;

/// An ID the task names that the ledger does not prove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unproven {
    pub id: String,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    AlreadyDone {
        text: String,
    },
    /// The box was checked; `warnings` are the unproven IDs (never strict mode).
    Done {
        text: String,
        warnings: Vec<Unproven>,
    },
    /// Strict mode: nothing was written.
    Refused {
        unproven: Vec<Unproven>,
    },
}

/// Byte offset of each line start in `raw`, treating `\r\n`, `\n` and a lone `\r` as line ends
/// (the same lines `authoring::read` sees after normalising).
fn line_starts(raw: &[u8]) -> Vec<usize> {
    let mut out = vec![0];
    let mut i = 0;
    while i < raw.len() {
        match raw[i] {
            b'\r' if raw.get(i + 1) == Some(&b'\n') => {
                out.push(i + 2);
                i += 2;
                continue;
            }
            b'\r' | b'\n' => out.push(i + 1),
            _ => {}
        }
        i += 1;
    }
    out
}

pub fn done(
    spec_root: &Path,
    root: &Path,
    change: &str,
    number: &str,
    mode: Mode,
) -> Result<Outcome> {
    let c = authoring::find_change(spec_root, change)?;
    let file = c.dir.join("tasks.md");
    let tasks = authoring::tasks(&file)?;
    let Some(task) = tasks.iter().find(|t| task_number(&t.text) == number) else {
        bail!("no task {number} in {}", file.display());
    };
    if task.done {
        return Ok(Outcome::AlreadyDone {
            text: task.text.clone(),
        });
    }

    let ledger = Ledger::load(root)?;
    let mut unproven = Vec::new();
    for id in blocks::task_ids(&task.text) {
        if unproven.iter().any(|u: &Unproven| u.id == id) {
            continue;
        }
        if let Some(why) = ledger.why_unproven(&id) {
            unproven.push(Unproven { id, why });
        }
    }
    if mode == Mode::Strict && !unproven.is_empty() {
        return Ok(Outcome::Refused { unproven });
    }

    let mut raw =
        std::fs::read(&file).with_context(|| format!("cannot read {}", file.display()))?;
    let starts = line_starts(&raw);
    let start = starts[task.line - 1];
    let end = starts.get(task.line).copied().unwrap_or(raw.len());
    let Some(at) = raw[start..end].windows(3).position(|w| w == b"[ ]") else {
        bail!("cannot find the box of task {number} in {}", file.display());
    };
    raw[start + at + 1] = b'x';
    std::fs::write(&file, &raw).with_context(|| format!("cannot write {}", file.display()))?;
    Ok(Outcome::Done {
        text: task.text.clone(),
        warnings: unproven,
    })
}
