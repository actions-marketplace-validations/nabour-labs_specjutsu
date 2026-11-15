//! `sj test`: choose the tests in scope, run them, and join results to scenario IDs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{bail, Result};
use regex::Regex;
use serde::Serialize;
use sj_runners::cargo::{cargo_test_detailed, Outcome};

use crate::{spec, testscan, trace};

/// Most failure-output lines shown per failing test.
pub const EXCERPT_LINES: usize = 20;

/// What `sj test [SCOPE]` runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    All,
    /// Only the tests mapped to this scenario ID.
    Scenario(String),
    /// One Cargo package.
    Package(String),
}

impl Scope {
    /// A scope shaped like a scenario ID (`PREFIX-NNN`) is a scenario; anything else is a package.
    pub fn parse(arg: Option<&str>) -> Scope {
        static ID: OnceLock<Regex> = OnceLock::new();
        let id = ID.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9]*-[0-9]+$").unwrap());
        match arg {
            None => Scope::All,
            Some(s) if id.is_match(s) => Scope::Scenario(s.to_string()),
            Some(s) => Scope::Package(s.to_string()),
        }
    }
}

/// How to invoke the runner for a scope.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    pub cargo_args: Vec<String>,
    pub filters: Vec<String>,
    /// For a scenario scope: the mapped test names and their IDs. Results for other names are dropped.
    pub only: Option<BTreeMap<String, Vec<String>>>,
    /// Test function name → mapped scenario IDs, for every test in source.
    pub ids: BTreeMap<String, Vec<String>>,
}

/// The first lines of a failure's output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Excerpt {
    pub lines: Vec<String>,
    /// Lines cut after the limit.
    pub omitted: usize,
}

pub fn excerpt(text: &str, limit: usize) -> Excerpt {
    let all: Vec<&str> = text.lines().collect();
    Excerpt {
        lines: all.iter().take(limit).map(|l| l.to_string()).collect(),
        omitted: all.len().saturating_sub(limit),
    }
}

/// One test that ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestCase {
    /// libtest path, e.g. `tests::foo` or `foo`.
    pub path: String,
    pub result: Outcome,
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<Excerpt>,
}

/// The outcome of `sj test`, independent of the runner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestRun {
    pub pass: bool,
    pub build_failed: bool,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_errors: Vec<String>,
    pub tests: Vec<TestCase>,
    /// For a scenario scope: the selected test names and IDs (used to record build failures).
    #[serde(skip)]
    pub selected: Vec<(String, Vec<String>)>,
}

/// Work out what to run for `scope`. A scenario with no mapped test is an error.
pub fn resolve(root: &Path, spec_root: &Path, scope: &Scope) -> Result<Selection> {
    let scenarios = spec::load_scenarios(&root.join(spec_root))?;
    let tests = testscan::scan_tree(root)?;
    let report = trace::build_report(&scenarios, &tests);
    let mut ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for m in &report.mapped {
        for t in &m.tests {
            let v = ids.entry(t.name.clone()).or_default();
            if !v.contains(&m.id) {
                v.push(m.id.clone());
            }
        }
    }
    let mut sel = Selection {
        ids,
        ..Selection::default()
    };
    match scope {
        Scope::All => sel.cargo_args.push("--workspace".into()),
        Scope::Package(p) => sel.cargo_args.extend(["-p".to_string(), p.clone()]),
        Scope::Scenario(id) => {
            let Some(m) = report.mapped.iter().find(|m| &m.id == id) else {
                bail!("scenario {id} has no mapped test");
            };
            let names: BTreeSet<&str> = m.tests.iter().map(|t| t.name.as_str()).collect();
            sel.cargo_args.push("--workspace".into());
            sel.filters = names.iter().map(|n| n.to_string()).collect();
            sel.only = Some(
                names
                    .iter()
                    .map(|n| (n.to_string(), sel.ids[*n].clone()))
                    .collect(),
            );
        }
    }
    Ok(sel)
}

/// Run the tests in `scope` under `root` (a Cargo project) and join results to scenario IDs.
pub fn run_tests(root: &Path, spec_root: &Path, scope: &Scope) -> Result<TestRun> {
    if !root.join("Cargo.toml").is_file() {
        bail!(
            "no Cargo.toml in {}; sj test supports Cargo projects only for now",
            root.display()
        );
    }
    let sel = resolve(root, spec_root, scope)?;
    let args: Vec<&str> = sel.cargo_args.iter().map(String::as_str).collect();
    let filters: Vec<&str> = sel.filters.iter().map(String::as_str).collect();
    let d = cargo_test_detailed(root, &args, &filters)?;
    let selected: Vec<(String, Vec<String>)> = sel
        .only
        .as_ref()
        .map(|o| o.iter().map(|(n, i)| (n.clone(), i.clone())).collect())
        .unwrap_or_default();
    if d.build_failed {
        return Ok(TestRun {
            build_failed: true,
            build_errors: d.build_errors,
            selected,
            ..TestRun::default()
        });
    }
    let mut run = TestRun {
        selected,
        ..TestRun::default()
    };
    for r in &d.run.results {
        let name = r.path.rsplit("::").next().unwrap_or(&r.path);
        if let Some(only) = &sel.only {
            if !only.contains_key(name) {
                continue;
            }
        }
        match r.outcome {
            Outcome::Passed => run.passed += 1,
            Outcome::Failed => run.failed += 1,
            Outcome::Ignored => run.ignored += 1,
        }
        run.tests.push(TestCase {
            path: r.path.clone(),
            result: r.outcome,
            ids: sel.ids.get(name).cloned().unwrap_or_default(),
            excerpt: (r.outcome == Outcome::Failed)
                .then(|| d.failures.get(&r.path))
                .flatten()
                .map(|t| excerpt(t, EXCERPT_LINES)),
        });
    }
    // With a post-filter, a dropped unrelated failure must not fail the run.
    run.pass = run.failed == 0 && (sel.only.is_some() || d.run.success);
    Ok(run)
}
