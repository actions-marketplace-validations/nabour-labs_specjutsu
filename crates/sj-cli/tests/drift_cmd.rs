//! DRIFT scenarios from openspec/changes/spec-drift: the coverage map and `sj drift`.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

/// The auth spec with AUTH-002's THEN status, and AUTH-003.
fn auth_spec(status: u16) -> String {
    format!(
        "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
         #### Scenario: AUTH-002 Reject a bad token\n- **WHEN** the token is bad\n- **THEN** the response is {status}\n\n\
         #### Scenario: AUTH-003 Reject an expired token\n- **WHEN** the token is old\n- **THEN** the response is 401\n"
    )
}

const AUTH_TESTS: &str = "#[test]\nfn auth_002_rejects_bad() { assert_eq!(demo::add(1, 1), 2); }\n\n#[test]\nfn auth_003_rejects_expired() { assert_eq!(demo::add(2, 2), 4); }\n";

/// A repository on `feature` with the auth spec and tests on `main`.
fn auth_repo() -> Repo {
    let repo = Repo::with_lib("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
    repo.write("openspec/specs/auth/spec.md", &auth_spec(401));
    repo.write("tests/auth.rs", AUTH_TESTS);
    repo.commit("auth");
    repo.branch("feature");
    repo
}

fn drift(repo: &Repo, extra: &[&str]) -> (Option<i32>, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .arg("drift")
        .arg("--root")
        .arg(repo.path())
        .args(["--base", "main"])
        .args(extra)
        .output()
        .unwrap();
    (
        out.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn drift_001_a_coverage_command_without_lcov_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("specjutsu.toml"),
        "schema = 1\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n[[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\ncoverage = \"cargo llvm-cov --lcov\"\n",
    )
    .unwrap();
    Command::cargo_bin("sj")
        .unwrap()
        .args(["doctor", "--root"])
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "suite cargo (.): coverage must contain {lcov}",
        ));
}

#[test]
fn drift_007_editing_a_scenario_s_then_without_touching_its_test() {
    let repo = auth_repo();
    repo.write("openspec/specs/auth/spec.md", &auth_spec(403));
    repo.commit("docs: AUTH-002 returns 403");

    let (code, out) = drift(&repo, &[]);
    assert!(
        out.contains("stale-test AUTH-002 — spec changed; its tests did not"),
        "{out}"
    );
    assert!(!out.contains("AUTH-003"), "{out}");
    assert_eq!(code, Some(0), "warnings only: {out}");
}

#[test]
fn drift_008_no_map_means_only_stale_test_runs() {
    let repo = auth_repo();
    repo.write("openspec/specs/auth/spec.md", &auth_spec(403));
    repo.commit("docs: AUTH-002 returns 403");

    let (_, out) = drift(&repo, &[]);
    assert!(out.contains("stale-test AUTH-002"), "{out}");
    assert!(
        out.contains("no coverage map: code drift not checked (run sj coverage map)"),
        "{out}"
    );
}

/// `src/auth.rs` with 40 numbered lines.
fn auth_rs(edit: Option<(usize, &str)>) -> String {
    (1..=40)
        .map(|n| match edit {
            Some((l, text)) if l == n => format!("{text}\n"),
            _ => format!("// auth line {n}\n"),
        })
        .collect()
}

/// `auth_repo` plus `src/auth.rs`, ignoring `.specjutsu/`, with a coverage map built on `main`
/// in which AUTH-003 owns `src/auth.rs` lines 10–30.
fn mapped_repo() -> Repo {
    let repo = Repo::with_lib("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
    repo.write(".gitignore", "target/\nCargo.lock\n.specjutsu/\n");
    repo.write("openspec/specs/auth/spec.md", &auth_spec(401));
    repo.write("tests/auth.rs", AUTH_TESTS);
    repo.write("src/auth.rs", &auth_rs(None));
    repo.commit("auth");
    let commit = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.branch("feature");
    write_map(&repo, &commit, &[("AUTH-003", "src/auth.rs", 10, 30)]);
    repo
}

fn write_map(repo: &Repo, commit: &str, owned: &[(&str, &str, usize, usize)]) {
    let mut scenarios = serde_json::Map::new();
    for (id, file, a, b) in owned {
        scenarios.insert(id.to_string(), serde_json::json!({ *file: [[a, b]] }));
    }
    let map = serde_json::json!({
        "commit": commit,
        "tree": "0000",
        "shared_lines": 0,
        "scenarios": scenarios,
    });
    repo.write(".specjutsu/coverage-map.json", &map.to_string());
}

#[test]
fn drift_005_changing_a_scenario_s_code_without_its_spec_or_tests_is_drift() {
    let repo = mapped_repo();
    repo.write(
        "src/auth.rs",
        &auth_rs(Some((15, "// expiry is now 48 hours"))),
    );
    repo.commit("fix: longer expiry");

    let (code, out) = drift(&repo, &[]);
    assert!(
        out.contains(
            "code-drift AUTH-003 src/auth.rs:15 — code its tests run changed; spec and tests did not"
        ),
        "{out}"
    );
    assert!(!out.contains("AUTH-002"), "{out}");
    assert_eq!(code, Some(0), "warnings only: {out}");
}

#[test]
fn drift_006_updating_the_scenario_s_test_clears_the_drift() {
    let repo = mapped_repo();
    repo.write(
        "src/auth.rs",
        &auth_rs(Some((15, "// expiry is now 48 hours"))),
    );
    repo.commit("fix: longer expiry");
    let (_, before) = drift(&repo, &[]);
    assert!(before.contains("code-drift AUTH-003"), "{before}");

    repo.write(
        "tests/auth.rs",
        &AUTH_TESTS.replace("demo::add(2, 2), 4", "demo::add(2, 3), 5"),
    );
    repo.commit("test: expiry");
    let (_, after) = drift(&repo, &[]);
    assert!(!after.contains("AUTH-003"), "{after}");
    assert_eq!(after.lines().next(), Some("drift: none"), "{after}");
}

#[test]
fn drift_009_lines_shift_with_insertions_before_them() {
    // The map is built on main's first commit; main then inserts 5 lines at line 3, so
    // AUTH-003's lines 10–30 are 15–35 on base.
    let repo = Repo::with_lib("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
    repo.write(".gitignore", "target/\nCargo.lock\n.specjutsu/\n");
    repo.write("openspec/specs/auth/spec.md", &auth_spec(401));
    repo.write("tests/auth.rs", AUTH_TESTS);
    repo.write("src/auth.rs", &auth_rs(None));
    repo.commit("auth");
    let mapped = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    let mut lines: Vec<String> = auth_rs(None).lines().map(str::to_string).collect();
    for k in 0..5 {
        lines.insert(2, format!("// inserted {k}"));
    }
    let base_text = lines.join("\n") + "\n";
    repo.write("src/auth.rs", &base_text);
    repo.commit("insert a header");
    repo.branch("feature");
    write_map(&repo, &mapped, &[("AUTH-003", "src/auth.rs", 10, 30)]);

    // Base line 33 is map line 28 (owned); without translation it would be outside 10–30.
    let mut edited: Vec<String> = base_text.lines().map(str::to_string).collect();
    edited[32] = "// changed".to_string();
    repo.write("src/auth.rs", &(edited.join("\n") + "\n"));
    repo.commit("fix");

    let (_, out) = drift(&repo, &[]);
    assert!(out.contains("code-drift AUTH-003 src/auth.rs:33"), "{out}");
    assert!(
        out.contains(&format!(
            "coverage map from {}, 1 commit behind base",
            &mapped[..7]
        )),
        "{out}"
    );
}

#[test]
fn drift_012_json_lists_each_finding() {
    let repo = mapped_repo();
    let commit = repo.git(&["rev-parse", "main"]).trim().to_string();
    repo.write(
        "src/auth.rs",
        &auth_rs(Some((15, "// expiry is now 48 hours"))),
    );
    repo.commit("fix: longer expiry");

    let (code, out) = drift(&repo, &["--json"]);
    assert_eq!(code, Some(0), "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let findings = v["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{v}");
    let f = &findings[0];
    assert_eq!(f["kind"], "code-drift");
    assert_eq!(f["id"], "AUTH-003");
    assert_eq!(f["file"], "src/auth.rs");
    assert_eq!(f["line"], 15);
    assert_eq!(f["commit"], commit, "each finding names the map's commit: {v}");
}
