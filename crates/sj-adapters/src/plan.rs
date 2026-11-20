//! An adapter install or update as data: every file it would write, keep or remove, computed
//! before anything is touched. `sj update --dry-run` prints the plan, and `apply` carries it
//! out, so the preview can't drift from what the update does.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::claude::{rel, Change};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op {
    Write(String),
    Keep,
    Delete,
}

/// One file in a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    path: PathBuf,
    op: Op,
    /// What applying this step reports.
    pub change: Change,
}

impl Step {
    /// Write `text` to `path`: created when `existed` is false, updated otherwise.
    pub(crate) fn write(path: PathBuf, root: &Path, existed: bool, text: String) -> Step {
        let r = rel(&path, root);
        Step {
            path,
            op: Op::Write(text),
            change: if existed {
                Change::Updated(r)
            } else {
                Change::Created(r)
            },
        }
    }

    /// Leave a hand-edited generated file as it is.
    pub(crate) fn keep(path: PathBuf, root: &Path) -> Step {
        let change = Change::Kept(rel(&path, root));
        Step {
            path,
            op: Op::Keep,
            change,
        }
    }

    /// Delete a file an older version generated and this one no longer does.
    pub(crate) fn delete(path: PathBuf, root: &Path) -> Step {
        let change = Change::Removed(rel(&path, root));
        Step {
            path,
            op: Op::Delete,
            change,
        }
    }

    /// The step's action as `sj update --dry-run` prints it: `write`, `keep` or `remove`.
    pub fn action(&self) -> &'static str {
        match self.op {
            Op::Write(_) => "write",
            Op::Keep => "keep",
            Op::Delete => "remove",
        }
    }

    /// The file, forward-slash and relative to the repository root.
    pub fn path(&self) -> &str {
        match &self.change {
            Change::Created(p) | Change::Updated(p) | Change::Removed(p) | Change::Kept(p) => p,
        }
    }
}

/// The step that brings `path` from `old` to `new`, or `None` when they are equal.
pub(crate) fn put(path: PathBuf, root: &Path, old: Option<&str>, new: String) -> Option<Step> {
    (old != Some(new.as_str())).then(|| Step::write(path, root, old.is_some(), new))
}

/// Carry out every step, in order, and report what changed.
pub fn apply(steps: Vec<Step>) -> Result<Vec<Change>> {
    let mut out = Vec::with_capacity(steps.len());
    for s in steps {
        match &s.op {
            Op::Write(text) => {
                if let Some(dir) = s.path.parent() {
                    std::fs::create_dir_all(dir)
                        .with_context(|| format!("cannot create {}", dir.display()))?;
                }
                std::fs::write(&s.path, text)
                    .with_context(|| format!("cannot write {}", s.path.display()))?;
            }
            Op::Keep => {}
            Op::Delete => {
                std::fs::remove_file(&s.path)
                    .with_context(|| format!("cannot remove {}", s.path.display()))?;
                if let Some(dir) = s.path.parent() {
                    crate::claude::remove_if_empty(dir);
                }
            }
        }
        out.push(s.change);
    }
    Ok(out)
}
