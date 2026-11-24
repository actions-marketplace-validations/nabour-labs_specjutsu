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

/// Base `main`: AUTH-001 specified, so AUTH is a known prefix and a test naming a missing
/// AUTH ID is an orphan.
fn auth_base() -> support::Repo {
    let repo = support::Repo::with_lib("pub fn lib() {}\n");
    repo.write(
        "openspec/specs/auth/spec.md",
        &support::spec_md(&[("AUTH-001", "Logs in")]),
    );
    repo
}

#[test]
fn loop_032_a_baselined_orphan_is_not_a_verify_problem() {
    let repo = auth_base();
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_009_old() { assert!(true); }\n",
    );
    repo.write(
        "openspec/trace-baseline.txt",
        "orphan AUTH-009 tests/auth.rs auth_009_old\n",
    );
    repo.commit("base with baselined orphan");
    repo.branch("feature");
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");

    sj(repo.path(), &["verify"])
        .assert()
        .success()
        .stdout("verify: ok\n");
}

#[test]
fn loop_033_a_baseline_entry_added_without_an_exemption_is_a_verify_problem() {
    let repo = auth_base();
    repo.commit("base");
    repo.branch("feature");
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_010_x() { assert!(true); }\n",
    );
    repo.write(
        "openspec/trace-baseline.txt",
        "orphan AUTH-010 tests/auth.rs auth_010_x\n",
    );
    repo.commit("test: AUTH-010, baselined");

    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\nbaseline-added orphan AUTH-010 tests/auth.rs auth_010_x\n",
        ));
}

/// Base `main` specifies STAFF-063 with no test; branch `feature` adds the change `link-staff`,
/// whose MODIFIED requirement copies STAFF-063 unchanged and whose task 4.1 names it, and its
/// first test, passing. Strict mode.
fn linked_repo() -> support::Repo {
    let scenarios = [("STAFF-063", "Lists staff")];
    let repo = support::Repo::with_lib("pub fn staff() {}\n");
    repo.write(
        "specjutsu.toml",
        "schema = 1\nmode = \"strict\"\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n\
         [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**\"]\n",
    );
    repo.write(
        "openspec/specs/staff/spec.md",
        &support::spec_md(&scenarios),
    );
    repo.commit("base: STAFF-063 without a test");
    repo.branch("feature");
    repo.write(
        "openspec/changes/link-staff/proposal.md",
        "## Why\n\nLink.\n",
    );
    repo.write(
        "openspec/changes/link-staff/specs/staff/spec.md",
        &support::spec_md(&scenarios).replace("## ADDED", "## MODIFIED"),
    );
    repo.write(
        "openspec/changes/link-staff/tasks.md",
        "## 4. Staff\n\n- [ ] 4.1 Test the staff list (STAFF-063)\n",
    );
    repo.write("tests/staff.rs", "#[test]\nfn staff_063_lists_staff() {}\n");
    repo.commit("test: STAFF-063");
    repo
}

/// The ledger's lines.
fn ledger_text(repo: &support::Repo) -> String {
    std::fs::read_to_string(repo.path().join(".specjutsu/ledger/runs.jsonl")).unwrap_or_default()
}

#[test]
fn loop_034_verify_and_task_done_accept_a_linked_existing_scenario() {
    let repo = linked_repo();
    let verify = stdout(&mut sj(repo.path(), &["verify"]));
    assert!(!verify.contains("unproven STAFF-063"), "{verify}");
    let ledger = ledger_text(&repo);
    assert!(ledger.contains("STAFF-063"), "{verify}{ledger}");
    assert!(!ledger.contains("\"failed\""), "{ledger}");

    let file = repo.path().join("openspec/changes/link-staff/tasks.md");
    sj(repo.path(), &["task", "done", "link-staff", "4.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("task: done 4.1"));
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .contains("- [x] 4.1 Test the staff list (STAFF-063)"));
}

#[test]
fn loop_034_a_linked_scenario_never_red_does_not_cover_a_source_change() {
    let repo = linked_repo();
    repo.write("src/lib.rs", "pub fn staff() { let _ = 1; }\n");
    repo.commit("edit source");
    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("unspecified change: src/lib.rs"));
}

/// Base `main` has AUTH-002 with a passing test; branch `feature` adds a change whose
/// `## MODIFIED Requirements` copies AUTH-002 with the same text.
fn unchanged_repo() -> support::Repo {
    let scenarios = [("AUTH-002", "Logs out")];
    let repo = support::Repo::with_lib("pub fn logout() {}\n");
    repo.write("openspec/specs/auth/spec.md", &support::spec_md(&scenarios));
    repo.write("tests/auth.rs", "#[test]\nfn auth_002_logs_out() {}\n");
    repo.commit("base: AUTH-002 tested");
    repo.branch("feature");
    repo.write(
        "openspec/changes/touch-auth/proposal.md",
        "## Why\n\nTouch.\n",
    );
    repo.write(
        "openspec/changes/touch-auth/specs/auth/spec.md",
        &support::spec_md(&scenarios).replace("## ADDED", "## MODIFIED"),
    );
    repo.commit("spec: copy AUTH-002");
    repo
}

#[test]
fn loop_035_an_unchanged_scenario_in_a_modified_requirement_needs_no_red_run() {
    let repo = unchanged_repo();
    let verify = stdout(&mut sj(repo.path(), &["verify"]));
    assert!(!verify.contains("unproven AUTH-002"), "{verify}");
    let ledger = ledger_text(&repo);
    assert!(ledger.contains("AUTH-002"), "{ledger}");
    assert!(!ledger.contains("\"failed\""), "{ledger}");
}

#[test]
fn loop_035_a_failing_unchanged_scenario_is_still_reported() {
    let repo = unchanged_repo();
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_002_logs_out() { panic!(\"broken\"); }\n",
    );
    repo.commit("break AUTH-002");
    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_match(r"failing \S+  AUTH-002\n").unwrap());
}

#[test]
fn loop_034_a_linked_scenario_proven_red_covers_a_source_change() {
    let repo = linked_repo();
    repo.write(
        "tests/staff.rs",
        "#[test]\nfn staff_063_lists_staff() { panic!(\"red\"); }\n",
    );
    repo.commit("red");
    sj(repo.path(), &["test", "STAFF-063"]).assert().code(1);
    repo.write("tests/staff.rs", "#[test]\nfn staff_063_lists_staff() {}\n");
    repo.write("src/lib.rs", "pub fn staff() { let _ = 1; }\n");
    repo.commit("green");
    sj(repo.path(), &["test", "STAFF-063"]).assert().success();
    sj(repo.path(), &["verify"])
        .assert()
        .success()
        .stdout("verify: ok\n");
}

#[test]
fn loop_036_an_added_scenario_committed_on_the_default_branch_still_needs_a_red_run() {
    let repo = support::Repo::with_lib("pub fn x() {}\n");
    repo.write(
        "specjutsu.toml",
        "schema = 1\nmode = \"strict\"\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n\
         [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**\"]\n",
    );
    repo.commit("config");
    repo.add_change("add-x", &[("MATH-001", "Adds")]);
    repo.write(
        "openspec/changes/add-x/tasks.md",
        "## 1. X\n\n- [ ] 1.1 Add (MATH-001)\n",
    );
    repo.write("tests/math.rs", "#[test]\nfn math_001_adds() {}\n");
    repo.commit("feat: MATH-001 on main");

    sj(repo.path(), &["verify"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unproven MATH-001 (no red run recorded)",
        ));
    assert!(!ledger_text(&repo).contains("\"failed\""));
    let file = repo.path().join("openspec/changes/add-x/tasks.md");
    sj(repo.path(), &["task", "done", "add-x", "1.1"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unproven MATH-001 (no red run recorded)",
        ));
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .contains("- [ ] 1.1 Add (MATH-001)"));
}
