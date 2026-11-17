//! `sj spec` CLI scenarios (openspec/changes/spec-authoring).

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
    c.arg("spec").args(args).arg("--root").arg(root);
    c
}

/// A spec with AUTH-001..AUTH-004, one requirement each.
fn auth_spec() -> String {
    let mut s = String::from("# auth Specification\n\n## Purpose\nSign users in with tokens the API checks on every call.\n\n## Requirements\n");
    for (i, name) in ["Login", "Logout", "Refresh", "Revoke"].iter().enumerate() {
        s.push_str(&format!(
            "\n### Requirement: {name}\nThe system SHALL {name}.\n\n#### Scenario: AUTH-00{} {name} works\n- **WHEN** asked\n- **THEN** it works\n",
            i + 1
        ));
    }
    s
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/auth/spec.md", &auth_spec());
    dir
}

#[test]
fn spec_001_a_new_change_is_scaffolded() {
    let dir = project();
    put(
        dir.path(),
        "openspec/changes/archive/2026-01-01-old/specs/chat/spec.md",
        "## ADDED Requirements\n\n### Requirement: Send\nThe system SHALL send.\n\n#### Scenario: CHAT-007 Send\n- **WHEN** a\n- **THEN** b\n",
    );
    sj(dir.path(), &["new", "add-oauth-login"])
        .assert()
        .success()
        .stdout("created openspec/changes/add-oauth-login\nnext AUTH-005\nnext CHAT-008\n");
    let change = dir.path().join("openspec/changes/add-oauth-login");
    for f in ["proposal.md", "design.md", "tasks.md"] {
        let text = std::fs::read_to_string(change.join(f)).unwrap();
        assert!(
            text.starts_with("## ") || text.starts_with("# "),
            "{f}: {text}"
        );
    }
    assert!(change.join("specs").is_dir());
    assert_eq!(std::fs::read_dir(change.join("specs")).unwrap().count(), 0);
}

#[test]
fn spec_001_a_fresh_change_lists_with_no_tasks_or_scenarios() {
    let dir = project();
    sj(dir.path(), &["new", "add-oauth-login"])
        .assert()
        .success();
    // The templates hold no task lines and no scenarios.
    sj(dir.path(), &["list"])
        .assert()
        .success()
        .stdout("add-oauth-login  0/0 tasks  0 scenarios\n");
}

#[test]
fn spec_002_bad_or_duplicate_names_are_refused() {
    let dir = project();
    sj(dir.path(), &["new", "Add_OAuth"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("kebab-case"));
    assert!(!dir.path().join("openspec/changes").exists());

    sj(dir.path(), &["new", "add-oauth-login"])
        .assert()
        .success();
    std::fs::write(
        dir.path()
            .join("openspec/changes/add-oauth-login/proposal.md"),
        "## Why\n\nMine.\n",
    )
    .unwrap();
    sj(dir.path(), &["new", "add-oauth-login"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(
            dir.path()
                .join("openspec/changes/add-oauth-login/proposal.md")
        )
        .unwrap(),
        "## Why\n\nMine.\n"
    );
}

#[test]
fn spec_003_list_shows_progress() {
    let dir = project();
    let mut tasks = String::from("# Tasks\n\n");
    for i in 1..=7 {
        let mark = if i <= 3 { "x" } else { " " };
        tasks.push_str(&format!("- [{mark}] 1.{i} Step (AUTH-005)\n"));
    }
    put(
        dir.path(),
        "openspec/changes/add-oauth-login/tasks.md",
        &tasks,
    );
    let mut delta = String::from(
        "## ADDED Requirements\n\n### Requirement: OAuth\nThe system SHALL do OAuth.\n",
    );
    for i in 5..=7 {
        delta.push_str(&format!(
            "\n#### Scenario: AUTH-00{i} Case {i}\n- **WHEN** a\n- **THEN** b\n"
        ));
    }
    delta.push_str("\n## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL Login.\n\n#### Scenario: AUTH-001 Login works\n- **WHEN** asked\n- **THEN** it works via OAuth\n");
    delta.push_str("\n## REMOVED Requirements\n\n### Requirement: Revoke\n**Reason**: r\n**Migration**: m\n\n#### Scenario: AUTH-004 Revoke works\n- **WHEN** asked\n- **THEN** it works\n");
    put(
        dir.path(),
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &delta,
    );
    put(
        dir.path(),
        "openspec/changes/b/tasks.md",
        "- [x] 1.1 One (AUTH-001)\n",
    );
    sj(dir.path(), &["list"]).assert().success().stdout(
        "add-oauth-login  3/7 tasks  4 scenarios\n\
         b                1/1 tasks  0 scenarios\n",
    );
    let out = sj(dir.path(), &["list", "--json"]).assert().success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v[0]["name"], "add-oauth-login");
    assert_eq!(v[0]["tasks_done"], 3);
    assert_eq!(v[0]["tasks_total"], 7);
    assert_eq!(v[0]["scenarios"], 4);
}

#[test]
fn spec_003_no_open_changes() {
    let dir = project();
    sj(dir.path(), &["list"])
        .assert()
        .success()
        .stdout("no open changes\n");
}

// ---------- validate ----------

/// A change `add-refresh` adding AUTH-005, with one task naming it.
fn with_change(dir: &Path, then_line: &str) {
    put(
        dir,
        "openspec/changes/add-refresh/specs/auth/spec.md",
        &format!(
            "## ADDED Requirements\n\n### Requirement: Rotate\nThe system SHALL rotate tokens.\n\n#### Scenario: AUTH-005 Rotate\n- **WHEN** a token is old\n{then_line}"
        ),
    );
    put(
        dir,
        "openspec/changes/add-refresh/tasks.md",
        "# Tasks\n\n- [x] 1.1 Rotate (AUTH-005)\n",
    );
}

#[test]
fn spec_004_validate_reports_a_missing_then_and_exits_1() {
    let dir = project();
    with_change(dir.path(), "");
    sj(dir.path(), &["validate", "add-refresh"])
        .assert()
        .code(1)
        .stdout(
            "validate: 1 error, 0 warnings\n\
             fail    AUTH-005 has no THEN step (openspec/changes/add-refresh/specs/auth/spec.md:6)\n",
        );
}

#[test]
fn spec_004_a_clean_tree_validates() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    sj(dir.path(), &["validate"])
        .assert()
        .success()
        .stdout("validate: ok\n");
}

#[test]
fn spec_004_strict_fails_on_warnings() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    std::fs::remove_file(dir.path().join("openspec/changes/add-refresh/tasks.md")).unwrap();
    sj(dir.path(), &["validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "warn    change has no tasks.md (openspec/changes/add-refresh)",
        ));
    sj(dir.path(), &["validate", "--strict"]).assert().code(1);
}

#[test]
fn spec_005_modified_must_name_an_existing_requirement() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    put(
        dir.path(),
        "openspec/changes/add-refresh/specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Login flow\nThe system SHALL log in.\n\n#### Scenario: AUTH-001 Login works\n- **WHEN** a\n- **THEN** b\n",
    );
    sj(dir.path(), &["validate", "add-refresh"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    MODIFIED requirement not found: Login flow (",
        ));
}

#[test]
fn spec_006_a_task_without_a_scenario_id_is_reported() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    put(
        dir.path(),
        "openspec/changes/add-refresh/tasks.md",
        "# Tasks\n\n- [x] 1.1 Rotate (AUTH-005)\n- [ ] 2.1 Wire up the router\n",
    );
    sj(dir.path(), &["validate", "add-refresh"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    task names no scenario: 2.1 Wire up the router (openspec/changes/add-refresh/tasks.md:4)",
        ));
}

// ---------- check ----------

#[test]
fn spec_007_check_reports_uncovered_and_clarify() {
    let dir = project();
    with_change(
        dir.path(),
        "- **THEN** it is replaced\n\n#### Scenario: AUTH-006 Rotate twice\n- **WHEN** again\n- **THEN** replaced again\n",
    );
    put(
        dir.path(),
        "openspec/changes/add-refresh/proposal.md",
        "## Why\n\nRotation.\n\n[NEEDS CLARIFICATION: token lifetime?]\n",
    );
    sj(dir.path(), &["check", "add-refresh"])
        .assert()
        .code(1)
        .stdout(
            "check: 2 problems\n\
         uncovered AUTH-006\n\
         clarify token lifetime? (proposal.md:5)\n",
        );
}

#[test]
fn spec_007_a_covered_change_checks_clean() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    sj(dir.path(), &["check", "add-refresh"])
        .assert()
        .success()
        .stdout("check: ok\n");
}

// ---------- archive ----------

#[test]
fn spec_008_archive_moves_the_change_under_todays_date() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    sj(dir.path(), &["archive", "add-refresh"])
        .assert()
        .success()
        .stdout(format!(
            "updated openspec/specs/auth/spec.md (+1 ~0 -0 →0)\n\
             archived add-refresh → openspec/changes/archive/{today}-add-refresh\n"
        ));
    let root = dir.path().join("openspec");
    assert!(root
        .join(format!("changes/archive/{today}-add-refresh/tasks.md"))
        .is_file());
    assert!(!root.join("changes/add-refresh").exists());
    let spec = std::fs::read_to_string(root.join("specs/auth/spec.md")).unwrap();
    assert!(
        spec.trim_end().ends_with("- **THEN** it is replaced"),
        "{spec}"
    );
}

#[test]
fn spec_009_archive_refuses_open_tasks() {
    let dir = project();
    with_change(dir.path(), "- **THEN** it is replaced\n");
    put(
        dir.path(),
        "openspec/changes/add-refresh/tasks.md",
        "# Tasks\n\n- [ ] 1.1 Rotate (AUTH-005)\n",
    );
    let before = std::fs::read_to_string(dir.path().join("openspec/specs/auth/spec.md")).unwrap();
    sj(dir.path(), &["archive", "add-refresh"])
        .assert()
        .code(1)
        .stdout(
            "archive: refused\n\
             fail    open task 1.1 Rotate (AUTH-005) (openspec/changes/add-refresh/tasks.md:3)\n",
        );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("openspec/specs/auth/spec.md")).unwrap(),
        before
    );
    assert!(dir.path().join("openspec/changes/add-refresh").is_dir());
    sj(dir.path(), &["archive", "add-refresh", "--force"])
        .assert()
        .success();
}

#[test]
fn spec_010_archive_creates_a_new_capability() {
    let dir = project();
    put(
        dir.path(),
        "openspec/changes/add-billing/specs/billing/spec.md",
        "## Purpose\n\nCharge customers monthly and send them an invoice.\n\n## ADDED Requirements\n\n### Requirement: Invoice\nThe system SHALL invoice.\n\n#### Scenario: BILL-001 Invoice\n- **WHEN** the month starts\n- **THEN** an invoice exists\n",
    );
    put(
        dir.path(),
        "openspec/changes/add-billing/tasks.md",
        "- [x] 1.1 Invoice (BILL-001)\n",
    );
    sj(dir.path(), &["archive", "add-billing"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "created openspec/specs/billing/spec.md (+1)\n",
        ));
    let spec = std::fs::read_to_string(dir.path().join("openspec/specs/billing/spec.md")).unwrap();
    assert!(spec.starts_with(
        "# billing Specification\n\n## Purpose\nCharge customers monthly and send them an invoice.\n\n## Requirements\n"
    ), "{spec}");
}
