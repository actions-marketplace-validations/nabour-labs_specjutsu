//! TRACE-015 and TRACE-016 from openspec/changes/polyglot-trace: `sj trace` on a polyglot repo.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = r#"schema = 1

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = []

[[package]]
path = "python/worker"
sources = ["python/worker/**"]

[[package.suite]]
runner = "pytest"
tests = []

[[package]]
path = "packages/ui"
sources = ["packages/ui/src/**"]
"#;

const SPEC: &str = "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
                    #### Scenario: AUTH-003 Reject expired token\n- **WHEN** x\n- **THEN** y\n\n\
                    #### Scenario: INV-002 Remove item\n- **WHEN** x\n- **THEN** y\n";

fn polyglot() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "openspec/specs/auth/spec.md", SPEC);
    write(
        r,
        "apps/api/src/auth.test.ts",
        "it(\"AUTH-003 rejects expired token\", () => {})\n",
    );
    write(
        r,
        "python/worker/tests/test_inventory.py",
        "def test_inv_002_removes_item():\n    pass\n",
    );
    write(
        r,
        "packages/ui/src/button.tsx",
        "export const Button = () => null\n",
    );
    dir
}

#[test]
fn trace_015_package_and_suite_appear_in_json() {
    let dir = polyglot();
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--json", "--root"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mapped = v["mapped"].as_array().unwrap();
    assert_eq!(mapped.len(), 2, "{v}");
    let auth = &mapped[0]["tests"][0];
    assert_eq!(auth["package"], "apps/api");
    assert_eq!(auth["suite"], "vitest");
    assert_eq!(auth["file"], "apps/api/src/auth.test.ts");
    assert_eq!(auth["skipped"], false);
    let inv = &mapped[1]["tests"][0];
    assert_eq!(inv["package"], "python/worker");
    assert_eq!(inv["suite"], "pytest");
    assert_eq!(v["warnings"][0], "no test suite: packages/ui");
}

#[test]
fn trace_016_a_package_without_a_suite_warns() {
    let dir = polyglot();
    Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--root"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("trace: 2 mapped, 0 gaps"))
        .stdout(predicate::str::contains(
            "\nwarn    no test suite: packages/ui\n",
        ));
}
