//! `sj status`: where each open change stands.

use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::{targets, Evidence, Ledger};
use crate::authoring::{self, Change};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Scenarios {
    pub proven: Vec<String>,
    pub green_unproven: Vec<String>,
    pub red: Vec<String>,
    pub untested: Vec<String>,
}

impl Scenarios {
    pub fn total(&self) -> usize {
        self.proven.len() + self.green_unproven.len() + self.red.len() + self.untested.len()
    }

    /// (state, IDs) in display order.
    pub fn by_state(&self) -> [(Evidence, &Vec<String>); 4] {
        [
            (Evidence::Proven, &self.proven),
            (Evidence::GreenUnproven, &self.green_unproven),
            (Evidence::Red, &self.red),
            (Evidence::Untested, &self.untested),
        ]
    }
}

/// One change's status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    pub name: String,
    pub tasks_done: usize,
    pub tasks_total: usize,
    pub scenarios: Scenarios,
    /// The first unchecked task's text.
    pub next: Option<String>,
}

pub fn status_of(change: &Change, ledger: &Ledger) -> Result<Status> {
    let tasks = authoring::tasks(&change.dir.join("tasks.md"))?;
    let mut scenarios = Scenarios::default();
    for id in targets(change)? {
        let bucket = match ledger.evidence(&id) {
            Evidence::Proven => &mut scenarios.proven,
            Evidence::GreenUnproven => &mut scenarios.green_unproven,
            Evidence::Red => &mut scenarios.red,
            Evidence::Untested => &mut scenarios.untested,
        };
        bucket.push(id);
    }
    Ok(Status {
        name: change.name.clone(),
        tasks_done: tasks.iter().filter(|t| t.done).count(),
        tasks_total: tasks.len(),
        scenarios,
        next: tasks.iter().find(|t| !t.done).map(|t| t.text.clone()),
    })
}

/// Status of the named change, or of every open change in name order.
pub fn status(root: &Path, spec_root: &Path, change: Option<&str>) -> Result<Vec<Status>> {
    status_of_changes(root, &changes(spec_root, change)?)
}

/// The named change, or every open change in name order: what `status` reports on.
pub fn changes(spec_root: &Path, change: Option<&str>) -> Result<Vec<Change>> {
    Ok(match change {
        Some(name) => vec![authoring::find_change(spec_root, name)?],
        None => authoring::open_changes(spec_root)?,
    })
}

/// Status of each of `changes`, in order.
pub fn status_of_changes(root: &Path, changes: &[Change]) -> Result<Vec<Status>> {
    let ledger = Ledger::load(root)?;
    changes.iter().map(|c| status_of(c, &ledger)).collect()
}

/// Most IDs listed on one line before `… +<n>`.
pub const MAX_IDS: usize = 8;

/// The compact text for one change: at most five lines.
pub fn render(s: &Status) -> Vec<String> {
    let counts: Vec<String> = s
        .scenarios
        .by_state()
        .iter()
        .filter(|(_, ids)| !ids.is_empty())
        .map(|(e, ids)| format!("{} {}", ids.len(), e.as_str()))
        .collect();
    let total = s.scenarios.total();
    let noun = if total == 1 { "scenario" } else { "scenarios" };
    let mut head = format!(
        "{}  {}/{} tasks  {total} {noun}",
        s.name, s.tasks_done, s.tasks_total
    );
    if !counts.is_empty() {
        head.push_str(&format!(": {}", counts.join(", ")));
    }
    let mut out = vec![head];
    for (e, ids) in s.scenarios.by_state() {
        if e == Evidence::Proven || ids.is_empty() {
            continue;
        }
        let mut line = format!(
            "  {} {}",
            e.as_str(),
            ids.iter()
                .take(MAX_IDS)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
        if ids.len() > MAX_IDS {
            line.push_str(&format!(" … +{}", ids.len() - MAX_IDS));
        }
        out.push(line);
    }
    out.push(match &s.next {
        Some(t) => format!("  next {t}"),
        None => "  all tasks done".to_string(),
    });
    out
}
