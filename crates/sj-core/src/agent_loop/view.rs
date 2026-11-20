//! `sj view`: a one-screen project dashboard. Reads files and the ledger only.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::status::{status_of, Status};
use super::{Evidence, Ledger};
use crate::authoring::{self, show};
use crate::config::Config;
use crate::spec;

/// Most capability lines before `… +N more`.
pub const MAX_CAPABILITIES: usize = 20;
/// Most change lines before `… +N more`.
pub const MAX_CHANGES: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Capability {
    pub name: String,
    pub scenarios: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub proven: usize,
    pub green_unproven: usize,
    pub red: usize,
    pub untested: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub capabilities: Vec<Capability>,
    pub changes: Vec<Status>,
    /// Evidence over every scenario in `specs/` and the open changes, each ID once.
    pub totals: Totals,
    /// Tests naming an ID that no scenario has.
    pub orphans: usize,
    /// Folders in `changes/archive/`.
    pub archived: usize,
}

pub fn view(root: &Path, spec_root: &Path, config: &Config) -> Result<View> {
    let mut capabilities = Vec::new();
    for (name, file) in show::capabilities(spec_root)? {
        let text = std::fs::read_to_string(&file)?;
        capabilities.push(Capability {
            name,
            scenarios: spec::parse_scenarios(&file, &text).len(),
        });
    }

    let ledger = Ledger::load(root)?;
    let changes = authoring::open_changes(spec_root)?
        .iter()
        .map(|c| status_of(c, &ledger))
        .collect::<Result<Vec<_>>>()?;

    let scenarios = spec::load_scenarios(spec_root)?;
    let mut totals = Totals::default();
    let ids: BTreeSet<&str> = scenarios.iter().filter_map(|s| s.id.as_deref()).collect();
    for id in ids {
        match ledger.evidence(id) {
            Evidence::Proven => totals.proven += 1,
            Evidence::GreenUnproven => totals.green_unproven += 1,
            Evidence::Red => totals.red += 1,
            Evidence::Untested => totals.untested += 1,
        }
    }

    let scan = crate::testscan::scan_project(root, config)?;
    let orphans = crate::trace::build_report(&scenarios, &scan.tests)
        .orphans
        .len();

    Ok(View {
        capabilities,
        changes,
        totals,
        orphans,
        archived: show::archived(spec_root)?.len(),
    })
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn counts(proven: usize, green: usize, red: usize, untested: usize) -> String {
    format!("{proven} proven, {green} green-unproven, {red} red, {untested} untested")
}

/// The dashboard text: at most 40 lines.
pub fn render(v: &View) -> Vec<String> {
    let mut out = vec![format!("capabilities ({})", v.capabilities.len())];
    for c in v.capabilities.iter().take(MAX_CAPABILITIES) {
        out.push(format!("  {}  {}", c.name, plural(c.scenarios, "scenario")));
    }
    if v.capabilities.len() > MAX_CAPABILITIES {
        out.push(format!(
            "  … +{} more",
            v.capabilities.len() - MAX_CAPABILITIES
        ));
    }
    if v.changes.is_empty() {
        out.push("open changes: none".to_string());
    } else {
        out.push(format!("open changes ({})", v.changes.len()));
        for c in v.changes.iter().take(MAX_CHANGES) {
            let s = &c.scenarios;
            out.push(format!(
                "  {}  {}/{} tasks  {}",
                c.name,
                c.tasks_done,
                c.tasks_total,
                counts(
                    s.proven.len(),
                    s.green_unproven.len(),
                    s.red.len(),
                    s.untested.len()
                )
            ));
        }
        if v.changes.len() > MAX_CHANGES {
            out.push(format!("  … +{} more", v.changes.len() - MAX_CHANGES));
        }
    }
    let t = &v.totals;
    out.push(format!(
        "scenarios  {}",
        counts(t.proven, t.green_unproven, t.red, t.untested)
    ));
    out.push(plural(v.orphans, "orphan test"));
    out.push(format!("{} archived", v.archived));
    out
}
