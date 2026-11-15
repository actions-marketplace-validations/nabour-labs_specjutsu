//! The project file, `specjutsu.toml`: spec root, mode and packages with their test suites.
//!
//! Parsing happens in two stages. Raw structs reject unknown keys with their position, then
//! validation turns them into the typed model and names the package in every error.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// The project file's name, at the repository root.
pub const FILE: &str = "specjutsu.toml";

/// The only schema version this build reads and writes.
pub const SCHEMA: u32 = 1;

/// The spec root used when the file is missing or does not set one.
pub const DEFAULT_SPEC_ROOT: &str = "openspec";

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
    Playwright,
    Pytest,
    Vitest,
}

impl Runner {
    /// Every known runner, in alphabetical order.
    pub const ALL: [Runner; 5] = [
        Runner::Cargo,
        Runner::Jest,
        Runner::Playwright,
        Runner::Pytest,
        Runner::Vitest,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Runner::Cargo => "cargo",
            Runner::Jest => "jest",
            Runner::Playwright => "playwright",
            Runner::Pytest => "pytest",
            Runner::Vitest => "vitest",
        }
    }

    fn known_list() -> String {
        Runner::ALL.map(Runner::as_str).join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suite {
    pub runner: Runner,
    /// Globs relative to the repository root.
    pub tests: Vec<String>,
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
    pub packages: Vec<Package>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema: Option<u32>,
    spec_root: Option<String>,
    mode: Option<String>,
    #[serde(default, rename = "package")]
    packages: Vec<RawPackage>,
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
}

/// Load `<root>/specjutsu.toml`, or the defaults for `root` when it does not exist.
pub fn load(root: &Path) -> Result<Config> {
    let path = root.join(FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => Config::parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default_for(root)),
        Err(e) => Err(e).with_context(|| format!("{FILE}: cannot read {}", path.display())),
    }
}

impl Config {
    /// The configuration of a repository that has no project file: spec root `openspec`, mode
    /// `guided`, and one root package with a `cargo` suite when a root `Cargo.toml` exists.
    pub fn default_for(root: &Path) -> Config {
        let packages = if root.join("Cargo.toml").is_file() {
            vec![Package {
                path: ".".into(),
                sources: vec!["**/*.rs".into()],
                suites: vec![Suite {
                    runner: Runner::Cargo,
                    tests: vec!["**/tests/**/*.rs".into()],
                }],
            }]
        } else {
            Vec::new()
        };
        Config {
            schema: SCHEMA,
            spec_root: DEFAULT_SPEC_ROOT.into(),
            mode: Mode::default(),
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
                suites.push(Suite {
                    runner,
                    tests: s.tests,
                });
            }
            packages.push(Package {
                path: p.path,
                sources: p.sources,
                suites,
            });
        }

        Ok(Config {
            schema,
            spec_root,
            mode,
            packages,
        })
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
