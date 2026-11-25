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
         #### Scenario: AUTH-006 Bulk import\n- **WHEN** x\n- **THEN** y\n\n\
         #### Scenario: AUTH-007 Tight reason\n- **WHEN** x\n- **THEN** y\n\n\
         #### Scenario: AUTH-008 Conditional ignore\n- **WHEN** x\n- **THEN** y\n",
    );
    write(
        r,
        "tests/auth.rs",
        "#[test]\n#[ignore = \"slow integration test\"]\nfn auth_006_bulk_import() {}\n\n\
         #[test]\n#[ignore=\"slow\"]\nfn auth_007_tight_reason() {}\n\n\
         #[test]\n#[ignore_if(foo)]\nfn auth_008_conditional_ignore() {}\n",
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
    assert_eq!(mapped.len(), 3, "{v}");
    for (id, name, skipped) in [
        ("AUTH-006", "auth_006_bulk_import", true),
        ("AUTH-007", "auth_007_tight_reason", true),
        ("AUTH-008", "auth_008_conditional_ignore", false),
    ] {
        let m = mapped
            .iter()
            .find(|m| m["id"] == id)
            .unwrap_or_else(|| panic!("{id} not mapped: {v}"));
        let test = &m["tests"][0];
        assert_eq!(test["name"], name, "{v}");
        assert_eq!(test["skipped"], skipped, "{id}: {v}");
    }
}
