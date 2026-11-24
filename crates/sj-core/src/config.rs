//! The project file, `specjutsu.toml`: spec root, mode and packages with their test suites.
//!
//! Parsing happens in two stages. Raw structs reject unknown keys with their position, then
//! validation turns them into the typed model and names the package in every error.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// The project file's name, at the repository root.
pub const FILE: &str = "specjutsu.toml";

/// The `.gitignore` line that keeps the local run ledger out of commits (CONF-038).
pub const LEDGER_IGNORE: &str = ".specjutsu/ledger/";

/// What [`ignore_ledger`] would do, or did, to the root `.gitignore`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerIgnore {
    /// A line already ignores the ledger.
    Present,
    /// The line is appended to an existing file.
    Change,
    /// The file is created with the line.
    Create,
}

/// What adding the ledger line to `<root>/.gitignore` involves, without writing.
pub fn ledger_ignore(root: &Path) -> LedgerIgnore {
    match std::fs::read_to_string(root.join(".gitignore")) {
        Err(_) => LedgerIgnore::Create,
        Ok(text) => {
            let covered = text.lines().map(str::trim).any(|l| {
                matches!(
                    l.strip_prefix('/').unwrap_or(l),
                    ".specjutsu/ledger/" | ".specjutsu/ledger" | ".specjutsu/" | ".specjutsu"
                )
            });
            if covered {
                LedgerIgnore::Present
            } else {
                LedgerIgnore::Change
            }
        }
    }
}

/// Add [`LEDGER_IGNORE`] to `<root>/.gitignore` unless a line already covers it, keeping the
/// file's other lines (CONF-038).
pub fn ignore_ledger(root: &Path) -> Result<LedgerIgnore> {
    let action = ledger_ignore(root);
    let path = root.join(".gitignore");
    match action {
        LedgerIgnore::Present => {}
        LedgerIgnore::Create => std::fs::write(&path, format!("{LEDGER_IGNORE}\n"))?,
        LedgerIgnore::Change => {
            let mut text = std::fs::read_to_string(&path)?;
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(LEDGER_IGNORE);
            text.push('\n');
            std::fs::write(&path, text)?;
        }
    }
    Ok(action)
}

/// The only schema version this build reads and writes.
pub const SCHEMA: u32 = 1;

/// The spec root used when the file is missing or does not set one.
pub const DEFAULT_SPEC_ROOT: &str = "specjutsu";

/// The spec root of projects created before `specjutsu` became the default.
pub const LEGACY_SPEC_ROOT: &str = "openspec";

/// The spec root for `root` when `spec_root` is not set: `specjutsu`, unless the repository has
/// an `openspec/` folder and no `specjutsu/` folder.
pub fn default_spec_root(root: &Path) -> String {
    if !root.join(DEFAULT_SPEC_ROOT).is_dir() && root.join(LEGACY_SPEC_ROOT).is_dir() {
        LEGACY_SPEC_ROOT.into()
    } else {
        DEFAULT_SPEC_ROOT.into()
    }
}

/// Folder names that hold test helpers and fixtures rather than product code. `sj adopt`
/// declares them as `support` globs (CONF-039), and the gate checks files the change added or
/// modified under them for helpers missing on base (GATE-056/062). The gate does not treat them
/// as support: only files under `tests/` and those a `support` glob matches are overlaid.
pub const HELPER_DIRS: [&str; 8] = [
    "testing",
    "test-utils",
    "test-helpers",
    "testutils",
    "fixtures",
    "__fixtures__",
    "mocks",
    "__mocks__",
];

/// Whether a path segment names a helper folder (see [`HELPER_DIRS`]).
pub fn is_helper_dir(segment: &str) -> bool {
    HELPER_DIRS.contains(&segment)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Advisory,
    #[default]
    Guided,
    Strict,
}

impl Mode {
    const ALL: [Mode; 3] = [Mode::Advisory, Mode::Guided, Mode::Strict];

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Advisory => "advisory",
            Mode::Guided => "guided",
            Mode::Strict => "strict",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    Cargo,
    Jest,
    Junit,
    Playwright,
    Pytest,
    Vitest,
}

impl Runner {
    /// Every known runner, in alphabetical order.
    pub const ALL: [Runner; 6] = [
        Runner::Cargo,
        Runner::Jest,
        Runner::Junit,
        Runner::Playwright,
        Runner::Pytest,
        Runner::Vitest,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Runner::Cargo => "cargo",
            Runner::Jest => "jest",
            Runner::Junit => "junit",
            Runner::Playwright => "playwright",
            Runner::Pytest => "pytest",
            Runner::Vitest => "vitest",
        }
    }

    fn known_list() -> String {
        Runner::ALL.map(Runner::as_str).join(", ")
    }
}

/// Whether the gate runs a suite's target tests on the base code.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RedOnBase {
    #[default]
    Run,
    /// Only an assertion failure on base counts as red (GATE-061).
    Assert,
    Skip,
}

impl RedOnBase {
    pub const ALL: [RedOnBase; 3] = [RedOnBase::Run, RedOnBase::Assert, RedOnBase::Skip];

    pub fn as_str(self) -> &'static str {
        match self {
            RedOnBase::Run => "run",
            RedOnBase::Assert => "assert",
            RedOnBase::Skip => "skip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suite {
    pub runner: Runner,
    /// Globs relative to the repository root.
    pub tests: Vec<String>,
    /// Replaces the runner's launcher (built-in runners) or is the whole command (`junit`).
    pub command: Option<String>,
    /// `junit` only: the report the command writes, relative to the package directory.
    pub report: Option<String>,
    /// Extra environment for the run.
    pub env: BTreeMap<String, String>,
    /// Run in the base package directory before the gate's first base run of this suite.
    pub setup: Option<String>,
    /// Root-relative globs of helpers and fixtures the gate overlays on base with the tests.
    pub support: Vec<String>,
    pub red_on_base: RedOnBase,
    /// Why red-on-base is skipped (required with `red_on_base = "skip"`).
    pub reason: Option<String>,
    /// A command that runs the selected tests and writes LCOV to `{lcov}` (DRIFT-001).
    pub coverage: Option<String>,
}

impl Suite {
    pub fn new(runner: Runner, tests: Vec<String>) -> Suite {
        Suite {
            runner,
            tests,
            command: None,
            report: None,
            env: BTreeMap::new(),
            setup: None,
            support: Vec::new(),
            red_on_base: RedOnBase::Run,
            reason: None,
            coverage: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Package {
    /// Forward-slash path relative to the repository root; `.` is the root.
    pub path: String,
    /// Globs relative to the repository root.
    pub sources: Vec<String>,
    pub suites: Vec<Suite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Config {
    pub schema: u32,
    /// Forward-slash path relative to the repository root.
    pub spec_root: String,
    pub mode: Mode,
    /// Root-relative globs of files that never need a scenario, on top of the built-in list.
    pub non_source: Vec<String>,
    /// `[ids.prefixes]`: capability folder name to the prefix `sj ids` gives its new scenarios.
    pub id_prefixes: BTreeMap<String, String>,
    pub packages: Vec<Package>,
    /// `[audit] assert_helpers`: functions or macros `sj audit` counts as assertions.
    pub assert_helpers: Vec<String>,
    /// `[drift] fail`: drift findings fail the gate (DRIFT-010).
    pub drift_fail: bool,
    /// `[drift] shared_threshold` in thousandths (default 500): lines run by more than this
    /// share of mapped scenarios are shared, not owned (DRIFT-003).
    pub drift_shared_permille: u32,
    /// `language`: the BCP-47 tag specs are written in (CONF-012); `None` means English.
    pub language: Option<String>,
}

/// The default `[drift] shared_threshold`, in thousandths.
pub const DRIFT_SHARED_PERMILLE: u32 = 500;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema: Option<u32>,
    spec_root: Option<String>,
    mode: Option<String>,
    language: Option<String>,
    #[serde(default)]
    non_source: Vec<String>,
    ids: Option<RawIds>,
    audit: Option<RawAudit>,
    drift: Option<RawDrift>,
    #[serde(default, rename = "package")]
    packages: Vec<RawPackage>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDrift {
    #[serde(default)]
    fail: bool,
    shared_threshold: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAudit {
    #[serde(default)]
    assert_helpers: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawIds {
    #[serde(default)]
    prefixes: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackage {
    path: String,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default, rename = "suite")]
    suites: Vec<RawSuite>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSuite {
    runner: String,
    #[serde(default)]
    tests: Vec<String>,
    command: Option<String>,
    report: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    setup: Option<String>,
    #[serde(default)]
    support: Vec<String>,
    red_on_base: Option<String>,
    reason: Option<String>,
    coverage: Option<String>,
}

/// Whether `tag` is a well-formed BCP-47 language tag: a 2–3 letter language, then optional
/// script, region and variant subtags (`fr`, `pt-BR`, `zh-Hant-TW`, `de-CH-1996`).
pub fn is_language_tag(tag: &str) -> bool {
    use std::sync::OnceLock;
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| {
        regex::Regex::new(
            r"^[A-Za-z]{2,3}(-[A-Za-z]{4})?(-(?:[A-Za-z]{2}|[0-9]{3}))?(-(?:[A-Za-z0-9]{5,8}|[0-9][A-Za-z0-9]{3}))*$",
        )
        .unwrap()
    })
    .is_match(tag)
}

/// Load `<root>/specjutsu.toml`, or the defaults for `root` when it does not exist.
pub fn load(root: &Path) -> Result<Config> {
    let path = root.join(FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let mut config = Config::parse(&text)?;
            let set = toml::from_str::<toml::Table>(&text)
                .map(|t| t.contains_key("spec_root"))
                .unwrap_or(true);
            if !set {
                config.spec_root = default_spec_root(root);
            }
            Ok(config)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default_for(root)),
        Err(e) => Err(e).with_context(|| format!("{FILE}: cannot read {}", path.display())),
    }
}

impl Config {
    /// The configuration of a repository that has no project file: spec root from
    /// [`default_spec_root`], mode
    /// `guided`, and one root package with a `cargo` suite when a root `Cargo.toml` exists.
    pub fn default_for(root: &Path) -> Config {
        let packages = if root.join("Cargo.toml").is_file() {
            vec![Package {
                path: ".".into(),
                sources: vec!["**/*.rs".into()],
                suites: vec![Suite::new(Runner::Cargo, vec!["**/*.rs".into()])],
            }]
        } else {
            Vec::new()
        };
        Config {
            schema: SCHEMA,
            spec_root: default_spec_root(root),
            mode: Mode::default(),
            non_source: Vec::new(),
            assert_helpers: Vec::new(),
            drift_fail: false,
            drift_shared_permille: DRIFT_SHARED_PERMILLE,
            language: None,
            id_prefixes: BTreeMap::new(),
            packages,
        }
    }

    /// Parse and validate the text of a project file.
    pub fn parse(text: &str) -> Result<Config> {
        let raw: RawConfig = toml::from_str(text)
            .map_err(|e| anyhow::anyhow!("{FILE}: {}", e.to_string().trim_end()))?;
        let schema = match raw.schema {
            Some(SCHEMA) => SCHEMA,
            Some(n) => bail!("{FILE}: unsupported schema {n} (this sj reads schema {SCHEMA})"),
            None => bail!("{FILE}: missing `schema` (add `schema = {SCHEMA}`)"),
        };
        let spec_root = match raw.spec_root {
            Some(s) => {
                check_path(&s, false).map_err(|why| anyhow::anyhow!("{FILE}: spec_root {why}"))?;
                s
            }
            None => DEFAULT_SPEC_ROOT.into(),
        };
        let mode = match raw.mode.as_deref() {
            None => Mode::default(),
            Some(m) => Mode::ALL
                .into_iter()
                .find(|x| x.as_str() == m)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{FILE}: unknown mode `{m}` (known: {})",
                        Mode::ALL.map(Mode::as_str).join(", ")
                    )
                })?,
        };

        if let Some(tag) = &raw.language {
            if !is_language_tag(tag) {
                bail!("{FILE}: language `{tag}` is not a BCP-47 tag (for example `fr` or `pt-BR`)");
            }
        }

        let id_prefixes = raw.ids.map(|i| i.prefixes).unwrap_or_default();
        for (cap, prefix) in &id_prefixes {
            if !crate::ids::is_prefix(prefix) {
                bail!(
                    "{FILE}: ids.prefixes: `{cap}` maps to `{prefix}`, which is not a prefix \
                     (an uppercase letter, then uppercase letters or digits)"
                );
            }
        }

        let mut packages: Vec<Package> = Vec::with_capacity(raw.packages.len());
        for p in raw.packages {
            check_path(&p.path, true)
                .map_err(|why| anyhow::anyhow!("{FILE}: package path {why}"))?;
            if packages.iter().any(|q| q.path == p.path) {
                bail!("{FILE}: duplicate package path `{}`", p.path);
            }
            let mut suites = Vec::with_capacity(p.suites.len());
            for s in p.suites {
                let runner = Runner::ALL
                    .into_iter()
                    .find(|r| r.as_str() == s.runner)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "{FILE}: package {}: unknown runner `{}` (known: {})",
                            p.path,
                            s.runner,
                            Runner::known_list()
                        )
                    })?;
                let what = format!("{FILE}: package {}: {} suite", p.path, s.runner);
                if s.coverage.as_ref().is_some_and(|c| !c.contains("{lcov}")) {
                    bail!(
                        "{FILE}: suite {} ({}): coverage must contain {{lcov}}",
                        s.runner,
                        p.path
                    );
                }
                match (runner, &s.command, &s.report) {
                    (Runner::Junit, None, _) => bail!("{what} needs `command`"),
                    (Runner::Junit, _, None) => bail!("{what} needs `report`"),
                    (Runner::Junit, _, _) | (_, _, None) => {}
                    (_, _, Some(_)) => bail!("{what}: `report` is only for the junit runner"),
                }
                if let Some(r) = &s.report {
                    check_path(r, false).map_err(|why| anyhow::anyhow!("{what}: report {why}"))?;
                }
                let red_on_base = match s.red_on_base.as_deref() {
                    None => RedOnBase::Run,
                    Some(v) => RedOnBase::ALL
                        .into_iter()
                        .find(|x| x.as_str() == v)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "{what}: unknown red_on_base `{v}` (known: {})",
                                RedOnBase::ALL.map(RedOnBase::as_str).join(", ")
                            )
                        })?,
                };
                match (red_on_base, &s.reason) {
                    (RedOnBase::Skip, None) => {
                        bail!("{what}: red_on_base = \"skip\" needs a `reason`")
                    }
                    (RedOnBase::Run | RedOnBase::Assert, Some(_)) => {
                        bail!("{what}: `reason` is only for red_on_base = \"skip\"")
                    }
                    _ => {}
                }
                suites.push(Suite {
                    runner,
                    tests: s.tests,
                    command: s.command,
                    report: s.report,
                    env: s.env,
                    setup: s.setup,
                    support: s.support,
                    red_on_base,
                    reason: s.reason,
                    coverage: s.coverage,
                });
            }
            packages.push(Package {
                path: p.path,
                sources: p.sources,
                suites,
            });
        }

        let (drift_fail, drift_shared_permille) = match &raw.drift {
            Some(d) => {
                let t = d.shared_threshold.unwrap_or(0.5);
                if !(0.0..=1.0).contains(&t) {
                    bail!("{FILE}: [drift] shared_threshold must be between 0 and 1, got {t}");
                }
                (d.fail, (t * 1000.0).round() as u32)
            }
            None => (false, DRIFT_SHARED_PERMILLE),
        };
        Ok(Config {
            schema,
            spec_root,
            mode,
            non_source: raw.non_source,
            id_prefixes,
            packages,
            assert_helpers: raw.audit.map(|a| a.assert_helpers).unwrap_or_default(),
            drift_fail,
            drift_shared_permille,
            language: raw.language,
        })
    }

    /// Render the project file: fixed key order, one table per package and suite.
    pub fn to_toml(&self) -> String {
        let mut out = format!(
            "schema = {}\nspec_root = {}\nmode = {}\n",
            self.schema,
            quote(&self.spec_root),
            quote(self.mode.as_str())
        );
        if let Some(l) = &self.language {
            out.push_str(&format!("language = {}\n", quote(l)));
        }
        if !self.non_source.is_empty() {
            out.push_str(&format!("non_source = {}\n", list(&self.non_source)));
        }
        if !self.id_prefixes.is_empty() {
            out.push_str("\n[ids.prefixes]\n");
            for (cap, prefix) in &self.id_prefixes {
                out.push_str(&format!("{} = {}\n", quote(cap), quote(prefix)));
            }
        }
        for p in &self.packages {
            out.push_str(&format!(
                "\n[[package]]\npath = {}\nsources = {}\n",
                quote(&p.path),
                list(&p.sources)
            ));
            for s in &p.suites {
                out.push_str(&format!(
                    "\n[[package.suite]]\nrunner = {}\ntests = {}\n",
                    quote(s.runner.as_str()),
                    list(&s.tests)
                ));
                if let Some(c) = &s.command {
                    out.push_str(&format!("command = {}\n", quote(c)));
                }
                if let Some(r) = &s.report {
                    out.push_str(&format!("report = {}\n", quote(r)));
                }
                if !s.env.is_empty() {
                    let pairs: Vec<String> = s
                        .env
                        .iter()
                        .map(|(k, v)| format!("{} = {}", quote(k), quote(v)))
                        .collect();
                    out.push_str(&format!("env = {{ {} }}\n", pairs.join(", ")));
                }
                if let Some(c) = &s.setup {
                    out.push_str(&format!("setup = {}\n", quote(c)));
                }
                if !s.support.is_empty() {
                    out.push_str(&format!("support = {}\n", list(&s.support)));
                }
                if s.red_on_base != RedOnBase::Run {
                    out.push_str(&format!(
                        "red_on_base = {}\n",
                        quote(s.red_on_base.as_str())
                    ));
                }
                if let Some(r) = &s.reason {
                    out.push_str(&format!("reason = {}\n", quote(r)));
                }
            }
        }
        out
    }

    /// The package that owns `rel` (a forward-slash path relative to the repository root): the
    /// one whose path is the longest prefix of `rel`, compared per path component.
    pub fn package_for(&self, rel: &str) -> Option<&Package> {
        self.packages
            .iter()
            .filter(|p| owns(&p.path, rel))
            .max_by_key(|p| if p.path == "." { 0 } else { p.path.len() + 1 })
    }
}

fn quote(s: &str) -> String {
    toml::Value::String(s.to_owned()).to_string()
}

fn list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| quote(s)).collect();
    format!("[{}]", quoted.join(", "))
}

fn owns(pkg: &str, rel: &str) -> bool {
    pkg == "." || rel == pkg || rel.strip_prefix(pkg).is_some_and(|r| r.starts_with('/'))
}

/// A path must be relative, use forward slashes, and have no `.`/`..`/empty components.
/// `.` alone is allowed only where it means the repository root (`allow_root`).
fn check_path(p: &str, allow_root: bool) -> std::result::Result<(), String> {
    if p == "." && allow_root {
        return Ok(());
    }
    let bad = p.is_empty()
        || p.contains('\\')
        || p.starts_with('/')
        || p.split('/').any(|c| c.is_empty() || c == "." || c == "..");
    if bad {
        Err(format!(
            "`{p}` must be relative to the repository root, use forward slashes, \
             and have no `.`, `..` or empty parts"
        ))
    } else {
        Ok(())
    }
}
