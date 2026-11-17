//! `sj spec`: create, list, validate, check and archive spec changes.

pub mod archive;
pub mod blocks;
pub mod check;
pub mod validate;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;

/// An open change folder: `<spec_root>/changes/<name>/`, not under `archive/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub name: String,
    pub dir: PathBuf,
}

/// Open changes in name order.
pub fn open_changes(spec_root: &Path) -> Result<Vec<Change>> {
    let dir = spec_root.join("changes");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<Change> = std::fs::read_dir(&dir)
        .with_context(|| format!("cannot read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name() != "archive")
        .map(|e| Change {
            name: e.file_name().to_string_lossy().into_owned(),
            dir: e.path(),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// The open change named `name`, or an error naming it.
pub fn find_change(spec_root: &Path, name: &str) -> Result<Change> {
    let dir = spec_root.join("changes").join(name);
    if name == "archive" || !dir.is_dir() {
        bail!(
            "no open change `{name}` in {}",
            spec_root.join("changes").display()
        );
    }
    Ok(Change {
        name: name.to_string(),
        dir,
    })
}

/// Every `spec.md` under `dir`, sorted, recursively.
pub fn spec_files(dir: &Path) -> Result<Vec<PathBuf>> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .with_context(|| format!("cannot read {}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out)?;
            } else if p.file_name().is_some_and(|n| n == "spec.md") {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, &mut out)?;
    Ok(out)
}

pub(crate) fn read(file: &Path) -> Result<String> {
    std::fs::read_to_string(file)
        .map(|t| blocks::normalize(&t))
        .with_context(|| format!("cannot read {}", file.display()))
}

/// One `- [ ]` / `- [x]` line of `tasks.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub done: bool,
    pub text: String,
    pub line: usize,
}

/// Task lines outside fenced code. A missing file has no tasks.
pub fn tasks(file: &Path) -> Result<Vec<Task>> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"^\s*[-*] \[( |x|X)\]\s+(.*?)\s*$").unwrap());
    if !file.is_file() {
        return Ok(Vec::new());
    }
    let text = read(file)?;
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    Ok(lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !mask[*i])
        .filter_map(|(i, l)| {
            re.captures(l).map(|c| Task {
                done: &c[1] != " ",
                text: c[2].to_string(),
                line: i + 1,
            })
        })
        .collect())
}

/// Whether a change name is kebab-case: lowercase letters and digits joined by single hyphens.
pub fn is_kebab(name: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap())
        .is_match(name)
}

const PROPOSAL: &str = "## Why

<!-- The problem or opportunity, and why now. One or two paragraphs. -->

## What Changes

<!-- One bullet per change. Mark breaking changes with **BREAKING**. -->

## Capabilities

### New Capabilities

<!-- Each new capability gets specs/<capability>/spec.md in this change. -->

### Modified Capabilities

<!-- Existing capabilities whose requirements change. -->

## Impact

<!-- Code, APIs, dependencies and systems affected. -->
";

const DESIGN: &str = "## Context

<!-- Background and the current state. -->

## Goals / Non-Goals

**Goals:**

**Non-Goals:**

## Decisions

<!-- Each decision with the alternatives considered. -->

## Risks / Trade-offs

<!-- [Risk] -> Mitigation -->
";

const TASKS: &str = "# Tasks

<!--
One checkbox line per task, grouped under numbered headings. Each task names the scenario
IDs it covers in parentheses, for example (AUTH-005) or (AUTH-005..007). A task that covers
no scenario says why: (no-scenario: docs).
-->
";

/// Create `changes/<name>/` with the templates and an empty `specs/` folder.
pub fn new_change(spec_root: &Path, name: &str) -> Result<PathBuf> {
    if !is_kebab(name) {
        bail!("change name `{name}` must be kebab-case (lowercase letters, digits and hyphens)");
    }
    let dir = spec_root.join("changes").join(name);
    if name == "archive" || dir.exists() {
        bail!("change `{name}` already exists: {}", dir.display());
    }
    std::fs::create_dir_all(dir.join("specs"))
        .with_context(|| format!("cannot create {}", dir.display()))?;
    for (file, text) in [
        ("proposal.md", PROPOSAL),
        ("design.md", DESIGN),
        ("tasks.md", TASKS),
    ] {
        std::fs::write(dir.join(file), text)
            .with_context(|| format!("cannot write {}", dir.join(file).display()))?;
    }
    Ok(dir)
}

/// One `sj spec list` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Listed {
    pub name: String,
    pub tasks_done: usize,
    pub tasks_total: usize,
    /// Scenarios in the change's ADDED and MODIFIED sections.
    pub scenarios: usize,
}

pub fn list(spec_root: &Path) -> Result<Vec<Listed>> {
    let mut out = Vec::new();
    for c in open_changes(spec_root)? {
        let t = tasks(&c.dir.join("tasks.md"))?;
        let mut scenarios = 0;
        for file in spec_files(&c.dir.join("specs"))? {
            let text = read(&file)?;
            let lines: Vec<&str> = text.split('\n').collect();
            let mask = blocks::fence_mask(&lines);
            let d = blocks::parse_delta(&text);
            for b in d.added.iter().chain(&d.modified) {
                scenarios += blocks::scenarios_in(&lines, &mask, b.lines.clone()).len();
            }
        }
        out.push(Listed {
            name: c.name,
            tasks_done: t.iter().filter(|t| t.done).count(),
            tasks_total: t.len(),
            scenarios,
        });
    }
    Ok(out)
}
