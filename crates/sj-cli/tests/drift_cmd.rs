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
