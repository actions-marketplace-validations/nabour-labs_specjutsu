//! `sj adopt`: detect an existing project's packages and test runners from manifests and
//! config files, without running any tool.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde_json::Value as Json;

use crate::config::{default_spec_root, Config, Package, Runner, Suite, SCHEMA};

/// Default vitest/jest test files.
const JS_TESTS: &str = "**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}";

/// Suite order inside a package: unit runners first, end-to-end last.
const SUITE_ORDER: [Runner; 5] = [
    Runner::Cargo,
    Runner::Vitest,
    Runner::Jest,
    Runner::Pytest,
    Runner::Playwright,
];

/// Directories never searched for packages.
const SKIP_DIRS: [&str; 5] = ["node_modules", ".venv", "venv", "target", ".git"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detected {
    pub config: Config,
    /// One line per thing the user should look at, e.g. `no runner detected: packages/ui`.
    pub notes: Vec<String>,
}

/// Detect packages and runners under `root`.
pub fn detect(root: &Path) -> Result<Detected> {
    let manifests = scan(root)?;
    // path → suites found for it so far (several sources can contribute to one directory).
    let mut found: BTreeMap<String, Vec<Suite>> = BTreeMap::new();

    for dir in js_packages(root, &manifests.package_json)? {
        let suites = js_suites(root, &dir)?;
        found.entry(dir).or_default().extend(suites);
    }
    for dir in &manifests.pyproject {
        let text = read(root, &join(dir, "pyproject.toml"))?;
        let doc: toml::Table = toml::from_str(&text)
            .with_context(|| format!("cannot parse {}", join(dir, "pyproject.toml")))?;
        // A root pyproject without `[project]` is a uv workspace root or tool settings only.
        if dir == "." && !doc.contains_key("project") {
            continue;
        }
        let entry = found.entry(dir.clone()).or_default();
        if has_pytest(root, dir, &doc) {
            entry.push(Suite::new(
                Runner::Pytest,
                vec![prefix(dir, "**/test_*.py"), prefix(dir, "**/*_test.py")],
            ));
        }
    }
    if root.join("Cargo.toml").is_file() {
        found
            .entry(".".into())
            .or_default()
            .push(Suite::new(Runner::Cargo, vec!["**/*.rs".into()]));
    }

    let mut notes = Vec::new();
    let packages = found
        .into_iter()
        .map(|(path, mut suites)| {
            suites.sort_by_key(|s| SUITE_ORDER.iter().position(|r| *r == s.runner));
            suites.dedup_by_key(|s| s.runner);
            if suites.is_empty() {
                notes.push(format!("no runner detected: {path}"));
            }
            let cargo_only = !suites.is_empty() && suites.iter().all(|s| s.runner == Runner::Cargo);
            let sources = match (path.as_str(), cargo_only) {
                (".", true) => vec!["**/*.rs".to_string()],
                (".", false) => vec!["**".to_string()],
                _ => vec![prefix(&path, "**")],
            };
            Package {
                path,
                sources,
                suites,
            }
        })
        .collect();

    Ok(Detected {
        config: Config {
            schema: SCHEMA,
            spec_root: default_spec_root(root),
            mode: Default::default(),
            non_source: Vec::new(),
            id_prefixes: Default::default(),
            packages,
            assert_helpers: Vec::new(),
            drift_fail: false,
            drift_shared_permille: crate::config::DRIFT_SHARED_PERMILLE,
            language: None,
        },
        notes,
    })
}

/// Directories (relative, forward slashes, `.` for the root) holding each kind of manifest.
struct Manifests {
    package_json: Vec<String>,
    pyproject: Vec<String>,
}

fn scan(root: &Path) -> Result<Manifests> {
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
    let mut m = Manifests {
        package_json: Vec::new(),
        pyproject: Vec::new(),
    };
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let list = match entry.file_name().to_str() {
            Some("package.json") => &mut m.package_json,
            Some("pyproject.toml") => &mut m.pyproject,
            _ => continue,
        };
        let parent = entry.path().parent().unwrap_or(root);
        list.push(rel(root, parent));
    }
    Ok(m)
}

/// JS packages: workspace members when the root declares a workspace, else the outermost
/// `package.json` directories.
fn js_packages(root: &Path, dirs: &[String]) -> Result<Vec<String>> {
    let patterns = match workspace_patterns(root)? {
        Some(p) => p,
        // No workspace: every package.json that is not inside another package.
        None => {
            return Ok(dirs
                .iter()
                .filter(|d| !dirs.iter().any(|outer| *outer != **d && owns(outer, d)))
                .cloned()
                .collect());
        }
    };
    let (include, exclude) = globsets(&patterns)?;
    Ok(dirs
        .iter()
        .filter(|d| *d != "." && include.is_match(d.as_str()) && !exclude.is_match(d.as_str()))
        .cloned()
        .collect())
}

/// Workspace member globs from `pnpm-workspace.yaml` or the root `package.json` `workspaces`.
fn workspace_patterns(root: &Path) -> Result<Option<Vec<String>>> {
    let pnpm = root.join("pnpm-workspace.yaml");
    if pnpm.is_file() {
        return Ok(Some(pnpm_packages(&std::fs::read_to_string(pnpm)?)));
    }
    let pj = root.join("package.json");
    if !pj.is_file() {
        return Ok(None);
    }
    let v: Json = serde_json::from_str(&std::fs::read_to_string(&pj)?)
        .context("cannot parse package.json")?;
    let ws = match v.get("workspaces") {
        Some(Json::Array(a)) => a,
        Some(Json::Object(o)) => match o.get("packages") {
            Some(Json::Array(a)) => a,
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    Ok(Some(
        ws.iter()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect(),
    ))
}

/// The `packages:` list of a pnpm workspace file, read line by line (block or flow style).
fn pnpm_packages(text: &str) -> Vec<String> {
    let clean = |s: &str| {
        s.split(" #")
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string()
    };
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("packages:") {
            let rest = rest.trim();
            if let Some(flow) = rest.strip_prefix('[') {
                let flow = flow.split(']').next().unwrap_or("");
                out.extend(flow.split(',').map(clean).filter(|s| !s.is_empty()));
                return out;
            }
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let t = line.trim_start();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if line.starts_with(|c: char| !c.is_whitespace()) && !t.starts_with('-') {
            break;
        }
        if let Some(item) = t.strip_prefix('-') {
            let item = clean(item);
            if !item.is_empty() {
                out.push(item);
            }
        }
    }
    out
}

fn globsets(patterns: &[String]) -> Result<(GlobSet, GlobSet)> {
    let mut inc = GlobSetBuilder::new();
    let mut exc = GlobSetBuilder::new();
    for p in patterns {
        let (set, pat) = match p.strip_prefix('!') {
            Some(neg) => (&mut exc, neg),
            None => (&mut inc, p.as_str()),
        };
        let pat = pat.trim_start_matches("./").trim_end_matches('/');
        set.add(GlobBuilder::new(pat).literal_separator(true).build()?);
    }
    Ok((inc.build()?, exc.build()?))
}

fn js_suites(root: &Path, dir: &str) -> Result<Vec<Suite>> {
    let text = read(root, &join(dir, "package.json"))?;
    let v: Json = serde_json::from_str(&text)
        .with_context(|| format!("cannot parse {}", join(dir, "package.json")))?;
    let has_dep = |name: &str| {
        ["dependencies", "devDependencies"]
            .iter()
            .any(|k| v.get(k).and_then(|d| d.get(name)).is_some())
    };
    let has_config = |stem: &str| config_file(root, dir, stem).is_some();

    let mut suites = Vec::new();
    for (runner, dep, stem) in [
        (Runner::Vitest, "vitest", "vitest.config"),
        (Runner::Jest, "jest", "jest.config"),
    ] {
        if has_dep(dep) || has_config(stem) {
            suites.push(Suite::new(runner, vec![prefix(dir, JS_TESTS)]));
        }
    }
    if has_dep("@playwright/test") || has_config("playwright.config") {
        let test_dir = config_file(root, dir, "playwright.config")
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|t| playwright_test_dir(&t))
            .unwrap_or_else(|| "e2e".into());
        suites.push(Suite::new(
            Runner::Playwright,
            vec![prefix(dir, &format!("{test_dir}/**"))],
        ));
    }
    Ok(suites)
}

pub(crate) fn config_file(root: &Path, dir: &str, stem: &str) -> Option<std::path::PathBuf> {
    ["ts", "mts", "cts", "js", "mjs", "cjs"]
        .iter()
        .map(|ext| root.join(join(dir, &format!("{stem}.{ext}"))))
        .find(|p| p.is_file())
}

/// `testDir` when it is a plain string literal, without a leading `./` or trailing `/`.
pub(crate) fn playwright_test_dir(config: &str) -> Option<String> {
    let re = regex::Regex::new(r#"testDir\s*:\s*['"`]([^'"`$]+)['"`]"#).ok()?;
    let d = re.captures(config)?.get(1)?.as_str();
    let d = d.trim_start_matches("./").trim_end_matches('/');
    (!d.is_empty() && !d.starts_with('/') && !d.split('/').any(|c| c == ".."))
        .then(|| d.to_string())
}

/// pytest as a dependency anywhere a Python project lists them, a `[tool.pytest.ini_options]`
/// table, or a `pytest.ini` / `conftest.py` in the package or its `tests/`.
fn has_pytest(root: &Path, dir: &str, doc: &toml::Table) -> bool {
    let mut deps: Vec<String> = Vec::new();
    let mut strings = |v: Option<&toml::Value>| {
        if let Some(a) = v.and_then(|v| v.as_array()) {
            deps.extend(a.iter().filter_map(|x| x.as_str().map(str::to_owned)));
        }
    };
    let project = doc.get("project");
    strings(project.and_then(|p| p.get("dependencies")));
    for table in [
        project.and_then(|p| p.get("optional-dependencies")),
        doc.get("dependency-groups"),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(t) = table.as_table() {
            for v in t.values() {
                strings(Some(v));
            }
        }
    }
    let tool = doc.get("tool");
    strings(
        tool.and_then(|t| t.get("uv"))
            .and_then(|u| u.get("dev-dependencies")),
    );
    let poetry_dep = tool.and_then(|t| t.get("poetry")).is_some_and(|p| {
        let in_table = |t: Option<&toml::Value>| t.and_then(|t| t.get("pytest")).is_some();
        in_table(p.get("dev-dependencies"))
            || p.get("group")
                .and_then(|g| g.as_table())
                .is_some_and(|g| g.values().any(|grp| in_table(grp.get("dependencies"))))
    });
    let ini = tool
        .and_then(|t| t.get("pytest"))
        .and_then(|p| p.get("ini_options"))
        .is_some();
    let files = ["pytest.ini", "conftest.py", "tests/conftest.py"]
        .iter()
        .any(|f| root.join(join(dir, f)).is_file());
    deps.iter().any(|d| is_pytest_req(d)) || poetry_dep || ini || files
}

/// A requirement string naming `pytest` itself, e.g. `pytest>=8` or `pytest[testing]; python_version>"3"`.
fn is_pytest_req(req: &str) -> bool {
    let name: String = req
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect();
    name.eq_ignore_ascii_case("pytest")
}

fn owns(outer: &str, inner: &str) -> bool {
    outer == "."
        || inner
            .strip_prefix(outer)
            .is_some_and(|r| r.starts_with('/'))
}

fn prefix(dir: &str, glob: &str) -> String {
    if dir == "." {
        glob.to_string()
    } else {
        format!("{dir}/{glob}")
    }
}

fn join(dir: &str, file: &str) -> String {
    prefix(dir, file)
}

fn read(root: &Path, rel: &str) -> Result<String> {
    std::fs::read_to_string(root.join(rel)).with_context(|| format!("cannot read {rel}"))
}

fn rel(root: &Path, p: &Path) -> String {
    let r = p
        .strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/");
    if r.is_empty() {
        ".".into()
    } else {
        r
    }
}
