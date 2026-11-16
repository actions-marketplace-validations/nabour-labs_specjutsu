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
        out.push_str(&escape(lit));
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

/// Escape the characters that are special in both JavaScript and Rust regexes (not `-` or `/`,
/// which only matter inside classes or JS literals), so `-t` patterns stay readable.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\^$.*+?()[]{}|".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Whether a title holds `.each` placeholders.
fn has_placeholder(title: &str) -> bool {
    title_pattern(title).contains(".*")
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

/// A traced test's (or a loose case's) result in one suite run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum JoinedOutcome {
    Passed,
    Failed,
    Skipped,
    BuildFailed,
}

/// One result: a traced test (`test` is its index) or a case no traced test claimed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Joined {
    pub test: Option<usize>,
    /// `<file>::<name>` (repository-relative for traced tests, as reported otherwise).
    pub path: String,
    pub outcome: JoinedOutcome,
    pub message: Option<String>,
}

/// One title segment of a traced test: exact, or a pattern for `.each` placeholders.
enum Seg {
    Exact(String),
    Pattern(regex::Regex),
}

impl Seg {
    fn new(s: &str) -> Seg {
        if !has_placeholder(s) {
            return Seg::Exact(s.to_string());
        }
        regex::Regex::new(&format!("^{}$", title_pattern(s)))
            .map_or(Seg::Exact(s.to_string()), Seg::Pattern)
    }

    fn matches(&self, s: &str) -> bool {
        match self {
            Seg::Exact(e) => e == s,
            Seg::Pattern(r) => r.is_match(s),
        }
    }
}

struct Traced {
    /// Path components without the extension for pytest (`tests/test_inv` → [tests, test_inv]).
    file: Vec<String>,
    segs: Vec<Seg>,
}

/// Join a suite's JUnit cases to its traced tests. Several cases for one test (`.each`,
/// parametrize) aggregate: any failure fails it, otherwise any pass passes it. A file that
/// failed to load (a case named after the file) turns the traced tests in it into build
/// failures.
pub fn match_cases(
    runner: Runner,
    traced: &[TestFn],
    cases: &[sj_runners::junit::Case],
) -> Vec<Joined> {
    use sj_runners::junit::CaseOutcome;

    let pytest = runner == Runner::Pytest;
    let index: Vec<Traced> = traced
        .iter()
        .map(|t| {
            let f = t.file.to_string_lossy().replace('\\', "/");
            let f = if pytest {
                f.trim_end_matches(".py").to_string()
            } else {
                f
            };
            let sep = if pytest { "::" } else { " > " };
            Traced {
                file: f.split('/').map(String::from).collect(),
                segs: t.name.split(sep).map(Seg::new).collect(),
            }
        })
        .collect();
    // Basename → traced indices, so a case is only compared with tests in its own file.
    let mut by_base: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, t) in index.iter().enumerate() {
        if let Some(b) = t.file.last() {
            by_base.entry(b.as_str()).or_default().push(i);
        }
    }

    let mut results: BTreeMap<usize, (JoinedOutcome, Option<String>)> = BTreeMap::new();
    let mut loose: Vec<Joined> = Vec::new();
    let mut file_errors: Vec<(Vec<String>, String)> = Vec::new();

    for c in cases {
        let outcome = match c.outcome {
            CaseOutcome::Passed => JoinedOutcome::Passed,
            CaseOutcome::Failed => JoinedOutcome::Failed,
            CaseOutcome::Skipped => JoinedOutcome::Skipped,
        };
        if let Some(file) = file_level_error(runner, c) {
            file_errors.push((file, c.message.clone()));
            continue;
        }
        let (file, titles) = case_key(runner, c);
        let hit = candidates(&by_base, &file, pytest).into_iter().find(|&i| {
            let t = &index[i];
            if pytest {
                // [module…, classes…, func] as a component suffix either way.
                let mut want: Vec<&str> = t.file.iter().map(String::as_str).collect();
                want.extend(t.segs.iter().map(|s| match s {
                    Seg::Exact(e) => e.as_str(),
                    Seg::Pattern(_) => "",
                }));
                let got: Vec<&str> = file
                    .iter()
                    .map(String::as_str)
                    .chain(titles.iter().map(String::as_str))
                    .collect();
                got.len() >= 2 && (ends_with(&want, &got) || ends_with(&got, &want))
            } else {
                let fa: Vec<&str> = t.file.iter().map(String::as_str).collect();
                let fb: Vec<&str> = file.iter().map(String::as_str).collect();
                let same_file = file.is_empty() || ends_with(&fa, &fb) || ends_with(&fb, &fa);
                let full = t.segs.len() == titles.len()
                    && t.segs.iter().zip(&titles).all(|(s, x)| s.matches(x));
                // A generic report may only carry the leaf title.
                let leaf = runner == Runner::Junit
                    && titles.len() == 1
                    && t.segs.last().is_some_and(|s| s.matches(&titles[0]));
                same_file && (full || leaf)
            }
        });
        let message = (outcome == JoinedOutcome::Failed).then(|| c.message.clone());
        match hit {
            Some(i) => {
                let e = results.entry(i).or_insert((outcome, message.clone()));
                if rank(outcome) > rank(e.0) {
                    *e = (outcome, message);
                }
            }
            None => loose.push(Joined {
                test: None,
                path: format!("{}::{}", c.file.as_deref().unwrap_or(&c.classname), c.name),
                outcome,
                message,
            }),
        }
    }

    let mut out = Vec::new();
    let mut claimed = vec![false; file_errors.len()];
    for (i, t) in traced.iter().enumerate() {
        let path = format!(
            "{}::{}",
            t.file.to_string_lossy().replace('\\', "/"),
            t.name
        );
        if let Some((outcome, message)) = results.remove(&i) {
            out.push(Joined {
                test: Some(i),
                path,
                outcome,
                message,
            });
            continue;
        }
        let fa: Vec<&str> = index[i].file.iter().map(String::as_str).collect();
        if let Some(k) = file_errors.iter().position(|(f, _)| {
            let fb: Vec<&str> = f.iter().map(String::as_str).collect();
            ends_with(&fa, &fb) || ends_with(&fb, &fa)
        }) {
            claimed[k] = true;
            out.push(Joined {
                test: Some(i),
                path,
                outcome: JoinedOutcome::BuildFailed,
                message: Some(file_errors[k].1.clone()),
            });
        }
    }
    out.extend(loose);
    for ((file, message), used) in file_errors.into_iter().zip(claimed) {
        if !used {
            out.push(Joined {
                test: None,
                path: file.join("/"),
                outcome: JoinedOutcome::BuildFailed,
                message: Some(message),
            });
        }
    }
    out
}

/// Failed beats passed beats skipped when several cases fold into one test.
fn rank(o: JoinedOutcome) -> u8 {
    match o {
        JoinedOutcome::Skipped => 0,
        JoinedOutcome::Passed => 1,
        JoinedOutcome::Failed => 2,
        JoinedOutcome::BuildFailed => 3,
    }
}

/// A case that stands for a whole file failing to load: vitest names it after the file, and
/// pytest reports a collection error with an empty classname and the dotted module as name.
fn file_level_error(runner: Runner, c: &sj_runners::junit::Case) -> Option<Vec<String>> {
    use sj_runners::junit::CaseOutcome;
    if c.outcome != CaseOutcome::Failed {
        return None;
    }
    let file = c.file.as_deref().unwrap_or(&c.classname);
    match runner {
        Runner::Pytest if c.classname.is_empty() && !c.name.is_empty() => {
            Some(c.name.split('.').map(String::from).collect())
        }
        Runner::Vitest | Runner::Jest | Runner::Playwright
            if !file.is_empty() && c.name == file =>
        {
            Some(file.split('/').map(String::from).collect())
        }
        _ => None,
    }
}

/// A case's file components and title segments.
fn case_key(runner: Runner, c: &sj_runners::junit::Case) -> (Vec<String>, Vec<String>) {
    let split = |s: &str, sep: &str| -> Vec<String> {
        if s.is_empty() {
            Vec::new()
        } else {
            s.split(sep).map(String::from).collect()
        }
    };
    match runner {
        Runner::Pytest => {
            let name = c.name.split('[').next().unwrap_or(&c.name).to_string();
            (split(&c.classname, "."), vec![name])
        }
        Runner::Playwright => (split(&c.classname, "/"), split(&c.name, " › ")),
        Runner::Jest => {
            let file = c.file.as_deref().map(|f| split(f, "/")).unwrap_or_default();
            let mut titles = split(&c.classname, " > ");
            titles.push(c.name.clone());
            (file, titles)
        }
        Runner::Vitest => (split(&c.classname, "/"), split(&c.name, " > ")),
        Runner::Junit | Runner::Cargo => {
            let file = c.file.as_deref().map(|f| split(f, "/")).unwrap_or_default();
            (file, vec![c.name.clone()])
        }
    }
}

/// Traced tests worth comparing with a case: those whose file basename appears in the case's
/// file (or, for pytest, any of its dotted components). With no file, every test.
fn candidates(by_base: &BTreeMap<&str, Vec<usize>>, file: &[String], pytest: bool) -> Vec<usize> {
    let mut out: Vec<usize> = if pytest {
        file.iter()
            .filter_map(|c| by_base.get(c.as_str()))
            .flatten()
            .copied()
            .collect()
    } else if let Some(b) = file.last() {
        by_base.get(b.as_str()).cloned().unwrap_or_default()
    } else {
        by_base.values().flatten().copied().collect()
    };
    out.sort_unstable();
    out.dedup();
    out
}

fn ends_with(long: &[&str], short: &[&str]) -> bool {
    !short.is_empty() && long.len() >= short.len() && long[long.len() - short.len()..] == *short
}

/// What one non-Cargo suite run produced.
#[derive(Debug, Clone, Default)]
pub struct SuiteResult {
    pub joined: Vec<Joined>,
    /// No parsable report: the suite failed before running (RUN-013).
    pub no_report: bool,
    /// The last lines of the runner's output, when there is no report.
    pub build_errors: Vec<String>,
    /// The runner exited 0.
    pub exit_ok: bool,
}

/// Most output lines kept when a suite fails without a report.
pub const BUILD_ERROR_LINES: usize = 20;

/// Run one suite and join its report to `traced` (the suite's traced tests). With
/// `Pick::Tests`, results are post-filtered to the picked tests.
pub fn run_suite(
    root: &Path,
    pkg: &Package,
    suite: &Suite,
    traced: &[TestFn],
    pick: &Pick,
) -> Result<SuiteResult> {
    use anyhow::Context;

    let tmp = tempfile::tempdir()?;
    let report = match declared_report(root, pkg, suite) {
        Some(r) => {
            // Never read a stale report from an earlier run.
            let _ = std::fs::remove_file(&r);
            r
        }
        None => tmp.path().join("report.xml"),
    };
    let inv = invocation(root, pkg, suite, pick, &report)?;
    // Resolve through PATH (and PATHEXT on Windows, where `pnpm` is `pnpm.cmd`).
    let program =
        crate::doctor::on_path(&inv.program).unwrap_or_else(|| PathBuf::from(&inv.program));
    let mut command = std::process::Command::new(&program);
    shell_args(&mut command, suite.runner, &inv.args);
    let out = command
        .current_dir(&inv.cwd)
        .envs(&inv.env)
        .stdin(std::process::Stdio::null())
        .output()
        .with_context(|| {
            format!(
                "cannot run `{}` for {}:{} in {}",
                inv.program,
                pkg.path,
                suite.runner.as_str(),
                inv.cwd.display()
            )
        })?;

    let mut result = SuiteResult {
        exit_ok: out.status.success(),
        ..SuiteResult::default()
    };
    let cases = std::fs::read_to_string(&report)
        .ok()
        .and_then(|xml| sj_runners::junit::parse(&xml).ok());
    let Some(cases) = cases else {
        result.no_report = true;
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let lines: Vec<&str> = text
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.trim().is_empty())
            .collect();
        let from = lines.len().saturating_sub(BUILD_ERROR_LINES);
        result.build_errors = lines[from..].iter().map(|l| l.to_string()).collect();
        if result.build_errors.is_empty() {
            result.build_errors.push(format!(
                "{} exited with {} and wrote no report",
                inv.program, out.status
            ));
        }
        return Ok(result);
    };
    let mut joined = match_cases(suite.runner, traced, &cases);
    if let Pick::Tests(picked) = pick {
        let keep: Vec<bool> = traced
            .iter()
            .map(|t| picked.iter().any(|p| p.file == t.file && p.name == t.name))
            .collect();
        joined.retain(|j| j.test.is_some_and(|i| keep[i]));
    }
    result.joined = joined;
    Ok(result)
}

/// Adds the invocation's arguments. On Windows a `junit` command goes to
/// `cmd /S /C` verbatim: Rust's argument quoting would escape any `"` in it
/// in a way cmd.exe does not read.
#[cfg(windows)]
fn shell_args(command: &mut std::process::Command, runner: Runner, args: &[String]) {
    use std::os::windows::process::CommandExt;
    match (runner, args.split_last()) {
        (Runner::Junit, Some((cmd, flags))) => {
            command.arg("/S").args(flags).raw_arg(format!("\"{cmd}\""));
        }
        _ => {
            command.args(args);
        }
    }
}

#[cfg(not(windows))]
fn shell_args(command: &mut std::process::Command, _runner: Runner, args: &[String]) {
    command.args(args);
}
