//! Join scenarios and tests into a trace report.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Serialize;

use crate::config::Runner;
use crate::spec::Scenario;
use crate::testscan::TestFn;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestRef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub package: Option<String>,
    pub suite: Option<Runner>,
    pub skipped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mapped {
    pub id: String,
    pub name: String,
    pub tests: Vec<TestRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Orphan {
    pub id: String,
    pub test: TestRef,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub mapped: Vec<Mapped>,
    pub gaps: Vec<Scenario>,
    pub orphans: Vec<Orphan>,
    pub unidentified: Vec<Scenario>,
    /// Setup problems worth a look, e.g. `no test suite: packages/ui`.
    pub warnings: Vec<String>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.gaps.is_empty() && self.orphans.is_empty()
    }
}

/// File name of the accepted-debt list, kept in the spec root.
pub const BASELINE_FILE: &str = "trace-baseline.txt";

/// One accepted gap or orphan in the baseline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaselineEntry {
    Gap(String),
    /// ID, test file relative to the repo root with forward slashes, test name.
    Orphan(String, String, String),
}

/// The gaps and orphans a repository has accepted, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub entries: BTreeSet<BaselineEntry>,
}

impl Baseline {
    pub fn from_report(report: &Report) -> Self {
        let gaps = report
            .gaps
            .iter()
            .filter_map(|s| s.id.clone())
            .map(BaselineEntry::Gap);
        let orphans = report.orphans.iter().map(|o| {
            let file = o
                .test
                .file
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            BaselineEntry::Orphan(o.id.clone(), file, o.test.name.clone())
        });
        Baseline {
            entries: gaps.chain(orphans).collect(),
        }
    }

    /// One entry per line, with a trailing newline.
    pub fn render(&self) -> String {
        self.entries
            .iter()
            .map(|e| match e {
                BaselineEntry::Gap(id) => format!("gap {id}\n"),
                BaselineEntry::Orphan(id, file, test) => format!("orphan {id} {file} {test}\n"),
            })
            .collect()
    }
}

/// Link tests to scenarios. A name-embedded ID counts as an orphan only when its prefix is one the
/// specs use (avoids noise like `v_2`); `// spec:` comment IDs are explicit and always count.
pub fn build_report(scenarios: &[Scenario], tests: &[TestFn]) -> Report {
    let mut report = Report::default();
    // First occurrence of each ID wins (a change's delta may repeat a current-spec ID).
    let mut order: Vec<&Scenario> = Vec::new();
    let mut known: BTreeSet<&str> = BTreeSet::new();
    for s in scenarios {
        match &s.id {
            None => report.unidentified.push(s.clone()),
            Some(id) => {
                if known.insert(id.as_str()) {
                    order.push(s);
                }
            }
        }
    }
    let prefixes: BTreeSet<&str> = known
        .iter()
        .filter_map(|id| id.rsplit_once('-').map(|(p, _)| p))
        .collect();

    let mut links: BTreeMap<&str, Vec<TestRef>> = BTreeMap::new();
    for t in tests {
        let tref = TestRef {
            name: t.name.clone(),
            file: t.file.clone(),
            line: t.line,
            package: t.package.clone(),
            suite: t.suite,
            skipped: t.skipped,
        };
        let mut ids: Vec<(String, bool)> =
            t.comment_ids.iter().map(|i| (i.clone(), true)).collect();
        ids.extend(
            t.name_ids
                .iter()
                .filter(|id| !t.comment_ids.contains(id))
                .map(|id| (id.clone(), false)),
        );
        for (id, explicit) in ids {
            if let Some(k) = known.get(id.as_str()) {
                links.entry(k).or_default().push(tref.clone());
            } else {
                let prefix = id.rsplit_once('-').map_or("", |(p, _)| p);
                if explicit || prefixes.contains(prefix) {
                    report.orphans.push(Orphan {
                        id,
                        test: tref.clone(),
                    });
                }
            }
        }
    }

    for s in order {
        let id = s.id.as_deref().unwrap_or_default();
        match links.remove(id) {
            Some(tests) => report.mapped.push(Mapped {
                id: id.to_string(),
                name: s.name.clone(),
                tests,
            }),
            None => report.gaps.push(s.clone()),
        }
    }
    report
}
