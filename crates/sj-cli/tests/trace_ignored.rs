//! TRACE-021 from specjutsu/changes/specs-from-tests: an ignored Rust test is marked skipped.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = r#"schema = 1
spec_root = "specjutsu"

[[package]]
path = "."
sources = ["src/**"]

[[package.suite]]
runner = "cargo"
tests = ["tests/**"]
"#;

#[test]
fn trace_021_an_ignored_rust_test_is_marked_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(
        r,
        "specjutsu/specs/auth/spec.md",
        "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
         #### Scenario: AUTH-006 Bulk import\n- **WHEN** x\n- **THEN** y\n",
    );
    write(
        r,
        "tests/auth.rs",
        "#[test]\n#[ignore = \"slow\"]\nfn auth_006_bulk_import() {}\n",
    );

    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--json", "--root"])
        .arg(r)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mapped = v["mapped"].as_array().unwrap();
    assert_eq!(mapped.len(), 1, "{v}");
    assert_eq!(mapped[0]["id"], "AUTH-006");
    let test = &mapped[0]["tests"][0];
    assert_eq!(test["name"], "auth_006_bulk_import");
    assert_eq!(test["skipped"], true, "{v}");
}
