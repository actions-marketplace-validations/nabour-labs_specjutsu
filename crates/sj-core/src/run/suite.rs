//! Non-Cargo suites: the command each runner gets, and joining its JUnit report to traced tests.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::config::{Package, Runner, Suite};
use crate::testscan::TestFn;

/// What to run in a suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    /// The whole suite.
    All,
    /// Only these traced tests (paths relative to the repository root).
    Tests(Vec<TestFn>),
}

/// A process to start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
}

/// The package directory on disk.
pub fn package_dir(root: &Path, pkg: &Package) -> PathBuf {
    if pkg.path == "." {
        root.to_path_buf()
    } else {
        root.join(&pkg.path)
    }
}

/// The report a `junit` suite's command writes, if it declares one.
pub fn declared_report(root: &Path, pkg: &Package, suite: &Suite) -> Option<PathBuf> {
    suite
        .report
        .as_ref()
        .map(|r| package_dir(root, pkg).join(r))
}

/// The command for `suite` in `pkg`, writing its JUnit report to `report` (ignored by `junit`,
/// which writes its declared report).
pub fn invocation(
    root: &Path,
    pkg: &Package,
    suite: &Suite,
    pick: &Pick,
    report: &Path,
) -> Result<Invocation> {
    let cwd = package_dir(root, pkg);
    let report = report.to_string_lossy().into_owned();
    let tests: &[TestFn] = match pick {
        Pick::All => &[],
        Pick::Tests(t) => t,
    };
    let mut env = BTreeMap::new();

    let (launcher, args): (Vec<String>, Vec<String>) = match suite.runner {
        Runner::Cargo => bail!("cargo suites run through cargo_test_detailed"),
        Runner::Junit => {
            let Some(cmd) = &suite.command else {
                bail!("package {}: junit suite needs `command`", pkg.path);
            };
            let shell = if cfg!(windows) {
                vec!["cmd".to_string(), "/C".into()]
            } else {
                vec!["sh".to_string(), "-c".into()]
            };
            (shell, vec![cmd.clone()])
        }
        Runner::Vitest | Runner::Jest | Runner::Playwright => {
            let bin = match suite.runner {
                Runner::Vitest => "vitest",
                Runner::Jest => "jest",
                _ => "playwright",
            };
            let mut args: Vec<String> = Vec::new();
            match suite.runner {
                Runner::Vitest => {
                    args.extend(
                        ["run", "--reporter=default", "--reporter=junit"].map(String::from),
                    );
                    args.push(format!("--outputFile.junit={report}"));
                    args.extend(js_files_and_titles(&cwd, root, tests));
                }
                Runner::Jest => {
                    args.extend(
                        ["--ci", "--reporters=default", "--reporters=jest-junit"].map(String::from),
                    );
                    args.extend(js_files_and_titles(&cwd, root, tests));
                    env.insert("JEST_JUNIT_OUTPUT_FILE".into(), report.clone());
                    env.insert("JEST_JUNIT_ADD_FILE_ATTRIBUTE".into(), "true".into());
                    env.insert("JEST_JUNIT_CLASSNAME".into(), "{classname}".into());
                    env.insert("JEST_JUNIT_TITLE".into(), "{title}".into());
                    env.insert("JEST_JUNIT_ANCESTOR_SEPARATOR".into(), " > ".into());
                }
                _ => {
                    args.extend(["test", "--reporter=junit"].map(String::from));
                    args.extend(
                        tests
                            .iter()
                            .map(|t| format!("{}:{}", rel_to(&cwd, root, &t.file), t.line)),
                    );
                    env.insert("PLAYWRIGHT_JUNIT_OUTPUT_NAME".into(), report.clone());
                }
            }
            let launcher = match &suite.command {
                Some(c) => split(c),
                None => node_launcher(root, &cwd, bin),
            };
            (launcher, args)
        }
        Runner::Pytest => {
            let mut args = vec![
                format!("--junitxml={report}"),
                "-o".into(),
                "junit_family=xunit1".into(),
            ];
            args.extend(
                tests
                    .iter()
                    .map(|t| format!("{}::{}", rel_to(&cwd, root, &t.file), t.name)),
            );
            let launcher = match &suite.command {
                Some(c) => split(c),
                None if has_up(root, &cwd, "uv.lock") => {
                    vec!["uv".into(), "run".into(), "pytest".into()]
                }
                None => {
                    let py = if cfg!(windows) { "python" } else { "python3" };
                    vec![py.into(), "-m".into(), "pytest".into()]
                }
            };
            (launcher, args)
        }
    };

    for (k, v) in &suite.env {
        env.insert(k.clone(), v.clone());
    }
    let mut it = launcher.into_iter();
    let Some(program) = it.next() else {
        bail!("package {}: empty `command`", pkg.path);
    };
    let mut all: Vec<String> = it.collect();
    all.extend(args);
    Ok(Invocation {
        program,
        args: all,
        cwd,
        env,
    })
}

/// `pnpm exec`, `yarn` or `npx --no-install`, from the nearest lockfile between `dir` and `root`.
fn node_launcher(root: &Path, dir: &Path, bin: &str) -> Vec<String> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if d.join("pnpm-lock.yaml").is_file() {
            return vec!["pnpm".into(), "exec".into(), bin.into()];
        }
        if d.join("yarn.lock").is_file() {
            return vec!["yarn".into(), bin.into()];
        }
        if d.join("package-lock.json").is_file() || d == root {
            break;
        }
        cur = d.parent();
    }
    vec!["npx".into(), "--no-install".into(), bin.into()]
}

/// Whether `file` exists in `dir` or a parent up to and including `root`.
fn has_up(root: &Path, dir: &Path, file: &str) -> bool {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if d.join(file).is_file() {
            return true;
        }
        if d == root {
            return false;
        }
        cur = d.parent();
    }
    false
}

/// Test files (relative to the package) and a `-t` pattern of their leaf titles.
fn js_files_and_titles(cwd: &Path, root: &Path, tests: &[TestFn]) -> Vec<String> {
    if tests.is_empty() {
        return Vec::new();
    }
    let mut files: Vec<String> = tests.iter().map(|t| rel_to(cwd, root, &t.file)).collect();
    files.sort();
    files.dedup();
    let mut titles: Vec<String> = Vec::new();
    for t in tests {
        let leaf = t.name.rsplit(" > ").next().unwrap_or(&t.name);
        let pat = title_pattern(leaf);
        if !titles.contains(&pat) {
            titles.push(pat);
        }
    }
    files.push("-t".into());
    files.push(titles.join("|"));
    files
}

/// A regex for a test title, with `.each` placeholders (`%s`, `$name`) as wildcards.
pub fn title_pattern(title: &str) -> String {
    let mut out = String::new();
    let mut lit = String::new();
    let chars: Vec<char> = title.chars().collect();
    let mut i = 0;
    let flush = |lit: &mut String, out: &mut String| {
        out.push_str(&regex::escape(lit));
        lit.clear();
    };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '%' && next.is_some_and(|n| "sdifjoO#".contains(n)) {
            flush(&mut lit, &mut out);
            out.push_str(".*");
            i += 2;
        } else if c == '$' && next.is_some_and(|n| n.is_ascii_alphanumeric() || n == '_') {
            flush(&mut lit, &mut out);
            out.push_str(".*");
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '_' | '.'))
            {
                i += 1;
            }
        } else {
            lit.push(c);
            i += 1;
        }
    }
    flush(&mut lit, &mut out);
    out
}

/// `file` (relative to the repository root) relative to `cwd`, with forward slashes.
fn rel_to(cwd: &Path, root: &Path, file: &Path) -> String {
    let abs = root.join(file);
    abs.strip_prefix(cwd)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn split(cmd: &str) -> Vec<String> {
    cmd.split_whitespace().map(String::from).collect()
}
