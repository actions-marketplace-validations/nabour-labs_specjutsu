//! `sj spec check`: gaps between a change's scenarios and its tasks, and open questions.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;

use super::{blocks, find_change, read, spec_files, tasks};

/// A task naming an ID that neither the change nor `specs/` defines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unknown {
    pub id: String,
    /// Relative to the change folder.
    pub file: PathBuf,
    pub line: usize,
}

/// A `[NEEDS CLARIFICATION: <question>]` marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Clarify {
    pub question: String,
    /// Relative to the change folder.
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CheckReport {
    /// Scenario IDs the change adds or modifies that no task names.
    pub uncovered: Vec<String>,
    pub unknown: Vec<Unknown>,
    pub clarify: Vec<Clarify>,
    /// Problems with the change as a whole, e.g. a bug change with no reproduction (SPEC-042).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    /// Advice that never fails the check (SPEC-041).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl CheckReport {
    pub fn is_clean(&self) -> bool {
        self.uncovered.is_empty()
            && self.unknown.is_empty()
            && self.clarify.is_empty()
            && self.errors.is_empty()
    }

    pub fn problems(&self) -> usize {
        self.uncovered.len() + self.unknown.len() + self.clarify.len() + self.errors.len()
    }
}

/// Scenarios in a file's requirement blocks (`delta_only`: ADDED and MODIFIED blocks only).
fn scenarios_of(text: &str, delta_only: bool) -> Vec<blocks::Scenario> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    let ranges: Vec<std::ops::Range<usize>> = if delta_only {
        let d = blocks::parse_delta(text);
        d.added
            .iter()
            .chain(&d.modified)
            .map(|b| b.lines.clone())
            .collect()
    } else {
        std::iter::once(0..lines.len()).collect()
    };
    ranges
        .into_iter()
        .flat_map(|r| blocks::scenarios_in(&lines, &mask, r))
        .collect()
}

/// Scenario IDs in a file's requirement blocks (`delta_only`: ADDED and MODIFIED blocks only).
fn ids_in(text: &str, delta_only: bool) -> Vec<String> {
    scenarios_of(text, delta_only)
        .into_iter()
        .filter_map(|s| s.id)
        .collect()
}

fn md_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("cannot read {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            md_files(&p, out)?;
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
    Ok(())
}

pub fn check(spec_root: &Path, change: &str) -> Result<CheckReport> {
    static CLARIFY: OnceLock<Regex> = OnceLock::new();
    static CODE: OnceLock<Regex> = OnceLock::new();
    let clarify_re =
        CLARIFY.get_or_init(|| Regex::new(r"\[NEEDS CLARIFICATION:\s*(.*?)\s*\]").unwrap());
    // A marker that starts inside an inline code span is quoted, not asked.
    let code_re = CODE.get_or_init(|| Regex::new(r"`[^`]*`").unwrap());
    let c = find_change(spec_root, change)?;
    let mut r = CheckReport::default();

    let mut change_ids: Vec<String> = Vec::new();
    let (mut keeps, mut normals) = (0usize, 0usize);
    for file in spec_files(&c.dir.join("specs"))? {
        for s in scenarios_of(&read(&file)?, true) {
            if s.marker.as_deref() == Some(crate::spec::KEEP) {
                keeps += 1;
            } else {
                normals += 1;
            }
            if let Some(id) = s.id {
                if !change_ids.contains(&id) {
                    change_ids.push(id);
                }
            }
        }
    }
    // SPEC-041/042: a bug change needs a reproduction, and should pin what must not change.
    let proposal = c.dir.join("proposal.md");
    if proposal.is_file() && super::is_bug_proposal(&read(&proposal)?) {
        if normals == 0 {
            r.errors
                .push("bug change has no reproduction scenario".to_string());
        }
        if keeps == 0 {
            r.warnings.push(
                "bug change has no [keep] scenario; add one for behaviour that must not change"
                    .to_string(),
            );
        }
    }
    let mut known: BTreeSet<String> = change_ids.iter().cloned().collect();
    for file in spec_files(&spec_root.join("specs"))? {
        known.extend(ids_in(&read(&file)?, false));
    }

    let tasks_file = c.dir.join("tasks.md");
    let mut named = BTreeSet::new();
    for t in tasks(&tasks_file)? {
        for id in blocks::task_ids(&t.text) {
            if !known.contains(&id) && !r.unknown.iter().any(|u| u.id == id) {
                r.unknown.push(Unknown {
                    id: id.clone(),
                    file: PathBuf::from("tasks.md"),
                    line: t.line,
                });
            }
            named.insert(id);
        }
    }
    r.uncovered = change_ids
        .into_iter()
        .filter(|id| !named.contains(id))
        .collect();

    let mut files = Vec::new();
    md_files(&c.dir, &mut files)?;
    for file in files {
        let text = read(&file)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let mask = blocks::fence_mask(&lines);
        let rel = file.strip_prefix(&c.dir).unwrap_or(&file).to_path_buf();
        for (i, line) in lines.iter().enumerate() {
            if mask[i] {
                continue;
            }
            let spans: Vec<_> = code_re.find_iter(line).map(|m| m.range()).collect();
            for m in clarify_re.captures_iter(line) {
                let at = m.get(0).unwrap().start();
                if spans.iter().any(|s| s.contains(&at)) {
                    continue;
                }
                r.clarify.push(Clarify {
                    question: m[1].to_string(),
                    file: rel.clone(),
                    line: i + 1,
                });
            }
        }
    }
    Ok(r)
}
