//! CONF-035..037 from specjutsu/changes/migrate-ready: adopt declares each suite's base setup
//! from its lockfile.

use std::path::Path;

use sj_core::adopt::detect;
use sj_core::config::{Config, Runner, Suite};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const VITEST_PKG: &str = r#"{"name":"x","devDependencies":{"vitest":"^3.0.0"}}"#;
const PYTEST_PROJECT: &str =
    "[project]\nname = \"svc\"\nversion = \"0.1.0\"\n\n[dependency-groups]\ndev = [\"pytest>=8\"]\n";

fn suite<'a>(c: &'a Config, path: &str, runner: Runner) -> &'a Suite {
    c.packages
        .iter()
        .find(|p| p.path == path)
        .and_then(|p| p.suites.iter().find(|s| s.runner == runner))
        .unwrap_or_else(|| panic!("no {} suite in {path}: {:?}", runner.as_str(), c.packages))
}

#[test]
fn conf_035_a_pnpm_workspace_package_gets_the_workspace_install() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "package.json", r#"{"name":"root","private":true}"#);
    write(r, "pnpm-workspace.yaml", "packages:\n  - apps/*\n");
    write(r, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
    write(r, "apps/api/package.json", VITEST_PKG);
    let c = detect(r).unwrap().config;
    assert_eq!(
        suite(&c, "apps/api", Runner::Vitest).setup.as_deref(),
        Some("pnpm install --frozen-lockfile")
    );
}

#[test]
fn conf_036_each_lockfile_picks_its_install_command() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    for (pkg, lock) in [
        ("npm", "package-lock.json"),
        ("yarn", "yarn.lock"),
        ("bun", "bun.lock"),
    ] {
        write(r, &format!("{pkg}/package.json"), VITEST_PKG);
        write(r, &format!("{pkg}/{lock}"), "");
    }
    write(r, "svc/pyproject.toml", PYTEST_PROJECT);
    write(r, "svc/uv.lock", "version = 1\n");
    let c = detect(r).unwrap().config;
    let setup = |path: &str, runner| suite(&c, path, runner).setup.clone();
    assert_eq!(setup("npm", Runner::Vitest).as_deref(), Some("npm ci"));
    assert_eq!(
        setup("yarn", Runner::Vitest).as_deref(),
        Some("yarn install --frozen-lockfile")
    );
    assert_eq!(
        setup("bun", Runner::Vitest).as_deref(),
        Some("bun install --frozen-lockfile")
    );
    assert_eq!(setup("svc", Runner::Pytest).as_deref(), Some("uv sync --frozen"));
}

#[test]
fn conf_037_a_suite_without_a_lockfile_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "web/package.json", VITEST_PKG);
    let d = detect(r).unwrap();
    assert_eq!(suite(&d.config, "web", Runner::Vitest).setup, None);
    assert!(
        d.notes
            .iter()
            .any(|n| n == "no lockfile: declare setup for web:vitest"),
        "{:?}",
        d.notes
    );
}
