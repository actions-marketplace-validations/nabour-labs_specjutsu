//! Dependency checks for `sj audit --registry` (AUDIT-009, AUDIT-010): direct dependencies a
//! change adds to `Cargo.toml`, `package.json` or `pyproject.toml`, looked up on crates.io,
//! npm and PyPI. Off by default; a registry that can't be reached only warns.

use std::collections::BTreeSet;
use std::time::Duration;

use chrono::NaiveDate;

use crate::audit::{Finding, Severity};

/// A package that is younger than this many days is reported as `new-package`.
pub const NEW_PACKAGE_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ecosystem {
    Crates,
    Npm,
    Pypi,
}

impl Ecosystem {
    pub fn name(self) -> &'static str {
        match self {
            Ecosystem::Crates => "crates",
            Ecosystem::Npm => "npm",
            Ecosystem::Pypi => "pypi",
        }
    }

    /// The manifest file this ecosystem's dependencies are read from.
    pub fn for_manifest(rel: &str) -> Option<Ecosystem> {
        match rel.rsplit('/').next()? {
            "Cargo.toml" => Some(Ecosystem::Crates),
            "package.json" => Some(Ecosystem::Npm),
            "pyproject.toml" => Some(Ecosystem::Pypi),
            _ => None,
        }
    }

    fn base_url(self) -> String {
        let (var, default) = match self {
            Ecosystem::Crates => ("SJ_REGISTRY_CRATES", "https://crates.io"),
            Ecosystem::Npm => ("SJ_REGISTRY_NPM", "https://registry.npmjs.org"),
            Ecosystem::Pypi => ("SJ_REGISTRY_PYPI", "https://pypi.org"),
        };
        std::env::var(var)
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| default.to_string())
            .trim_end_matches('/')
            .to_string()
    }

    fn url(self, name: &str) -> String {
        let base = self.base_url();
        match self {
            Ecosystem::Crates => format!("{base}/api/v1/crates/{name}"),
            Ecosystem::Npm => format!("{base}/{}", name.replace('/', "%2F")),
            Ecosystem::Pypi => format!("{base}/pypi/{name}/json"),
        }
    }
}

/// Registry dependency names declared in one manifest's text. Path, git and workspace
/// dependencies are local, so they are left out.
pub fn dependencies(eco: Ecosystem, text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    match eco {
        Ecosystem::Crates => {
            let Ok(doc) = text.parse::<toml::Table>() else {
                return out;
            };
            let mut tables: Vec<&toml::Table> = Vec::new();
            for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                tables.extend(doc.get(key).and_then(|v| v.as_table()));
            }
            if let Some(ws) = doc.get("workspace").and_then(|v| v.as_table()) {
                tables.extend(ws.get("dependencies").and_then(|v| v.as_table()));
            }
            for t in tables {
                for (name, spec) in t {
                    let local = spec.as_table().is_some_and(|s| {
                        s.contains_key("path")
                            || s.contains_key("git")
                            || s.contains_key("workspace")
                    });
                    if !local {
                        let real = spec
                            .as_table()
                            .and_then(|s| s.get("package"))
                            .and_then(|p| p.as_str())
                            .unwrap_or(name);
                        out.insert(real.to_string());
                    }
                }
            }
        }
        Ecosystem::Npm => {
            let Ok(doc) = serde_json::from_str::<serde_json::Value>(text) else {
                return out;
            };
            for key in [
                "dependencies",
                "devDependencies",
                "peerDependencies",
                "optionalDependencies",
            ] {
                if let Some(deps) = doc.get(key).and_then(|v| v.as_object()) {
                    for (name, ver) in deps {
                        let v = ver.as_str().unwrap_or("");
                        let local = ["file:", "link:", "workspace:", "git", "http", "npm:"]
                            .iter()
                            .any(|p| v.starts_with(p));
                        if !local {
                            out.insert(name.clone());
                        }
                    }
                }
            }
        }
        Ecosystem::Pypi => {
            let Ok(doc) = text.parse::<toml::Table>() else {
                return out;
            };
            let mut reqs: Vec<String> = Vec::new();
            if let Some(project) = doc.get("project").and_then(|v| v.as_table()) {
                let strings = |v: Option<&toml::Value>| -> Vec<String> {
                    v.and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default()
                };
                reqs.extend(strings(project.get("dependencies")));
                if let Some(opt) = project
                    .get("optional-dependencies")
                    .and_then(|v| v.as_table())
                {
                    for v in opt.values() {
                        reqs.extend(strings(Some(v)));
                    }
                }
            }
            if let Some(groups) = doc.get("dependency-groups").and_then(|v| v.as_table()) {
                for v in groups.values() {
                    if let Some(a) = v.as_array() {
                        reqs.extend(a.iter().filter_map(|x| x.as_str().map(str::to_string)));
                    }
                }
            }
            for r in reqs {
                let name: String = r
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
                    .collect();
                if !name.is_empty() && !r.contains(" @ ") {
                    out.insert(name.to_lowercase().replace(['_', '.'], "-"));
                }
            }
        }
    }
    out
}

/// What a registry said about one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    Missing,
    /// First published on this date (or unknown).
    Found(Option<NaiveDate>),
    Unreachable,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .user_agent("specjutsu (+https://github.com/nabour-labs/specjutsu)")
        .build()
}

/// The earliest `YYYY-MM-DD` among `dates`, ignoring ones that don't parse.
fn earliest<'a>(dates: impl Iterator<Item = &'a str>) -> Option<NaiveDate> {
    dates
        .filter_map(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok())
        .min()
}

pub fn lookup(agent: &ureq::Agent, eco: Ecosystem, name: &str) -> Lookup {
    let resp = match agent.get(&eco.url(name)).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(404, _)) => return Lookup::Missing,
        Err(_) => return Lookup::Unreachable,
    };
    let Ok(v) = resp.into_json::<serde_json::Value>() else {
        return Lookup::Found(None);
    };
    let first = match eco {
        Ecosystem::Crates => earliest(
            v.pointer("/crate/created_at")
                .and_then(|d| d.as_str())
                .into_iter(),
        ),
        Ecosystem::Npm => earliest(
            v.pointer("/time/created")
                .and_then(|d| d.as_str())
                .into_iter(),
        ),
        Ecosystem::Pypi => {
            let files = v
                .get("releases")
                .and_then(|r| r.as_object())
                .into_iter()
                .flat_map(|r| r.values())
                .filter_map(|files| files.as_array())
                .flatten();
            earliest(files.filter_map(|f| f.get("upload_time_iso_8601").and_then(|d| d.as_str())))
        }
    };
    Lookup::Found(first)
}

/// Today, or `SJ_TODAY` (`YYYY-MM-DD`) when set, so tests are deterministic.
fn today() -> NaiveDate {
    std::env::var("SJ_TODAY")
        .ok()
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive())
}

/// Look up every added dependency and turn the answers into findings.
pub fn check(added: &BTreeSet<(Ecosystem, String)>) -> Vec<Finding> {
    let agent = agent();
    let now = today();
    let mut out = Vec::new();
    let mut unreachable: BTreeSet<Ecosystem> = BTreeSet::new();
    for (eco, name) in added {
        if unreachable.contains(eco) {
            continue;
        }
        let target = format!("{} {name}", eco.name());
        match lookup(&agent, *eco, name) {
            Lookup::Missing => out.push(Finding::new(
                "unknown-package",
                target,
                String::new(),
                Severity::Fail,
            )),
            Lookup::Found(Some(first)) => {
                let days = (now - first).num_days();
                if days < NEW_PACKAGE_DAYS {
                    let ago = if days == 1 {
                        "1 day ago".to_string()
                    } else {
                        format!("{days} days ago")
                    };
                    out.push(Finding::new(
                        "new-package",
                        target,
                        format!("(first published {ago})"),
                        Severity::Warn,
                    ));
                }
            }
            Lookup::Found(None) => {}
            Lookup::Unreachable => {
                unreachable.insert(*eco);
                out.push(Finding::new(
                    "registry unreachable:",
                    eco.name().to_string(),
                    String::new(),
                    Severity::Warn,
                ));
            }
        }
    }
    out
}
