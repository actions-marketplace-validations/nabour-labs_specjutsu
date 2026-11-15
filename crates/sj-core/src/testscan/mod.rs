//! Test discovery: per-language scanners that find test functions and the scenario IDs they name.

mod js;
mod python;

pub use js::scan_js;
pub use python::scan_python;

use std::path::{Path, PathBuf};

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::config::{Config, Package, Runner};

/// A test found in source.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestFn {
    /// Function name (Rust, pytest `Class::func`) or titles joined with ` > ` (JS/TS).
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    /// Explicit IDs: `spec:` comments above the test and `@pytest.mark.spec` markers.
    pub comment_ids: Vec<String>,
    /// Implicit IDs read from the name or titles; they only count as orphans for known prefixes.
    pub name_ids: Vec<String>,
    /// Marked skip/todo/fixme (or inside a skipped group).
    pub skipped: bool,
    /// Owning package path, when found through the project config.
    pub package: Option<String>,
    /// Suite runner, when found through the project config.
    pub suite: Option<Runner>,
}

/// Find test functions (`fn` items carrying a `#[...test...]` attribute) in one Rust file.
pub fn scan_rust(file: &Path, content: &str) -> Vec<TestFn> {
    static ATTR: OnceLock<Regex> = OnceLock::new();
    static FN: OnceLock<Regex> = OnceLock::new();
    static SPEC: OnceLock<Regex> = OnceLock::new();
    let attr = ATTR.get_or_init(|| Regex::new(r"^\s*#\[").unwrap());
    let test_attr = |l: &str| attr.is_match(l) && l.contains("test") && !l.contains("cfg(test)");
    let fn_re = FN.get_or_init(|| {
        Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
            .unwrap()
    });
    let spec_re = SPEC.get_or_init(|| {
        Regex::new(r"^\s*//+\s*spec:\s*([A-Z][A-Z0-9]*-[0-9]+(?:\s*,\s*[A-Z][A-Z0-9]*-[0-9]+)*)")
            .unwrap()
    });

    let mut out = Vec::new();
    // State gathered from the run of attribute/comment/blank lines above the current line.
    let mut pending_test = false;
    let mut pending_ids: Vec<String> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if let Some(c) = spec_re.captures(line) {
            pending_ids.extend(c[1].split(',').map(|s| s.trim().to_string()));
            continue;
        }
        if test_attr(line) {
            pending_test = true;
            continue;
        }
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || attr.is_match(line) {
            continue;
        }
        if let Some(c) = fn_re.captures(line) {
            if pending_test {
                out.push(TestFn {
                    name: c[1].to_string(),
                    file: file.to_path_buf(),
                    line: i + 1,
                    comment_ids: std::mem::take(&mut pending_ids),
                    name_ids: ids_in_name(&c[1]),
                    ..TestFn::default()
                });
            }
        }
        pending_test = false;
        pending_ids.clear();
    }
    out
}

/// Walk `root` for `*.rs` files (respecting .gitignore, skipping `target/` and `fixtures/`) and scan each.
pub fn scan_tree(root: &Path) -> Result<Vec<TestFn>> {
    let walker = ignore::WalkBuilder::new(root)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(|e| {
            e.depth() == 0
                || !(e.file_type().is_some_and(|t| t.is_dir())
                    && matches!(e.file_name().to_str(), Some("target" | "fixtures")))
        })
        .build();
    let mut out = Vec::new();
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_some_and(|t| t.is_file())
            && path.extension().is_some_and(|x| x == "rs")
        {
            let content = std::fs::read_to_string(path)?;
            out.extend(scan_rust(path, &content));
        }
    }
    Ok(out)
}

/// Tests found through the project config, and the packages that have source files but no suite.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scan {
    pub tests: Vec<TestFn>,
    pub unsuited: Vec<String>,
}

/// Directories never searched for tests.
const SKIP_DIRS: [&str; 6] = [
    "target",
    "node_modules",
    ".venv",
    "venv",
    "fixtures",
    ".git",
];

/// One suite's compiled globs, with the literal prefix length of each for tie-breaking.
struct SuiteGlobs {
    runner: Runner,
    set: GlobSet,
    prefixes: Vec<usize>,
}

/// Walk `root` once and scan every file that a suite of its owning package matches.
///
/// A file belongs to the package with the longest matching path. Within that package, the suite
/// whose matching glob has the longest literal prefix wins; config order breaks ties.
pub fn scan_project(root: &Path, config: &Config) -> Result<Scan> {
    let mut suites: Vec<Vec<SuiteGlobs>> = Vec::with_capacity(config.packages.len());
    let mut sources: Vec<GlobSet> = Vec::with_capacity(config.packages.len());
    for p in &config.packages {
        let mut v = Vec::new();
        for s in &p.suites {
            let globs = if s.tests.is_empty() {
                default_globs(root, p, s.runner)
            } else {
                s.tests.clone()
            };
            let (set, prefixes) = compile(&globs)?;
            v.push(SuiteGlobs {
                runner: s.runner,
                set,
                prefixes,
            });
        }
        suites.push(v);
        sources.push(compile(&p.sources)?.0);
    }

    let mut scan = Scan::default();
    let mut has_source = vec![false; config.packages.len()];
    let walker = ignore::WalkBuilder::new(root)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(|e| {
            e.depth() == 0
                || !(e.file_type().is_some_and(|t| t.is_dir())
                    && e.file_name()
                        .to_str()
                        .is_some_and(|n| SKIP_DIRS.contains(&n)))
        })
        .build();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let Some(pkg) = config.package_for(&rel) else {
            continue;
        };
        let idx = config
            .packages
            .iter()
            .position(|p| std::ptr::eq(p, pkg))
            .unwrap_or_default();
        if suites[idx].is_empty() {
            if sources[idx].is_match(&rel) {
                has_source[idx] = true;
            }
            continue;
        }
        let Some(runner) = pick_suite(&suites[idx], &rel) else {
            continue;
        };
        let scanner: fn(&Path, &str) -> Vec<TestFn> = match runner {
            Runner::Cargo if rel.ends_with(".rs") => scan_rust,
            Runner::Cargo => continue,
            Runner::Vitest | Runner::Jest | Runner::Playwright => scan_js,
            Runner::Pytest if rel.ends_with(".py") => scan_python,
            Runner::Pytest => continue,
        };
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel_path = PathBuf::from(&rel);
        for mut t in scanner(&rel_path, &content) {
            t.package = Some(pkg.path.clone());
            t.suite = Some(runner);
            scan.tests.push(t);
        }
    }
    scan.unsuited = config
        .packages
        .iter()
        .zip(has_source)
        .filter(|(_, has)| *has)
        .map(|(p, _)| p.path.clone())
        .collect();
    Ok(scan)
}

fn pick_suite(suites: &[SuiteGlobs], rel: &str) -> Option<Runner> {
    let mut best: Option<(usize, Runner)> = None;
    for s in suites {
        let longest = s.set.matches(rel).into_iter().map(|i| s.prefixes[i]).max();
        if let Some(len) = longest {
            if best.map_or(true, |(b, _)| len > b) {
                best = Some((len, s.runner));
            }
        }
    }
    best.map(|(_, r)| r)
}

/// A runner's default test globs under a package, relative to the repository root.
fn default_globs(root: &Path, pkg: &Package, runner: Runner) -> Vec<String> {
    let under = |g: &str| {
        if pkg.path == "." {
            g.to_string()
        } else {
            format!("{}/{g}", pkg.path)
        }
    };
    match runner {
        Runner::Cargo => vec![under("**/*.rs")],
        Runner::Vitest | Runner::Jest => vec![under("**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}")],
        Runner::Pytest => vec![under("**/test_*.py"), under("**/*_test.py")],
        Runner::Playwright => {
            let test_dir = crate::adopt::config_file(root, &pkg.path, "playwright.config")
                .and_then(|f| std::fs::read_to_string(f).ok())
                .and_then(|t| crate::adopt::playwright_test_dir(&t))
                .unwrap_or_else(|| "e2e".into());
            vec![under(&format!("{test_dir}/**/*.spec.ts"))]
        }
    }
}

fn compile(globs: &[String]) -> Result<(GlobSet, Vec<usize>)> {
    let mut b = GlobSetBuilder::new();
    let mut prefixes = Vec::with_capacity(globs.len());
    for g in globs {
        b.add(GlobBuilder::new(g).literal_separator(true).build()?);
        prefixes.push(g.find(['*', '?', '[', '{']).unwrap_or(g.len()));
    }
    Ok((b.build()?, prefixes))
}

/// Candidate IDs embedded in a test name: an alphabetic segment followed by a numeric segment,
/// e.g. `test_auth_003_x` → `AUTH-003`.
pub fn ids_in_name(name: &str) -> Vec<String> {
    let segs: Vec<&str> = name.split('_').collect();
    segs.windows(2)
        .filter(|w| {
            !w[0].is_empty()
                && w[0].chars().all(|c| c.is_ascii_alphabetic())
                && !w[1].is_empty()
                && w[1].chars().all(|c| c.is_ascii_digit())
        })
        .map(|w| format!("{}-{}", w[0].to_ascii_uppercase(), w[1]))
        .collect()
}
