//! `sj doctor`: check the project file, packages, runner executables and spec folders.
//!
//! Probes never install, sync or build anything.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::{self, Config, Package, Runner};

/// One line per problem; empty when the setup is sound.
pub fn check(root: &Path) -> Vec<String> {
    let config = match config::load(root) {
        Ok(c) => c,
        Err(e) => return vec![one_line(&format!("{e:#}"))],
    };
    let mut out = Vec::new();
    for p in &config.packages {
        if p.suites.is_empty() {
            out.push(format!("no runner: {}", p.path));
        }
    }
    let mut seen = Vec::new();
    for p in &config.packages {
        for s in &p.suites {
            if seen.contains(&(s.runner, &p.path)) {
                continue;
            }
            seen.push((s.runner, &p.path));
            if !runner_available(root, p, s.runner) {
                out.push(format!("missing runner {} ({})", s.runner.as_str(), p.path));
            }
        }
    }
    out.extend(missing_spec_dirs(root, &config));
    out
}

/// Drop the source excerpt (`  |` gutter lines) from a TOML error and join what is left.
fn one_line(msg: &str) -> String {
    msg.lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty() && !l.trim_start().starts_with('|') && !l.contains(" | "))
        .collect::<Vec<_>>()
        .join(": ")
}

fn missing_spec_dirs(root: &Path, config: &Config) -> Vec<String> {
    ["specs", "changes"]
        .iter()
        .map(|d| format!("{}/{d}", config.spec_root))
        .filter(|rel| !root.join(rel).is_dir())
        .map(|rel| format!("missing spec root: {rel}"))
        .collect()
}

fn runner_available(root: &Path, pkg: &Package, runner: Runner) -> bool {
    let dir = if pkg.path == "." {
        root.to_path_buf()
    } else {
        root.join(&pkg.path)
    };
    match runner {
        Runner::Cargo => on_path("cargo").is_some(),
        Runner::Vitest | Runner::Jest | Runner::Playwright => {
            let bin = match runner {
                Runner::Vitest => "vitest",
                Runner::Jest => "jest",
                _ => "playwright",
            };
            node_bin(root, &dir, bin)
        }
        Runner::Pytest => {
            on_path("pytest").is_some()
                || on_path("uv").is_some_and(|uv| {
                    Command::new(uv)
                        .args(["run", "--no-sync", "pytest", "--version"])
                        .current_dir(&dir)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .is_ok_and(|s| s.success())
                })
        }
    }
}

/// `node_modules/.bin/<name>` in `dir` or any parent up to and including `root`.
fn node_bin(root: &Path, dir: &Path, name: &str) -> bool {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if executable_in(&d.join("node_modules").join(".bin"), name).is_some() {
            return true;
        }
        if d == root {
            break;
        }
        cur = d.parent();
    }
    false
}

/// The first `name` on `PATH`, honouring `PATHEXT` on Windows.
pub fn on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|d| executable_in(&d, name))
}

fn executable_in(dir: &Path, name: &str) -> Option<PathBuf> {
    if cfg!(windows) {
        let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
        std::iter::once(String::new())
            .chain(exts.split(';').filter(|e| !e.is_empty()).map(str::to_owned))
            .map(|ext| dir.join(format!("{name}{ext}")))
            .find(|p| p.is_file())
    } else {
        let p = dir.join(name);
        is_executable(&p).then_some(p)
    }
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}
