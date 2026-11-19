//! `sj audit`: diff-based checks for tests an agent deleted, skipped, focused or weakened
//! (AUDIT-001..013). Works with or without a specjutsu setup.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
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
    pub(crate) fn new(kind: &str, target: String, detail: String, severity: Severity) -> Self {
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
    /// Look up added dependencies on their registries (AUDIT-009, AUDIT-010).
    pub registry: bool,
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
    pub assertions: usize,
}

/// Count assertions in a test's text by its file's language (design decision 3).
pub fn count_assertions(file: &str, text: &str, above: &[&str], helpers: &[String]) -> usize {
    use std::sync::OnceLock;
    static RS: OnceLock<Regex> = OnceLock::new();
    static JS: OnceLock<Regex> = OnceLock::new();
    static PY: OnceLock<Regex> = OnceLock::new();
    let re = if file.ends_with(".rs") {
        RS.get_or_init(|| Regex::new(r"\b(?:debug_)?assert\w*!|\bpanic!").unwrap())
    } else if file.ends_with(".py") {
        PY.get_or_init(|| {
            Regex::new(r"(?m)^\s*assert\b|\bpytest\.raises\b|\bself\.assert\w+\s*\(").unwrap()
        })
    } else {
        JS.get_or_init(|| Regex::new(r"\bexpect\s*\(|\bassert(?:\.\w+)*\s*\(").unwrap())
    };
    let mut n = re.find_iter(text).count();
    if above.iter().any(|l| l.contains("should_panic")) {
        n += 1;
    }
    for h in helpers {
        if let Ok(r) = Regex::new(&format!(r"\b{}\s*[!(]", regex::escape(h))) {
            n += r.find_iter(text).count();
        }
    }
    n
}

impl AuditTest {
    fn from(t: &TestFn, content: &str, helpers: &[String]) -> Self {
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
        let file = t.file.to_string_lossy().replace('\\', "/");
        let assertions = count_assertions(&file, &text, &above, helpers);
        AuditTest {
            file,
            name: t.name.clone(),
            assertions,
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
    helpers: &[String],
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
            out.push(AuditTest::from(t, content, helpers));
        }
    }
    Ok(out)
}

/// Lines of `text` that hold an assertion, by the file's language.
fn assertion_lines<'a>(file: &str, text: &'a str) -> Vec<&'a str> {
    text.lines()
        .filter(|l| count_assertions(file, l, &[], &[]) > 0)
        .collect()
}

/// A line's literals (numbers and quoted strings), and the line with each replaced by `#`.
fn literals(line: &str) -> (String, Vec<String>) {
    use std::sync::OnceLock;
    static LIT: OnceLock<Regex> = OnceLock::new();
    let re = LIT.get_or_init(|| {
        Regex::new(r#""(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\b-?\d+(?:\.\d+)?\b"#).unwrap()
    });
    let lits = re.find_iter(line).map(|m| m.as_str().to_string()).collect();
    (re.replace_all(line, "#").trim().to_string(), lits)
}

/// The first expected literal that changed between two versions of a test, if the assertion
/// lines are otherwise the same.
fn rewritten(file: &str, base: &str, head: &str) -> Option<(String, String)> {
    let (b, h) = (assertion_lines(file, base), assertion_lines(file, head));
    if b.len() != h.len() {
        return None;
    }
    for (bl, hl) in b.iter().zip(&h) {
        let (bs, blits) = literals(bl);
        let (hs, hlits) = literals(hl);
        if bs == hs && blits != hlits {
            let (o, n) = blits.iter().zip(&hlits).find(|(o, n)| o != n)?;
            return Some((o.clone(), n.clone()));
        }
    }
    None
}

/// The test checks over two sides of a change. `source_changed` says whether the change also
/// edits implementation files (AUDIT-008).
pub fn check_tests(base: &[AuditTest], head: &[AuditTest], source_changed: bool) -> Vec<Finding> {
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
        if source_changed && h.text != b.text {
            if let Some((old, new)) = rewritten(&h.file, &b.text, &h.text) {
                out.push(Finding::new(
                    "rewritten-expectation",
                    h.target(),
                    format!(": {old} → {new}"),
                    Severity::Warn,
                ));
            }
        }
        if h.assertions < b.assertions {
            out.push(Finding::new(
                "weakened-test",
                h.target(),
                format!("({} → {} assertions)", b.assertions, h.assertions),
                Severity::Fail,
            ));
        }
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
    let base_bodies: BTreeSet<&str> = base.iter().map(|t| t.body_key.as_str()).collect();
    for h in head {
        let new = !base_by_name.contains_key(&(h.file.as_str(), h.name.as_str()))
            && !base_bodies.contains(h.body_key.as_str());
        if new && h.assertions == 0 {
            out.push(Finding::new(
                "assertion-free-test",
                h.target(),
                String::new(),
                Severity::Fail,
            ));
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

/// A `Specjutsu-Allow: <kind> <target> — <reason>` trailer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allow {
    pub kind: String,
    pub target: String,
    pub reason: String,
}

/// Parse every `Specjutsu-Allow:` line in `text`. The reason follows the first spaced dash
/// (` — `, ` – ` or ` - `), so paths and names with hyphens stay whole.
pub fn parse_allows(text: &str) -> Vec<Allow> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("Specjutsu-Allow:"))
        .filter_map(|rest| {
            let rest = rest.trim();
            let cut = [" — ", " – ", " - "]
                .iter()
                .filter_map(|sep| rest.find(sep).map(|i| (i, sep.len())))
                .min();
            let (what, reason) = match cut {
                Some((i, n)) => (&rest[..i], rest[i + n..].trim()),
                None => (rest, ""),
            };
            let (kind, target) = what.trim().split_once(' ')?;
            Some(Allow {
                kind: kind.to_string(),
                target: target.trim().to_string(),
                reason: reason.to_string(),
            })
        })
        .collect()
}

/// Mark findings named by an allow as allowed; an allow without a reason is a failure.
pub fn apply_allows(findings: &mut Vec<Finding>, allows: &[Allow]) {
    for a in allows {
        if a.reason.is_empty() {
            findings.push(Finding::new(
                "allow without reason",
                format!("{} {}", a.kind, a.target),
                String::new(),
                Severity::Fail,
            ));
            continue;
        }
        for f in findings.iter_mut() {
            if f.kind == a.kind && f.target == a.target && f.severity != Severity::Allowed {
                f.severity = Severity::Allowed;
                f.reason = a.reason.clone();
            }
        }
    }
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
    let helpers = config
        .as_ref()
        .map(|c| c.assert_helpers.clone())
        .unwrap_or_default();
    let base = scan_side(root, config.as_ref(), &changed, &helpers, |rel| {
        base_files
            .contains(rel)
            .then(|| git.show(&mb, rel).ok())
            .flatten()
    })?;
    let head = scan_side(root, config.as_ref(), &changed, &helpers, |rel| {
        std::fs::read_to_string(root.join(rel)).ok()
    })?;
    let classifier = match &config {
        Some(c) => crate::gate::Classifier::new(c, &c.spec_root)?,
        None => crate::gate::Classifier::builtin(crate::config::DEFAULT_SPEC_ROOT),
    };
    let test_files: BTreeSet<&str> = base.iter().chain(&head).map(|t| t.file.as_str()).collect();
    let source_changed = changed
        .iter()
        .any(|f| !test_files.contains(f.as_str()) && classifier.is_impl(f));
    let mut findings = check_tests(&base, &head, source_changed);
    if opts.registry {
        let mut added = BTreeSet::new();
        for rel in &changed {
            let Some(eco) = crate::registry::Ecosystem::for_manifest(rel) else {
                continue;
            };
            let before = base_files
                .contains(rel)
                .then(|| git.show(&mb, rel).ok())
                .flatten()
                .map(|t| crate::registry::dependencies(eco, &t))
                .unwrap_or_default();
            let after = std::fs::read_to_string(root.join(rel))
                .map(|t| crate::registry::dependencies(eco, &t))
                .unwrap_or_default();
            for name in after.difference(&before) {
                added.insert((eco, name.clone()));
            }
        }
        findings.extend(crate::registry::check(&added));
    }
    let allows: Vec<Allow> = git
        .log(&format!("{mb}..HEAD"))?
        .iter()
        .flat_map(|c| parse_allows(&c.body))
        .collect();
    apply_allows(&mut findings, &allows);
    Ok(AuditReport::from_findings(findings))
}
