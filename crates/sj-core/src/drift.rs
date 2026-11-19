//! Spec drift (DRIFT-001..012): a map of the source lines each scenario's tests run, built from
//! LCOV, and the checks that compare it with a change.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{Config, Runner};
use crate::git::Git;
use crate::trace::TestRef;

/// Where the map lives, relative to the repository root.
pub const MAP_PATH: &str = ".specjutsu/coverage-map.json";

/// Per scenario, the source files and line ranges (inclusive) its tests run.
pub type Owned = BTreeMap<String, Vec<[usize; 2]>>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageMap {
    /// The commit at HEAD when the map was built.
    pub commit: String,
    /// The working tree's hash when the map was built, to tell whether it is up to date.
    pub tree: String,
    /// Lines dropped because most scenarios run them (DRIFT-003).
    pub shared_lines: usize,
    pub scenarios: BTreeMap<String, Owned>,
}

impl CoverageMap {
    pub fn load(root: &Path) -> Result<Option<CoverageMap>> {
        let path = root.join(MAP_PATH);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                Ok(Some(serde_json::from_str(&text).with_context(|| {
                    format!("{MAP_PATH}: not a coverage map")
                })?))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("cannot read {MAP_PATH}")),
        }
    }

    /// Scenarios that own `line` of `file`.
    pub fn owners(&self, file: &str, line: usize) -> Vec<&str> {
        self.scenarios
            .iter()
            .filter(|(_, files)| {
                files
                    .get(file)
                    .is_some_and(|rs| rs.iter().any(|[a, b]| (*a..=*b).contains(&line)))
            })
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

/// Lines with a hit count above zero, per `SF:` path as written in the LCOV text.
pub fn parse_lcov(text: &str) -> BTreeMap<String, BTreeSet<usize>> {
    let mut out: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    let mut file: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(sf) = line.strip_prefix("SF:") {
            file = Some(sf.to_string());
        } else if line == "end_of_record" {
            file = None;
        } else if let (Some(f), Some(da)) = (&file, line.strip_prefix("DA:")) {
            let mut parts = da.split(',');
            let (Some(l), Some(hits)) = (parts.next(), parts.next()) else {
                continue;
            };
            let hits: f64 = hits.trim().parse().unwrap_or(0.0);
            if let (Ok(l), true) = (l.trim().parse::<usize>(), hits > 0.0) {
                out.entry(f.clone()).or_default().insert(l);
            }
        }
    }
    out
}

/// Sorted lines as inclusive `[from, to]` runs.
pub fn ranges(lines: &BTreeSet<usize>) -> Vec<[usize; 2]> {
    let mut out: Vec<[usize; 2]> = Vec::new();
    for &l in lines {
        match out.last_mut() {
            Some(r) if r[1] + 1 == l => r[1] = l,
            _ => out.push([l, l]),
        }
    }
    out
}

/// `{tests}` for a runner: how it selects these tests on its command line.
fn tests_arg(runner: Runner, pkg_dir: &Path, root: &Path, tests: &[&TestRef]) -> String {
    let rel = |t: &TestRef| -> String {
        let abs = root.join(&t.file);
        abs.strip_prefix(pkg_dir)
            .unwrap_or(&t.file)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let mut parts: Vec<String> = tests
        .iter()
        .map(|t| match runner {
            Runner::Cargo | Runner::Junit => t.name.clone(),
            Runner::Vitest | Runner::Jest => rel(t),
            Runner::Pytest => format!("{}::{}", rel(t), t.name),
            Runner::Playwright => format!("{}:{}", rel(t), t.line),
        })
        .collect();
    parts.dedup();
    parts.join(" ")
}

/// Repository-relative path of an LCOV `SF:` entry written by a run in `pkg_dir`. Relative
/// entries are resolved without touching the filesystem, so a file that no longer exists keeps
/// its path; absolute ones are matched against the root as given and canonicalized.
fn source_rel(root: &Path, pkg_dir: &Path, sf: &str) -> String {
    let slash = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let p = PathBuf::from(sf);
    if p.is_relative() {
        let pkg = pkg_dir.strip_prefix(root).unwrap_or(Path::new(""));
        return normalize(&slash(&pkg.join(p)));
    }
    if let Ok(r) = p.strip_prefix(root) {
        return normalize(&slash(r));
    }
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    match canon(&p).strip_prefix(canon(root)) {
        Ok(r) => normalize(&slash(r)),
        Err(_) => slash(&p),
    }
}

/// Drop `.` segments and fold `..` in a forward-slash path.
fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// What `sj coverage map` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapOutcome {
    UpToDate,
    Built {
        scenarios: usize,
        files: usize,
        shared_lines: usize,
    },
}

/// Build `.specjutsu/coverage-map.json` by running each mapped scenario's tests under its
/// suite's `coverage` command (DRIFT-002..004).
pub fn build_map(root: &Path, spec_root: &Path) -> Result<MapOutcome> {
    let git = Git::new(root);
    let tree = git.worktree_tree_hash()?;
    if let Some(old) = CoverageMap::load(root)? {
        if old.tree == tree {
            return Ok(MapOutcome::UpToDate);
        }
    }
    let config = crate::config::load(root)?;
    let scenarios = crate::spec::load_scenarios(&root.join(spec_root))?;
    let scan = crate::testscan::scan_project(root, &config)?;
    let report = crate::trace::build_report(&scenarios, &scan.tests);
    let spec_rel = spec_root.to_string_lossy().replace('\\', "/");
    let classifier = crate::gate::Classifier::new(&config, &spec_rel)?;
    let test_files: BTreeSet<String> = scan
        .tests
        .iter()
        .map(|t| t.file.to_string_lossy().replace('\\', "/"))
        .collect();

    let mut raw: BTreeMap<String, BTreeMap<String, BTreeSet<usize>>> = BTreeMap::new();
    for m in &report.mapped {
        // Group the scenario's tests by the suite that runs them.
        let mut by_suite: BTreeMap<(usize, usize), Vec<&TestRef>> = BTreeMap::new();
        for t in &m.tests {
            if let Some(at) = suite_index(&config, t) {
                by_suite.entry(at).or_default().push(t);
            }
        }
        for ((pi, si), tests) in by_suite {
            let pkg = &config.packages[pi];
            let suite = &pkg.suites[si];
            let Some(cmd) = &suite.coverage else {
                continue;
            };
            let pkg_dir = crate::run::suite::package_dir(root, pkg);
            let tmp = tempfile::tempdir()?;
            let out_path = tmp.path().join("lcov.info");
            let command = cmd
                .replace("{lcov}", &out_path.to_string_lossy())
                .replace("{tests}", &tests_arg(suite.runner, &pkg_dir, root, &tests));
            let status = crate::run::suite::shell(&command)
                .current_dir(&pkg_dir)
                .envs(&suite.env)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .with_context(|| format!("cannot run the coverage command for {}", m.id))?;
            let Ok(text) = std::fs::read_to_string(&out_path) else {
                anyhow::bail!(
                    "coverage for {} wrote no LCOV file ({}): {command}",
                    m.id,
                    status
                );
            };
            for (sf, lines) in parse_lcov(&text) {
                let rel = source_rel(root, &pkg_dir, &sf);
                if test_files.contains(&rel) || !classifier.is_impl(&rel) {
                    continue;
                }
                raw.entry(m.id.clone())
                    .or_default()
                    .entry(rel)
                    .or_default()
                    .extend(lines);
            }
        }
    }

    // Shared lines: run by more than the threshold share of mapped scenarios.
    let mapped = raw.len();
    let mut count: BTreeMap<(String, usize), usize> = BTreeMap::new();
    for files in raw.values() {
        for (f, lines) in files {
            for &l in lines {
                *count.entry((f.clone(), l)).or_default() += 1;
            }
        }
    }
    let shared: BTreeSet<(String, usize)> = count
        .into_iter()
        .filter(|(_, n)| (*n as u64) * 1000 > config.drift_shared_permille as u64 * mapped as u64)
        .map(|(k, _)| k)
        .collect();

    let mut map = CoverageMap {
        commit: git.resolve("HEAD")?,
        tree,
        shared_lines: shared.len(),
        scenarios: BTreeMap::new(),
    };
    let mut files_seen: BTreeSet<String> = BTreeSet::new();
    for (id, files) in raw {
        let mut owned = Owned::new();
        for (f, lines) in files {
            let kept: BTreeSet<usize> = lines
                .into_iter()
                .filter(|l| !shared.contains(&(f.clone(), *l)))
                .collect();
            if !kept.is_empty() {
                files_seen.insert(f.clone());
                owned.insert(f, ranges(&kept));
            }
        }
        map.scenarios.insert(id, owned);
    }
    let path = root.join(MAP_PATH);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&map)? + "\n")?;
    Ok(MapOutcome::Built {
        scenarios: map.scenarios.len(),
        files: files_seen.len(),
        shared_lines: map.shared_lines,
    })
}

/// The (package, suite) indexes of a traced test's suite.
fn suite_index(config: &Config, t: &TestRef) -> Option<(usize, usize)> {
    let pkg = t.package.as_deref()?;
    let pi = config.packages.iter().position(|p| p.path == pkg)?;
    let si = config.packages[pi]
        .suites
        .iter()
        .position(|s| Some(s.runner) == t.suite)?;
    Some((pi, si))
}
