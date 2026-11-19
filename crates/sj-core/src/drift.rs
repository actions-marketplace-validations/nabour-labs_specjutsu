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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Fail,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DriftFinding {
    /// `code-drift` or `stale-test`.
    pub kind: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub severity: Severity,
    /// The commit of the coverage map a code-drift finding came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

impl DriftFinding {
    /// The line printed after the severity column.
    pub fn text(&self) -> String {
        match (self.kind.as_str(), &self.file, self.line) {
            ("code-drift", Some(f), Some(l)) => format!(
                "code-drift {} {f}:{l} — code its tests run changed; spec and tests did not",
                self.id
            ),
            _ => format!("stale-test {} — spec changed; its tests did not", self.id),
        }
    }

    /// The gate's failure reason: the finding without its explanation.
    pub fn reason(&self) -> String {
        match (&self.file, self.line) {
            (Some(f), Some(l)) => format!("{} {} {f}:{l}", self.kind, self.id),
            _ => format!("{} {}", self.kind, self.id),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DriftReport {
    pub pass: bool,
    pub summary: String,
    pub findings: Vec<DriftFinding>,
    /// What was not checked, or which map was used.
    pub notes: Vec<String>,
    /// The commit the coverage map was built on, when one was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

impl DriftReport {
    fn finish(mut self) -> Self {
        self.pass = !self.findings.iter().any(|f| f.severity == Severity::Fail);
        self.summary = match self.findings.len() {
            0 => "drift: none".to_string(),
            1 => "drift: 1 finding".to_string(),
            n => format!("drift: {n} findings"),
        };
        self
    }

    pub fn render(&self) -> String {
        let mut out = format!("{}\n", self.summary);
        for f in &self.findings {
            let col = match f.severity {
                Severity::Fail => "fail   ",
                Severity::Warn => "warn   ",
                Severity::Info => "info   ",
            };
            out.push_str(&format!("{col} {}\n", f.text()));
        }
        for n in &self.notes {
            out.push_str(&format!("{n}\n"));
        }
        out
    }
}

pub struct Options {
    pub root: PathBuf,
    pub spec_root: PathBuf,
    pub base: String,
}

/// Compare the change against its scenarios and the coverage map (DRIFT-005..012).
pub fn run(opts: &Options) -> Result<DriftReport> {
    let root = &opts.root;
    let config = crate::config::load(root)?;
    let prep = crate::gate::prepare(root, &opts.spec_root, &config, &opts.base)?;
    let severity = if config.drift_fail {
        Severity::Fail
    } else {
        Severity::Warn
    };
    let rel = |p: &Path| -> String {
        p.strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let changed: BTreeSet<&str> = prep.changed_files.iter().map(String::as_str).collect();
    let test_files = |id: &str| -> Vec<String> {
        prep.report
            .mapped
            .iter()
            .filter(|m| m.id == id)
            .flat_map(|m| m.tests.iter().map(|t| rel(&t.file)))
            .collect()
    };

    let mut report = DriftReport::default();
    // DRIFT-007: a modified scenario none of whose tests changed.
    for (id, change) in &prep.changes {
        if *change != crate::gate::Change::Modified {
            continue;
        }
        let files = test_files(id);
        if !files.is_empty() && !files.iter().any(|f| changed.contains(f.as_str())) {
            report.findings.push(DriftFinding {
                kind: "stale-test".into(),
                id: id.clone(),
                file: None,
                line: None,
                severity,
                commit: None,
            });
        }
    }

    let Some(map) = CoverageMap::load(root)? else {
        report
            .notes
            .push("no coverage map: code drift not checked (run sj coverage map)".to_string());
        return Ok(report.finish());
    };
    report.commit = Some(map.commit.clone());
    let git = Git::new(root);
    // DRIFT-009: a map from an older commit is translated to the base.
    let map = if map.commit == prep.mb {
        map
    } else {
        let short = &map.commit[..map.commit.len().min(7)];
        let ancestor = git
            .run(&["merge-base", "--is-ancestor", &map.commit, &prep.mb])
            .is_ok();
        if !ancestor {
            report.notes.push(format!(
                "coverage map from {short} is not an ancestor of base: code drift not checked (run sj coverage map)"
            ));
            report.commit = None;
            return Ok(report.finish());
        }
        let behind: usize = git
            .run(&[
                "rev-list",
                "--count",
                &format!("{}..{}", map.commit, prep.mb),
            ])?
            .trim()
            .parse()
            .unwrap_or(0);
        report.notes.push(format!(
            "coverage map from {short}, {behind} commit{} behind base",
            if behind == 1 { "" } else { "s" }
        ));
        translate(&git, map, &prep.mb)?
    };

    // DRIFT-005/006: base lines the change modifies or deletes, in files the map knows.
    let mapped_files: BTreeSet<&str> = map
        .scenarios
        .values()
        .flat_map(|files| files.keys().map(String::as_str))
        .collect();
    // First changed line per scenario.
    let mut first: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for file in prep
        .changed_files
        .iter()
        .filter(|f| mapped_files.contains(f.as_str()))
    {
        let mut lines: Vec<usize> = git
            .hunks(&prep.mb, None, file)?
            .iter()
            .flat_map(|h| h.old_lines())
            .collect();
        lines.sort();
        for line in lines {
            for id in map.owners(file, line) {
                first
                    .entry(id.to_string())
                    .or_insert_with(|| (file.clone(), line));
            }
        }
    }
    // DRIFT-011: a refactor says behaviour did not change, so code drift is information.
    let refactor = prep.exemptions.iter().any(|e| e.kind == "refactor");
    for (id, (file, line)) in first {
        let text_unchanged = matches!(
            prep.changes.get(&id),
            None | Some(crate::gate::Change::Unchanged | crate::gate::Change::Implemented)
        );
        let tests_changed = test_files(&id).iter().any(|f| changed.contains(f.as_str()));
        if text_unchanged && !tests_changed {
            report.findings.push(DriftFinding {
                kind: "code-drift".into(),
                id,
                file: Some(file),
                line: Some(line),
                severity: if refactor { Severity::Info } else { severity },
                commit: report.commit.clone(),
            });
        }
    }
    Ok(report.finish())
}

/// Where line `line` of the `from` version lands after `hunks` (from `from` to `to`), or
/// `None` when a hunk modified or deleted it.
pub fn shift(line: usize, hunks: &[crate::git::Hunk]) -> Option<usize> {
    let mut offset: i64 = 0;
    for h in hunks {
        if h.old_len == 0 {
            // A pure insertion after `old_start`.
            if line <= h.old_start {
                break;
            }
        } else {
            if line < h.old_start {
                break;
            }
            if line < h.old_start + h.old_len {
                return None;
            }
        }
        offset += h.new_len as i64 - h.old_len as i64;
    }
    usize::try_from(line as i64 + offset).ok()
}

/// The map with every range moved from its commit to `to`; lines changed in between are
/// dropped (DRIFT-009).
fn translate(git: &Git, mut map: CoverageMap, to: &str) -> Result<CoverageMap> {
    let files: BTreeSet<String> = map
        .scenarios
        .values()
        .flat_map(|f| f.keys().cloned())
        .collect();
    let mut hunks: BTreeMap<String, Vec<crate::git::Hunk>> = BTreeMap::new();
    for f in files {
        let mut h = git.hunks(&map.commit, Some(to), &f)?;
        h.sort_by_key(|h| h.old_start);
        hunks.insert(f, h);
    }
    for owned in map.scenarios.values_mut() {
        for (file, rs) in owned.iter_mut() {
            let h = &hunks[file];
            let lines: BTreeSet<usize> = rs
                .iter()
                .flat_map(|[a, b]| *a..=*b)
                .filter_map(|l| shift(l, h))
                .collect();
            *rs = ranges(&lines);
        }
    }
    map.commit = to.to_string();
    Ok(map)
}
