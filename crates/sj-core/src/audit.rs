//! `sj audit`: diff-based checks for tests an agent deleted, skipped, focused or weakened
//! (AUDIT-001..013). Works with or without a specjutsu setup.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::{Config, FILE as CONFIG_FILE};
use crate::git::Git;
use crate::testscan::{self, TestFn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Fail,
    Warn,
    Allowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// `removed-test`, `skipped-test`, …
    pub kind: String,
    /// `<file> › <test name>`, or a package for dependency checks.
    pub target: String,
    /// Extra text after the target, e.g. `(5 → 2 assertions)`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub detail: String,
    pub severity: Severity,
    /// The reason from a `Specjutsu-Allow:` trailer, for allowed findings.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

impl Finding {
    fn new(kind: &str, target: String, detail: String, severity: Severity) -> Self {
        Finding {
            kind: kind.to_string(),
            target,
            detail,
            severity,
            reason: String::new(),
        }
    }

    /// `<kind> <target><detail>`, as printed after the severity column.
    pub fn line(&self) -> String {
        let mut s = format!("{} {}", self.kind, self.target);
        if !self.detail.is_empty() {
            if !self.detail.starts_with(':') {
                s.push(' ');
            }
            s.push_str(&self.detail);
        }
        if self.severity == Severity::Allowed {
            s.push_str(&format!(" ({})", self.reason));
        }
        s
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AuditReport {
    pub pass: bool,
    pub summary: String,
    pub findings: Vec<Finding>,
}

impl AuditReport {
    fn from_findings(findings: Vec<Finding>) -> Self {
        let n = |s: Severity| findings.iter().filter(|f| f.severity == s).count();
        let (fails, warns) = (n(Severity::Fail), n(Severity::Warn));
        let summary = if findings.is_empty() {
            "audit: clean".to_string()
        } else if fails > 0 {
            let mut s = format!("audit: fail — {}", plural(fails, "problem"));
            if warns > 0 {
                s.push_str(&format!(", {}", plural(warns, "warning")));
            }
            s
        } else if warns > 0 {
            format!("audit: pass — {}", plural(warns, "warning"))
        } else {
            format!("audit: pass — {} allowed", findings.len())
        };
        AuditReport {
            pass: fails == 0,
            summary,
            findings,
        }
    }

    /// The compact report: the summary, then one line per finding.
    pub fn render(&self) -> String {
        let mut out = format!("{}\n", self.summary);
        for f in &self.findings {
            let col = match f.severity {
                Severity::Fail => "fail   ",
                Severity::Warn => "warn   ",
                Severity::Allowed => "allowed",
            };
            out.push_str(&format!("{col} {}\n", f.line()));
        }
        out
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

pub struct Options {
    pub root: PathBuf,
    pub base: String,
}

/// A test with the text the checks read.
#[derive(Debug, Clone)]
pub struct AuditTest {
    pub file: String,
    pub name: String,
    /// Declaration through the closing line.
    pub text: String,
    /// The test's text with its own name removed and whitespace dropped, so a moved or renamed
    /// test with the same body matches.
    pub body_key: String,
    pub skipped: bool,
    pub focused: bool,
    pub marker_group: Option<String>,
}

impl AuditTest {
    fn from(t: &TestFn, content: &str) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        let start = t.line.saturating_sub(1).min(lines.len());
        let end = t.end_line.max(t.line).min(lines.len());
        let text = lines[start..end].join("\n");
        let last = t.name.rsplit([' ', ':']).next().unwrap_or(&t.name);
        let own = t.name.rsplit(" > ").next().unwrap_or(&t.name);
        let body_key: String = text
            .replace(own, "")
            .replace(last, "")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        // Attributes and decorators directly above the declaration (Rust `#[ignore]`).
        let above: Vec<&str> = lines[..start]
            .iter()
            .rev()
            .take_while(|l| {
                let l = l.trim();
                l.starts_with("#[") || l.starts_with("//") || l.starts_with('@')
            })
            .copied()
            .collect();
        let skipped = t.skipped
            || above.iter().any(|l| l.trim().starts_with("#[ignore"))
            || text.contains("pytest.skip(");
        AuditTest {
            file: t.file.to_string_lossy().replace('\\', "/"),
            name: t.name.clone(),
            text,
            body_key,
            skipped,
            focused: t.focused,
            marker_group: t.marker_group.clone(),
        }
    }

    pub fn target(&self) -> String {
        format!("{} › {}", self.file, self.name)
    }
}

/// The scanner for a file under the default globs used when there is no `specjutsu.toml`.
fn default_scanner(rel: &str) -> Option<fn(&Path, &str) -> Vec<TestFn>> {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if rel
        .split('/')
        .any(|d| matches!(d, "target" | "node_modules" | ".git"))
    {
        return None;
    }
    if name.ends_with(".rs") {
        return Some(testscan::scan_rust);
    }
    let js = ["ts", "tsx", "js", "jsx", "mjs", "cjs"];
    if let Some((stem, ext)) = name.rsplit_once('.') {
        if js.contains(&ext) && (stem.ends_with(".test") || stem.ends_with(".spec")) {
            return Some(testscan::scan_js);
        }
        if ext == "py" && (stem.starts_with("test_") || stem.ends_with("_test")) {
            return Some(testscan::scan_python);
        }
    }
    None
}

/// Tests in `paths`, read with `read` (which returns `None` for a file absent on that side).
fn scan_side(
    root: &Path,
    config: Option<&Config>,
    paths: &[String],
    read: impl Fn(&str) -> Option<String>,
) -> Result<Vec<AuditTest>> {
    let mut out = Vec::new();
    let mut contents: BTreeMap<String, String> = BTreeMap::new();
    for rel in paths {
        if let Some(c) = read(rel) {
            contents.insert(rel.clone(), c);
        }
    }
    let present: Vec<String> = contents.keys().cloned().collect();
    let tests: Vec<TestFn> = match config {
        Some(cfg) => testscan::scan_listed(root, cfg, &present, |rel| {
            Ok(contents.get(rel).cloned().unwrap_or_default())
        })?,
        None => present
            .iter()
            .filter_map(|rel| default_scanner(rel).map(|scan| scan(Path::new(rel), &contents[rel])))
            .flatten()
            .collect(),
    };
    for t in &tests {
        let rel = t.file.to_string_lossy().replace('\\', "/");
        if let Some(content) = contents.get(&rel) {
            out.push(AuditTest::from(t, content));
        }
    }
    Ok(out)
}

/// The test checks over two sides of a change.
pub fn check_tests(base: &[AuditTest], head: &[AuditTest]) -> Vec<Finding> {
    let head_by_name: BTreeSet<(&str, &str)> = head
        .iter()
        .map(|t| (t.file.as_str(), t.name.as_str()))
        .collect();
    let head_bodies: BTreeSet<&str> = head.iter().map(|t| t.body_key.as_str()).collect();
    let base_by_name: BTreeMap<(&str, &str), &AuditTest> = base
        .iter()
        .map(|t| ((t.file.as_str(), t.name.as_str()), t))
        .collect();
    let mut out = Vec::new();
    // A marker on a group is reported once for the group (AUDIT-004).
    let mut marked: BTreeSet<(String, String)> = BTreeSet::new();
    for h in head {
        let Some(b) = base_by_name.get(&(h.file.as_str(), h.name.as_str())) else {
            continue;
        };
        for (kind, now, before) in [
            ("skipped-test", h.skipped, b.skipped),
            ("focused-test", h.focused, b.focused),
        ] {
            if now && !before {
                let target = match &h.marker_group {
                    Some(g) => format!("{} › {g}", h.file),
                    None => h.target(),
                };
                if marked.insert((kind.to_string(), target.clone())) {
                    out.push(Finding::new(kind, target, String::new(), Severity::Fail));
                }
            }
        }
    }
    for b in base {
        let kept = head_by_name.contains(&(b.file.as_str(), b.name.as_str()));
        if !kept && !head_bodies.contains(b.body_key.as_str()) {
            out.push(Finding::new(
                "removed-test",
                b.target(),
                String::new(),
                Severity::Fail,
            ));
        }
    }
    out
}

/// Run the audit between the merge base of `opts.base` and the working tree.
pub fn run(opts: &Options) -> Result<AuditReport> {
    let root = &opts.root;
    let git = Git::new(root);
    git.resolve(&opts.base).with_context(|| {
        format!(
            "base ref `{}` not found (in CI, fetch it: actions/checkout with fetch-depth: 0)",
            opts.base
        )
    })?;
    let mb = git.merge_base(&opts.base, "HEAD")?;
    let changed = git.changed_files(&mb)?;
    let config = if root.join(CONFIG_FILE).exists() {
        Some(crate::config::load(root)?)
    } else {
        None
    };
    let base_files: BTreeSet<String> = git.ls_tree(&mb)?.into_iter().collect();
    let base = scan_side(root, config.as_ref(), &changed, |rel| {
        base_files
            .contains(rel)
            .then(|| git.show(&mb, rel).ok())
            .flatten()
    })?;
    let head = scan_side(root, config.as_ref(), &changed, |rel| {
        std::fs::read_to_string(root.join(rel)).ok()
    })?;
    Ok(AuditReport::from_findings(check_tests(&base, &head)))
}
