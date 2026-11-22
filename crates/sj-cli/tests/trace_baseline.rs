//! TRACE-017..020 from specjutsu/changes/brownfield-adoption: `sj trace --write-baseline`, and
//! `sj trace` reading the baseline.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const SPEC: &str = "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
                    #### Scenario: AUTH-005 Refresh token\n- **WHEN** x\n- **THEN** y\n\n\
                    #### Scenario: AUTH-002 Log in\n- **WHEN** x\n- **THEN** y\n";

const CONFIG: &str = r#"schema = 1
spec_root = "specjutsu"

[[package]]
path = "."
sources = ["src/**"]

[[package.suite]]
runner = "cargo"
tests = ["tests/**"]
"#;

/// A temp repo with `spec` as its only spec file and `tests` as tests/auth.rs.
fn fixture(spec: &str, tests: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(
        r,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(r, "src/lib.rs", "");
    write(r, "specjutsu/specs/auth/spec.md", spec);
    write(r, "tests/auth.rs", tests);
    dir
}

fn sj_trace(root: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("trace")
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

/// Output lines starting with `prefix` (e.g. `gap `).
fn lines_with<'a>(stdout: &'a str, prefix: &str) -> Vec<&'a str> {
    stdout.lines().filter(|l| l.starts_with(prefix)).collect()
}

#[test]
fn trace_018_only_new_gaps_get_a_line() {
    let spec = format!("{SPEC}\n#### Scenario: AUTH-006 Log out\n- **WHEN** x\n- **THEN** y\n");
    let dir = fixture(&spec, "");
    let r = dir.path();
    write(
        r,
        "specjutsu/trace-baseline.txt",
        "gap AUTH-002\ngap AUTH-005\n",
    );

    let out = sj_trace(r, &["--strict"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let gaps = lines_with(&stdout, "gap ");
    assert_eq!(gaps.len(), 1, "{stdout}");
    assert!(gaps[0].contains("AUTH-006"), "{stdout}");
    assert!(stdout.contains("3 gaps (2 known)"), "{stdout}");
    assert_eq!(out.status.code(), Some(1), "{stdout}");

    let out = sj_trace(r, &["--strict", "--json"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let known = |id: &str| {
        json["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["id"] == id)
            .unwrap_or_else(|| panic!("no gap {id}: {json}"))["known"]
            .clone()
    };
    assert_eq!(known("AUTH-006"), serde_json::json!(false));
    assert_eq!(known("AUTH-002"), serde_json::json!(true));
}

#[test]
fn trace_019_known_entries_alone_pass_strict() {
    let dir = fixture(
        SPEC,
        "#[test]\nfn auth_005_refresh() {}\n#[test]\nfn auth_009_old() {}\n",
    );
    let r = dir.path();
    write(
        r,
        "specjutsu/trace-baseline.txt",
        "gap AUTH-002\norphan AUTH-009 tests/auth.rs auth_009_old\n",
    );

    let out = sj_trace(r, &["--strict"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("1 gap"),
        "fixture must have the gap: {stdout}"
    );
    assert!(
        stdout.contains("1 orphan"),
        "fixture must have the orphan: {stdout}"
    );
    assert!(lines_with(&stdout, "gap ").is_empty(), "{stdout}");
    assert!(lines_with(&stdout, "orphan ").is_empty(), "{stdout}");
    assert_eq!(out.status.code(), Some(0), "{stdout}");
}

#[test]
fn trace_017_write_baseline_lists_every_gap_and_orphan() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(
        r,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(r, "src/lib.rs", "");
    write(r, "specjutsu/specs/auth/spec.md", SPEC);
    write(r, "tests/auth.rs", "#[test]\nfn auth_009_old() {}\n");

    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--write-baseline", "--root"])
        .arg(r)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let baseline = std::fs::read_to_string(r.join("specjutsu/trace-baseline.txt")).unwrap();
    assert_eq!(
        baseline,
        "gap AUTH-002\ngap AUTH-005\norphan AUTH-009 tests/auth.rs auth_009_old\n"
    );
}

#[test]
fn trace_020_a_gap_that_gained_a_test_is_stale() {
    let dir = fixture(
        SPEC,
        "#[test]\nfn auth_002_log_in() {}\n#[test]\nfn auth_005_refresh() {}\n",
    );
    let r = dir.path();
    write(r, "specjutsu/trace-baseline.txt", "gap AUTH-002\n");

    let out = sj_trace(r, &["--strict"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout
            .lines()
            .any(|l| l == "stale gap AUTH-002 (remove from trace-baseline.txt)"),
        "{stdout}"
    );
    assert_eq!(out.status.code(), Some(1), "{stdout}");

    let out = sj_trace(r, &["--strict", "--json"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["stale"], serde_json::json!(["gap AUTH-002"]), "{json}");
    assert_eq!(out.status.code(), Some(1), "{json}");

    let out = sj_trace(r, &["--write-baseline"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let baseline = std::fs::read_to_string(r.join("specjutsu/trace-baseline.txt")).unwrap();
    assert!(!baseline.lines().any(|l| l == "gap AUTH-002"), "{baseline}");
}

#[test]
fn an_orphan_that_lost_its_test_is_stale() {
    let dir = fixture(
        SPEC,
        "#[test]\nfn auth_002_log_in() {}\n#[test]\nfn auth_005_refresh() {}\n",
    );
    let r = dir.path();
    write(
        r,
        "specjutsu/trace-baseline.txt",
        "orphan AUTH-009 tests/auth.rs auth_009_old\n",
    );

    let out = sj_trace(r, &["--strict"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.lines().any(|l| l
            == "stale orphan AUTH-009 tests/auth.rs auth_009_old (remove from trace-baseline.txt)"),
        "{stdout}"
    );
    assert_eq!(out.status.code(), Some(1), "{stdout}");

    let out = sj_trace(r, &["--write-baseline"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let baseline = std::fs::read_to_string(r.join("specjutsu/trace-baseline.txt")).unwrap();
    assert_eq!(baseline, "");
}
