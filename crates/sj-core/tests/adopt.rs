//! CONF-008..010 from openspec/changes/project-config: detecting packages and runners.

mod support;

use std::path::Path;

use sj_core::adopt::detect;
use sj_core::config::{Config, Package, Runner};

const JS_TESTS: &str = "**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn pkg<'a>(c: &'a Config, path: &str) -> &'a Package {
    c.packages
        .iter()
        .find(|p| p.path == path)
        .unwrap_or_else(|| panic!("no package {path} in {:?}", c.packages))
}

fn runners(p: &Package) -> Vec<Runner> {
    p.suites.iter().map(|s| s.runner).collect()
}

#[test]
fn conf_008_adopt_a_pnpm_monorepo_with_a_python_service() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    let d = detect(dir.path()).unwrap();
    let c = &d.config;

    let paths: Vec<_> = c.packages.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(
        paths,
        ["apps/api", "apps/app", "packages/ui", "python/worker"]
    );

    let api = pkg(c, "apps/api");
    assert_eq!(runners(api), [Runner::Vitest]);
    assert_eq!(api.suites[0].tests, [format!("apps/api/{JS_TESTS}")]);
    assert_eq!(api.sources, ["apps/api/**"]);

    let app = pkg(c, "apps/app");
    assert_eq!(runners(app), [Runner::Vitest, Runner::Playwright]);
    assert_eq!(app.suites[1].tests, ["apps/app/e2e/**"]);

    let worker = pkg(c, "python/worker");
    assert_eq!(runners(worker), [Runner::Pytest]);
    assert_eq!(
        worker.suites[0].tests,
        ["python/worker/**/test_*.py", "python/worker/**/*_test.py"]
    );
}

#[test]
fn conf_008_playwright_test_dir_comes_from_its_config() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "package.json",
        r#"{ "name": "ext", "devDependencies": { "@playwright/test": "^1", "jest": "^29" } }"#,
    );
    write(
        dir.path(),
        "playwright.config.ts",
        "export default defineConfig({\n  testDir: './tests/e2e',\n})\n",
    );
    let c = detect(dir.path()).unwrap().config;
    assert_eq!(c.packages.len(), 1);
    let root = &c.packages[0];
    assert_eq!(root.path, ".");
    assert_eq!(runners(root), [Runner::Jest, Runner::Playwright]);
    assert_eq!(root.suites[0].tests, [JS_TESTS]);
    assert_eq!(root.suites[1].tests, ["tests/e2e/**"]);
    assert_eq!(root.sources, ["**"]);
}

#[test]
fn conf_008_npm_and_yarn_workspaces() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "package.json",
        r#"{ "name": "root", "workspaces": { "packages": ["pkgs/*", "!pkgs/skip"] } }"#,
    );
    write(dir.path(), "pkgs/a/package.json", r#"{ "name": "a" }"#);
    write(dir.path(), "pkgs/a/vitest.config.ts", "export default {}\n");
    write(
        dir.path(),
        "pkgs/skip/package.json",
        r#"{ "name": "skip" }"#,
    );
    write(
        dir.path(),
        "pkgs/a/nested/package.json",
        r#"{ "name": "deep" }"#,
    );
    let c = detect(dir.path()).unwrap().config;
    let paths: Vec<_> = c.packages.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, ["pkgs/a"]);
    assert_eq!(runners(&c.packages[0]), [Runner::Vitest]);

    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "package.json",
        r#"{ "name": "root", "workspaces": ["libs/*"] }"#,
    );
    write(
        dir.path(),
        "libs/x/package.json",
        r#"{ "name": "x", "devDependencies": { "jest": "^29" } }"#,
    );
    let c = detect(dir.path()).unwrap().config;
    assert_eq!(c.packages[0].path, "libs/x");
    assert_eq!(runners(&c.packages[0]), [Runner::Jest]);
}

#[test]
fn conf_008_cargo_and_pytest_config_files() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\"]\n",
    );
    write(
        dir.path(),
        "svc/pyproject.toml",
        "[project]\nname = \"svc\"\n",
    );
    write(dir.path(), "svc/tests/conftest.py", "");
    write(
        dir.path(),
        "tool/pyproject.toml",
        "[project]\nname = \"tool\"\n\n[tool.pytest.ini_options]\naddopts = \"-q\"\n",
    );
    write(
        dir.path(),
        "cov/pyproject.toml",
        "[project]\nname = \"cov\"\n[project.optional-dependencies]\ndev = [\"pytest-cov\"]\n",
    );
    let d = detect(dir.path()).unwrap();
    let c = &d.config;
    let root = pkg(c, ".");
    assert_eq!(runners(root), [Runner::Cargo]);
    assert_eq!(root.suites[0].tests, ["**/*.rs"]);
    assert_eq!(runners(pkg(c, "svc")), [Runner::Pytest]);
    assert_eq!(runners(pkg(c, "tool")), [Runner::Pytest]);
    // pytest-cov alone is a plugin, not pytest.
    assert!(pkg(c, "cov").suites.is_empty());
    assert!(d.notes.contains(&"no runner detected: cov".to_string()));
}

#[test]
fn conf_009_adopt_keeps_the_openspec_folder() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    assert_eq!(detect(dir.path()).unwrap().config.spec_root, "openspec");
}

#[test]
fn conf_033_adopt_without_an_openspec_folder_uses_specjutsu() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "Cargo.toml", "[workspace]\n");
    assert_eq!(detect(dir.path()).unwrap().config.spec_root, "specjutsu");
}

#[test]
fn conf_010_a_package_with_no_detectable_runner_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    let d = detect(dir.path()).unwrap();
    assert!(pkg(&d.config, "packages/ui").suites.is_empty());
    assert_eq!(d.notes, ["no runner detected: packages/ui"]);
}

#[test]
fn conf_010_detected_config_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    let c = detect(dir.path()).unwrap().config;
    assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);
}

#[test]
fn conf_008_separate_packages_without_a_workspace() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "backend/pyproject.toml",
        "[project]\nname = \"b\"\ndependencies = [\"pytest\"]\n",
    );
    write(
        dir.path(),
        "pwa/package.json",
        r#"{ "name": "pwa", "devDependencies": { "vitest": "^3" } }"#,
    );
    write(
        dir.path(),
        "pwa/tools/package.json",
        r#"{ "name": "inner" }"#,
    );
    write(dir.path(), "infra/package.json", r#"{ "name": "infra" }"#);
    let d = detect(dir.path()).unwrap();
    let paths: Vec<_> = d.config.packages.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, ["backend", "infra", "pwa"]);
    assert_eq!(runners(pkg(&d.config, "pwa")), [Runner::Vitest]);
    assert_eq!(d.notes, ["no runner detected: infra"]);
}
