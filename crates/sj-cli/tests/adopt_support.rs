//! CONF-039..041 from specjutsu/changes/gate-pilot-fixes: `sj adopt` declares helper folders
//! as `support` globs.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const VITEST_PKG: &str = r#"{"name":"x","devDependencies":{"vitest":"^3.0.0"}}"#;

/// The `support` globs of the `runner` suite in package `path` of `sj adopt --dry-run`.
fn dry_run_support(root: &Path, path: &str, runner: &str) -> Vec<String> {
    let suite = dry_run_suite(root, path, runner);
    suite
        .get("support")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("no support key: {suite:?}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// The `[[package.suite]]` table for `runner` in package `path` of `sj adopt --dry-run`.
fn dry_run_suite(root: &Path, path: &str, runner: &str) -> toml::Table {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["adopt", "--dry-run", "--root"])
        .arg(root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let doc: toml::Table = toml::from_str(&text).unwrap();
    doc["package"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"].as_str() == Some(path))
        .and_then(|p| {
            p["suite"]
                .as_array()?
                .iter()
                .find(|s| s["runner"].as_str() == Some(runner))
        })
        .and_then(|s| s.as_table().cloned())
        .unwrap_or_else(|| panic!("no {runner} suite in {path}:\n{text}"))
}

#[test]
fn conf_039_helper_folders_become_support_globs() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "apps/api/package.json", VITEST_PKG);
    write(
        r,
        "apps/api/src/testing/admin.ts",
        "export const admin = 1;\n",
    );
    write(r, "apps/api/src/__fixtures__/user.json", "{}\n");
    assert_eq!(
        dry_run_support(r, "apps/api", "vitest"),
        ["apps/api/src/__fixtures__/**", "apps/api/src/testing/**"]
    );
}

#[test]
fn conf_039_helper_folders_under_tests_are_left_out() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "apps/api/package.json", VITEST_PKG);
    write(
        r,
        "apps/api/src/testing/admin.ts",
        "export const admin = 1;\n",
    );
    // The gate always overlays files under a `tests/` folder.
    write(r, "apps/api/tests/fixtures/user.json", "{}\n");
    assert_eq!(
        dry_run_support(r, "apps/api", "vitest"),
        ["apps/api/src/testing/**"]
    );
}

#[test]
fn conf_041_adopt_names_each_support_glob_it_adds() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "apps/api/package.json", VITEST_PKG);
    write(
        r,
        "apps/api/src/testing/admin.ts",
        "export const admin = 1;\n",
    );
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["adopt", "--dry-run", "--root"])
        .arg(r)
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let lines: Vec<&str> = stderr.lines().filter(|l| l.contains("support: ")).collect();
    assert_eq!(
        lines,
        ["note    support: apps/api/src/testing/** (not source; overlaid on base; remove it if this folder holds product code)"],
        "{stderr}"
    );
}

#[test]
fn conf_040_a_package_without_helper_folders_gets_no_support_key() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "web/package.json", VITEST_PKG);
    write(r, "web/src/app.ts", "export const app = 1;\n");
    let suite = dry_run_suite(r, "web", "vitest");
    assert!(!suite.contains_key("support"), "{suite:?}");
}
