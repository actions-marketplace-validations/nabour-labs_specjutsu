//! `sj status`, `sj task done`, `sj next` and `sj verify` CLI scenarios (specjutsu/changes/agent-loop).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(root);
    c
}

const TASKS: &str = "# Tasks

## 1. Sign in

- [x] 1.1 Login form (AUTH-007)
- [x] 1.2 Logout (AUTH-008)
- [x] 1.3 Login docs (no-scenario: docs)

## 2. Tokens

- [ ] 2.1 Refresh tokens (AUTH-005, AUTH-006)
- [ ] 2.2 Revoke tokens (no-scenario: later)
- [ ] 2.3 Rotate keys (no-scenario: later)
- [ ] 2.4 Audit log (no-scenario: later)
";

fn delta() -> String {
    let mut s = String::from(
        "## ADDED Requirements\n\n### Requirement: Tokens\nThe system SHALL manage tokens.\n\n",
    );
    for (id, name) in [
        ("AUTH-005", "Refresh an expiring token"),
        ("AUTH-006", "Reject a reused refresh token"),
        ("AUTH-007", "Show the login form"),
        ("AUTH-008", "Log out"),
    ] {
        s.push_str(&format!(
            "#### Scenario: {id} {name}\n- **WHEN** the client asks for {name}\n- **THEN** {id} holds\n\n"
        ));
    }
    s
}

/// `add-oauth-login`: 7 tasks (3 checked) and AUTH-005..008 added.
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(
        root,
        "openspec/specs/auth/spec.md",
        "# auth Specification\n\n## Purpose\nTokens.\n\n## Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n#### Scenario: AUTH-001 Log in\n- **WHEN** a\n- **THEN** b\n",
    );
    put(
        root,
        "openspec/changes/add-oauth-login/proposal.md",
        "## Why\n\nOAuth.\n",
    );
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &delta(),
    );
    put(root, "openspec/changes/add-oauth-login/tasks.md", TASKS);
    dir
}

/// Write ledger lines: (tree, ID, result).
fn ledger(root: &Path, records: &[(&str, &str, &str)]) {
    let mut s = String::new();
    for (i, (tree, id, result)) in records.iter().enumerate() {
        s.push_str(&format!(
            "{{\"v\":1,\"run\":\"r{i}\",\"ts\":{i},\"tree\":\"{tree}\",\"test\":\"t\",\"ids\":[\"{id}\"],\"result\":\"{result}\"}}\n"
        ));
    }
    put(root, ".specjutsu/ledger/runs.jsonl", &s);
}

fn mode(root: &Path, mode: &str) {
    put(
        root,
        "specjutsu.toml",
        &format!("schema = 1\nmode = \"{mode}\"\n"),
    );
}

fn stdout(c: &mut Command) -> String {
    String::from_utf8(c.output().unwrap().stdout).unwrap()
}

#[test]
fn loop_001_status_fits_in_ten_lines() {
    let dir = project();
    ledger(
        dir.path(),
        &[
            ("t1", "AUTH-007", "failed"),
            ("t2", "AUTH-007", "passed"),
            ("t1", "AUTH-008", "build-failed"),
            ("t2", "AUTH-008", "passed"),
            ("t2", "AUTH-005", "failed"),
        ],
    );
    let out = sj(dir.path(), &["status", "add-oauth-login"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(out).unwrap();
    assert!(out.lines().count() <= 10, "{out}");
    assert!(out.contains("add-oauth-login"), "{out}");
    assert!(out.contains("3/7 tasks"), "{out}");
    assert!(out.contains("2 proven, 1 red, 1 untested"), "{out}");
    assert!(
        out.contains("next 2.1 Refresh tokens (AUTH-005, AUTH-006)"),
        "{out}"
    );
    assert!(out.contains("red AUTH-005"), "{out}");
    assert!(out.contains("untested AUTH-006"), "{out}");
}

#[test]
fn loop_001_status_without_a_change_lists_every_open_change() {
    let dir = project();
    put(
        dir.path(),
        "openspec/changes/zz-other/tasks.md",
        "- [x] 1.1 Done (no-scenario: docs)\n",
    );
    let out = stdout(&mut sj(dir.path(), &["status"]));
    assert!(out.contains("add-oauth-login  3/7 tasks"), "{out}");
    assert!(out.contains("zz-other  1/1 tasks"), "{out}");
    assert!(out.contains("all tasks done"), "{out}");
    assert!(out.find("add-oauth-login") < out.find("zz-other"), "{out}");
}

#[test]
fn loop_001_status_json_has_the_same_data() {
    let dir = project();
    ledger(dir.path(), &[("t2", "AUTH-005", "passed")]);
    let out = stdout(&mut sj(dir.path(), &["status", "--json"]));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let c = &v[0];
    assert_eq!(c["name"], "add-oauth-login");
    assert_eq!(c["tasks_done"], 3);
    assert_eq!(c["tasks_total"], 7);
    assert_eq!(c["scenarios"]["green_unproven"][0], "AUTH-005");
    assert_eq!(c["scenarios"]["untested"].as_array().unwrap().len(), 3);
    assert_eq!(c["next"], "2.1 Refresh tokens (AUTH-005, AUTH-006)");
}

#[test]
fn loop_004_strict_mode_refuses_an_unproven_task() {
    let dir = project();
    mode(dir.path(), "strict");
    ledger(
        dir.path(),
        &[
            ("t1", "AUTH-005", "passed"),
            ("t2", "AUTH-005", "passed"),
            ("t1", "AUTH-006", "failed"),
            ("t2", "AUTH-006", "passed"),
        ],
    );
    let file = dir.path().join("openspec/changes/add-oauth-login/tasks.md");
    sj(dir.path(), &["task", "done", "add-oauth-login", "2.1"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unproven AUTH-005 (no red run recorded)",
        ))
        .stdout(predicate::str::contains("AUTH-006").not());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), TASKS);
}

#[test]
fn loop_004_strict_mode_completes_a_proven_task() {
    let dir = project();
    mode(dir.path(), "strict");
    ledger(
        dir.path(),
        &[
            ("t1", "AUTH-005", "failed"),
            ("t2", "AUTH-005", "passed"),
            ("t1", "AUTH-006", "failed"),
            ("t2", "AUTH-006", "passed"),
        ],
    );
    let file = dir.path().join("openspec/changes/add-oauth-login/tasks.md");
    sj(dir.path(), &["task", "done", "add-oauth-login", "2.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("task: done 2.1 Refresh tokens"));
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        TASKS.replace("- [ ] 2.1", "- [x] 2.1")
    );
}

#[test]
fn loop_004_strict_mode_names_why_each_id_is_unproven() {
    let dir = project();
    mode(dir.path(), "strict");
    ledger(dir.path(), &[("t1", "AUTH-005", "failed")]);
    sj(dir.path(), &["task", "done", "add-oauth-login", "2.1"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unproven AUTH-005 (latest failed)",
        ))
        .stdout(predicate::str::contains(
            "unproven AUTH-006 (no runs recorded)",
        ));
}

#[test]
fn loop_005_guided_mode_warns_but_completes() {
    let dir = project();
    mode(dir.path(), "guided");
    ledger(
        dir.path(),
        &[("t1", "AUTH-005", "passed"), ("t2", "AUTH-005", "passed")],
    );
    // CRLF line endings stay as they are; only the one box changes.
    let crlf = TASKS.replace('\n', "\r\n");
    let file = dir.path().join("openspec/changes/add-oauth-login/tasks.md");
    std::fs::write(&file, &crlf).unwrap();
    sj(dir.path(), &["task", "done", "add-oauth-login", "2.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("task: done 2.1 Refresh tokens"))
        .stdout(predicate::str::contains(
            "warn    unproven AUTH-005 (no red run recorded)",
        ));
    assert_eq!(
        std::fs::read(&file).unwrap(),
        crlf.replace("- [ ] 2.1", "- [x] 2.1").into_bytes()
    );
}

#[test]
fn loop_005_done_tasks_and_unknown_tasks() {
    let dir = project();
    let file = dir.path().join("openspec/changes/add-oauth-login/tasks.md");
    sj(dir.path(), &["task", "done", "add-oauth-login", "1.1"])
        .assert()
        .success()
        .stdout("task: already done 1.1 Login form (AUTH-007)\n");
    // A task without IDs needs no proof; `2.` is not task 2.1.
    sj(dir.path(), &["task", "done", "add-oauth-login", "2."])
        .assert()
        .code(2);
    sj(dir.path(), &["task", "done", "add-oauth-login", "2.2"])
        .assert()
        .success()
        .stdout("task: done 2.2 Revoke tokens (no-scenario: later)\n");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        TASKS.replace("- [ ] 2.2", "- [x] 2.2")
    );
}

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

/// The scenario text `sj next` prints for one of `delta()`'s scenarios.
fn scenario_text(id: &str, name: &str) -> String {
    format!("#### Scenario: {id} {name}\n- **WHEN** the client asks for {name}\n- **THEN** {id} holds\n")
}

#[test]
fn loop_002_next_shows_one_task_and_its_scenarios() {
    let dir = project();
    put(dir.path(), "openspec/principles.md", "# Principles\n");
    let out = stdout(&mut sj(dir.path(), &["next"]));
    assert!(out.contains("change add-oauth-login\n"), "{out}");
    assert!(
        out.contains("task 2.1 Refresh tokens (AUTH-005, AUTH-006)\n"),
        "{out}"
    );
    assert!(
        out.contains(&scenario_text("AUTH-005", "Refresh an expiring token")),
        "{out}"
    );
    assert!(
        out.contains(&scenario_text("AUTH-006", "Reject a reused refresh token")),
        "{out}"
    );
    assert!(out.contains("  sj test AUTH-005\n"), "{out}");
    assert!(out.contains("  sj test AUTH-006\n"), "{out}");
    assert!(out.contains("principles openspec/principles.md\n"), "{out}");
    // No other task, and no other scenario.
    for other in ["2.2", "1.1", "AUTH-007", "AUTH-008"] {
        assert!(!out.contains(other), "{other}: {out}");
    }
    // Not a git repository: no files section.
    assert!(!out.contains("files"), "{out}");
}

#[test]
fn loop_002_next_lists_changed_files_within_the_byte_budget() {
    let repo = support::Repo::with_lib("");
    repo.branch("feature");
    let root = repo.path();
    put(root, "openspec/specs/auth/spec.md", "## Purpose\nTokens.\n");
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &delta(),
    );
    put(root, "openspec/changes/add-oauth-login/tasks.md", TASKS);
    put(root, "src/auth.rs", "pub fn refresh() {}\n");
    let out = stdout(&mut sj(root, &["next", "add-oauth-login"]));
    assert!(out.contains("files\n"), "{out}");
    assert!(out.contains("  src/auth.rs\n"), "{out}");

    // Hundreds of changed files: the list is cut, the rest stays within 1,600 bytes.
    for i in 0..300 {
        put(
            root,
            &format!("src/generated/a_rather_long_module_name_number_{i:03}.rs"),
            "",
        );
    }
    let out = stdout(&mut sj(root, &["next", "add-oauth-login"]));
    assert!(out.contains(" more\n"), "{out}");
    assert!(out.contains("  sj test AUTH-006\n"), "{out}");
    let scenarios = scenario_text("AUTH-005", "Refresh an expiring token").len()
        + scenario_text("AUTH-006", "Reject a reused refresh token").len();
    assert!(out.len() <= 1600 + scenarios, "{} bytes: {out}", out.len());
}

#[test]
fn loop_002_next_with_no_open_task() {
    let dir = project();
    let file = dir.path().join("openspec/changes/add-oauth-login/tasks.md");
    std::fs::write(&file, TASKS.replace("[ ]", "[x]")).unwrap();
    sj(dir.path(), &["next"])
        .assert()
        .success()
        .stdout("next: no open tasks\n");
}

const PARALLEL_TASKS: &str = "# Tasks

- [x] 1.1 Setup (no-scenario: setup)
- [ ] 2.1 Not parallel (AUTH-005)
- [ ] 3.1 [P] Scopes (AUTH-007)
- [ ] 3.2 [P] Audience (AUTH-008)
- [ ] 3.3 [P] Issuer (AUTH-006)
";

/// AUTH-007 and AUTH-006 are tested in tests/a.rs, AUTH-008 in tests/b.rs.
fn parallel_project() -> tempfile::TempDir {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    put(root, "src/lib.rs", "");
    put(
        root,
        "tests/a.rs",
        "#[test]\nfn auth_007_scopes() {}\n\n#[test]\nfn auth_006_issuer() {}\n",
    );
    put(root, "tests/b.rs", "#[test]\nfn auth_008_audience() {}\n");
    put(
        root,
        "openspec/changes/add-oauth-login/tasks.md",
        PARALLEL_TASKS,
    );
    dir
}

#[test]
fn loop_003_parallel_tasks_are_listed_only_when_independent() {
    let dir = parallel_project();
    sj(dir.path(), &["next", "--parallel"])
        .assert()
        .success()
        .stdout(
            "batch 1\n  3.1 [P] Scopes (AUTH-007)\n  3.2 [P] Audience (AUTH-008)\n\
             batch 2\n  3.3 [P] Issuer (AUTH-006)\n",
        );
}

/// A repo branched from `main` whose change adds AUTH-005 and AUTH-006, with `tests/auth.rs`.
fn verify_repo(tests: &str) -> support::Repo {
    let repo = support::Repo::with_lib("");
    repo.branch("feature");
    repo.add_change(
        "add-oauth-login",
        &[("AUTH-005", "Refresh"), ("AUTH-006", "Reject")],
    );
    repo.write(
        "openspec/changes/add-oauth-login/tasks.md",
        "- [ ] 1.1 Tokens (AUTH-005, AUTH-006)\n",
    );
    repo.write("src/lib.rs", "pub fn refresh() {}\n");
    repo.write("tests/auth.rs", tests);
    repo.commit("tokens");
    repo
}

#[test]
fn loop_006_verify_catches_what_the_gate_would_catch() {
    let repo = verify_repo("#[test]\nfn auth_005_refresh() {}\n");
    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with("verify: "))
        .stdout(predicate::str::contains("untested AUTH-006\n"))
        .stdout(predicate::str::contains("failing").not());
}

#[test]
fn loop_006_verify_reports_failing_tests_and_open_questions() {
    let repo = verify_repo(
        "#[test]\nfn auth_005_refresh() { panic!(\"no\"); }\n\n#[test]\nfn auth_006_reject() {}\n",
    );
    repo.write(
        "openspec/changes/add-oauth-login/design.md",
        "## Context\n\n[NEEDS CLARIFICATION: which issuer?]\n",
    );
    sj(repo.path(), &["verify", "add-oauth-login"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("failing"))
        .stdout(predicate::str::contains("AUTH-005"))
        .stdout(predicate::str::contains(
            "clarify which issuer? (openspec/changes/add-oauth-login/design.md:3)",
        ))
        .stdout(predicate::str::contains(
            "unproven AUTH-006 (no red run recorded)",
        ));
}

#[test]
fn loop_006_verify_applies_the_unspecified_change_rule() {
    let repo = support::Repo::with_lib("");
    repo.branch("feature");
    repo.write("src/lib.rs", "pub fn sneaky() {}\n");
    repo.commit("no spec");
    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("unspecified change: src/lib.rs"));
}

#[test]
fn loop_007_verify_passes_a_finished_change() {
    let repo = verify_repo(
        "#[test]\nfn auth_005_refresh() { panic!(\"red\"); }\n\n#[test]\nfn auth_006_reject() { panic!(\"red\"); }\n",
    );
    Command::cargo_bin("sj")
        .unwrap()
        .args(["test", "--root"])
        .arg(repo.path())
        .assert()
        .code(1);
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_005_refresh() {}\n\n#[test]\nfn auth_006_reject() {}\n",
    );
    repo.commit("green");
    Command::cargo_bin("sj")
        .unwrap()
        .args(["test", "--root"])
        .arg(repo.path())
        .assert()
        .success();
    sj(repo.path(), &["verify"])
        .assert()
        .success()
        .stdout("verify: ok\n");
}

/// A change adding AUTH-005 `[keep]` and AUTH-006, both with passing tests and no red run.
fn keep_repo() -> support::Repo {
    let repo = support::Repo::with_lib("");
    repo.branch("feature");
    repo.add_change(
        "fix-tokens",
        &[("AUTH-005", "[keep] Refresh"), ("AUTH-006", "Reject")],
    );
    repo.write(
        "openspec/changes/fix-tokens/tasks.md",
        "- [ ] 1.1 Tokens (AUTH-005, AUTH-006)\n",
    );
    repo.write("src/lib.rs", "pub fn refresh() {}\n");
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_005_refresh() {}\n\n#[test]\nfn auth_006_reject() {}\n",
    );
    repo.commit("tokens");
    repo
}

/// `sj verify`, then `sj task done 1.1` (guided, the default mode): their stdout.
fn keep_outputs(repo: &support::Repo) -> (String, String) {
    let verify = stdout(&mut sj(repo.path(), &["verify"]));
    let done = stdout(&mut sj(repo.path(), &["task", "done", "fix-tokens", "1.1"]));
    (verify, done)
}

#[test]
fn loop_020_verify_and_task_done_accept_a_passing_keep_scenario() {
    let repo = keep_repo();
    let (verify, done) = keep_outputs(&repo);
    assert!(!verify.contains("unproven AUTH-005"), "{verify}");
    assert!(!verify.contains("failing"), "{verify}");
    assert!(done.contains("task: done 1.1"), "{done}");
    assert!(!done.contains("unproven AUTH-005"), "{done}");
}

#[test]
fn loop_021_a_passing_scenario_without_a_red_run_is_still_unproven() {
    let repo = keep_repo();
    let (verify, done) = keep_outputs(&repo);
    assert!(
        verify.contains("unproven AUTH-006 (no red run recorded)"),
        "{verify}"
    );
    assert!(
        done.contains("unproven AUTH-006 (no red run recorded)"),
        "{done}"
    );
}
