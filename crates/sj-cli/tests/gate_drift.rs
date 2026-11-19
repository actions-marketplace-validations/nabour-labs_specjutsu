//! DRIFT-010 and DRIFT-011 from openspec/changes/spec-drift: drift in `sj gate`.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_TEST: &str = "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n";
const AUTH_SPEC: &str = "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n#### Scenario: AUTH-003 Reject an expired token\n- **WHEN** the token is old\n- **THEN** the response is 401\n";
const AUTH_TEST: &str = "#[test]\nfn auth_003_rejects_expired() { assert_eq!(1 + 1, 2); }\n";

fn auth_rs(edit_line_15: bool) -> String {
    (1..=40)
        .map(|n| {
            if edit_line_15 && n == 15 {
                "// expiry is now 48 hours\n".to_string()
            } else {
                format!("// auth line {n}\n")
            }
        })
        .collect()
}

/// `main`: a buggy `add`, AUTH-003 with its test, `src/auth.rs`, and `config` as
/// `specjutsu.toml` when given. `feature` then has a coverage map built on `main` in which
/// AUTH-003 owns `src/auth.rs` 10–30.
fn repo(config: Option<&str>) -> Repo {
    let repo = Repo::with_lib(ADD_BUG);
    repo.write(".gitignore", "target/\nCargo.lock\n.specjutsu/\n");
    if let Some(c) = config {
        repo.write("specjutsu.toml", c);
    }
    repo.write("openspec/specs/auth/spec.md", AUTH_SPEC);
    repo.write("tests/auth.rs", AUTH_TEST);
    repo.write("src/auth.rs", &auth_rs(false));
    repo.commit("auth");
    let commit = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.branch("feature");
    repo.write(
        ".specjutsu/coverage-map.json",
        &serde_json::json!({
            "commit": commit,
            "tree": "0000",
            "shared_lines": 0,
            "scenarios": { "AUTH-003": { "src/auth.rs": [[10, 30]] } },
        })
        .to_string(),
    );
    repo
}

/// A change that proves MATH-001 and also edits AUTH-003's code.
fn prove_math_and_touch_auth(repo: &Repo) {
    repo.add_change("math", &[("MATH-001", "Add")]);
    repo.write("src/lib.rs", ADD_OK);
    repo.write("tests/add.rs", ADD_TEST);
    repo.write("src/auth.rs", &auth_rs(true));
    repo.commit("feat(MATH-001): green");
}

fn gate(repo: &Repo) -> (Option<i32>, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .arg("gate")
        .arg("--root")
        .arg(repo.path())
        .args(["--base", "main"])
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

const FAIL_CONFIG: &str = "schema = 1\n\n[drift]\nfail = true\n\n[[package]]\npath = \".\"\nsources = [\"**/*.rs\"]\n\n[[package.suite]]\nrunner = \"cargo\"\ntests = [\"**/*.rs\"]\n";

#[test]
fn drift_010_drift_fails_the_gate_only_when_configured() {
    let warn = repo(None);
    prove_math_and_touch_auth(&warn);
    let (code, out) = gate(&warn);
    assert_eq!(code, Some(0), "{out}");
    assert!(out.starts_with("gate: pass — 1 scenario proven"), "{out}");
    assert!(
        out.contains("warn    code-drift AUTH-003 src/auth.rs:15"),
        "{out}"
    );

    let fail = repo(Some(FAIL_CONFIG));
    prove_math_and_touch_auth(&fail);
    let (code, out) = gate(&fail);
    assert_eq!(code, Some(1), "{out}");
    assert!(out.starts_with("gate: fail — 1 problem\n"), "{out}");
    assert!(
        out.contains("fail    code-drift AUTH-003 src/auth.rs:15\n"),
        "{out}"
    );
}

#[test]
fn drift_011_a_refactor_exemption_turns_code_drift_into_information() {
    let repo = repo(Some(FAIL_CONFIG));
    repo.write(
        "openspec/changes/pricing/proposal.md",
        "## Why\n\nTidy.\n\nExempt: refactor — extract pricing module\n",
    );
    repo.write("src/auth.rs", &auth_rs(true));
    repo.commit("refactor: extract pricing module");

    let (code, out) = gate(&repo);
    assert!(
        out.contains("info    code-drift AUTH-003 src/auth.rs:15"),
        "{out}"
    );
    assert!(!out.contains("fail    code-drift"), "{out}");
    assert_eq!(code, Some(0), "{out}");
}
