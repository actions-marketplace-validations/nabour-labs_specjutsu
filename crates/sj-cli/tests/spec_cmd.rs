//! `sj spec` CLI scenarios (specjutsu/changes/spec-authoring).

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

/// `with_change` plus a second finished change, `add-login`, that adds to the same capability.
fn with_two_changes(dir: &Path, refresh_tasks: &str) {
    with_change(dir, "- **THEN** it is replaced\n");
    put(dir, "openspec/changes/add-refresh/tasks.md", refresh_tasks);
    put(
        dir,
        "openspec/changes/add-login/specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Session\nThe system SHALL keep a session.\n\n#### Scenario: AUTH-006 Session\n- **WHEN** a user logs in\n- **THEN** a session exists\n",
    );
    put(
        dir,
        "openspec/changes/add-login/tasks.md",
        "# Tasks\n\n- [x] 1.1 Session (AUTH-006)\n",
    );
}

/// Every file under `dir`, relative path to contents.
fn snapshot(dir: &Path) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read_to_string(&p).unwrap());
            }
        }
    }
    out
}

#[test]
fn spec_030_archive_several_finished_changes() {
    let dir = project();
    with_two_changes(dir.path(), "# Tasks\n\n- [x] 1.1 Rotate (AUTH-005)\n");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    sj(dir.path(), &["archive", "add-login", "add-refresh"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "archived add-login → openspec/changes/archive/{today}-add-login\n"
        )))
        .stdout(predicate::str::contains(format!(
            "archived add-refresh → openspec/changes/archive/{today}-add-refresh\n"
        )));
    let root = dir.path().join("openspec");
    for name in ["add-login", "add-refresh"] {
        assert!(
            !root.join("changes").join(name).exists(),
            "{name} not moved"
        );
        assert!(root
            .join(format!("changes/archive/{today}-{name}/tasks.md"))
            .is_file());
    }
    // Both deltas land in the one spec: the second merge builds on the first.
    let spec = std::fs::read_to_string(root.join("specs/auth/spec.md")).unwrap();
    assert!(spec.contains("### Requirement: Session"), "{spec}");
    assert!(spec.contains("### Requirement: Rotate"), "{spec}");
    assert!(spec.contains("### Requirement: Revoke"), "{spec}");
}

#[test]
fn spec_031_one_failing_change_archives_nothing() {
    let dir = project();
    with_two_changes(dir.path(), "# Tasks\n\n- [ ] 1.1 Rotate (AUTH-005)\n");
    let before = snapshot(&dir.path().join("openspec"));
    sj(dir.path(), &["archive", "add-login", "add-refresh"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("add-refresh"))
        .stdout(predicate::str::contains("open task 1.1 Rotate (AUTH-005)"))
        .stdout(predicate::str::contains("archived").not());
    assert_eq!(snapshot(&dir.path().join("openspec")), before);
}

/// A change that MODIFIES `auth`'s `Login flow` with its own THEN line.
fn modifies_login_flow(dir: &Path, change: &str, then_line: &str) {
    put(
        dir,
        &format!("openspec/changes/{change}/specs/auth/spec.md"),
        &format!(
            "## MODIFIED Requirements\n\n### Requirement: Login flow\nThe system SHALL sign users in.\n\n#### Scenario: AUTH-001 Login works\n- **WHEN** asked\n- **THEN** {then_line}\n"
        ),
    );
    put(
        dir,
        &format!("openspec/changes/{change}/tasks.md"),
        "# Tasks\n\n- [x] 1.1 Login flow (AUTH-001)\n",
    );
}

#[test]
fn spec_032_two_changes_editing_one_requirement_are_refused() {
    let dir = project();
    put(
        dir.path(),
        "openspec/specs/auth/spec.md",
        &auth_spec().replace("Requirement: Login\n", "Requirement: Login flow\n"),
    );
    modifies_login_flow(dir.path(), "add-login", "it sends a code");
    modifies_login_flow(dir.path(), "add-sso", "it redirects to the provider");
    let before = snapshot(&dir.path().join("openspec"));
    sj(dir.path(), &["archive", "add-login", "add-sso"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("add-login"))
        .stdout(predicate::str::contains("add-sso"))
        .stdout(predicate::str::contains("auth: Login flow"))
        .stdout(predicate::str::contains("archived").not());
    assert_eq!(snapshot(&dir.path().join("openspec")), before);

    // Each one alone still archives.
    sj(dir.path(), &["archive", "add-login"]).assert().success();
}

#[test]
fn spec_033_all_done_archives_only_finished_changes() {
    let dir = project();
    with_two_changes(dir.path(), "# Tasks\n\n- [x] 1.1 Rotate (AUTH-005)\n");
    put(
        dir.path(),
        "openspec/changes/add-sso/specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Single sign-on\nThe system SHALL accept SSO.\n\n#### Scenario: AUTH-007 SSO\n- **WHEN** a provider signs a user in\n- **THEN** a session exists\n",
    );
    put(
        dir.path(),
        "openspec/changes/add-sso/tasks.md",
        "# Tasks\n\n- [ ] 1.1 Provider (AUTH-007)\n- [ ] 1.2 Callback (AUTH-007)\n",
    );
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let out = sj(dir.path(), &["archive", "--all-done"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(out).unwrap();
    let archived: Vec<&str> = out.lines().filter(|l| l.starts_with("archived ")).collect();
    assert_eq!(
        archived,
        vec![
            format!("archived add-login → openspec/changes/archive/{today}-add-login"),
            format!("archived add-refresh → openspec/changes/archive/{today}-add-refresh"),
        ],
        "{out}"
    );
    let changes = dir.path().join("openspec/changes");
    assert!(changes.join("add-sso/tasks.md").is_file());
    assert!(!changes.join("add-login").exists());
    assert!(!changes.join("add-refresh").exists());

    // Rerunning with nothing finished is a no-op.
    sj(dir.path(), &["archive", "--all-done"])
        .assert()
        .success()
        .stdout("nothing to archive\n");
}

// ---------- MODIFIED scenarios whose original has no ID ----------

/// `<root>/specs/staff/spec.md` with `Role changes are audited` holding the scenario
/// `original`, and a change `audit-roles` whose MODIFIED copy of it has `scenario`.
fn with_original(dir: &Path, root: &str, original: &str, scenario: &str) {
    put(
        dir,
        &format!("{root}/specs/staff/spec.md"),
        &format!(
            "# staff Specification\n\n## Purpose\nManage staff accounts and roles.\n\n## Requirements\n\n### Requirement: Role changes are audited\nThe system SHALL record role changes.\n\n#### Scenario: {original}\n- **WHEN** a role changes\n- **THEN** it is recorded\n"
        ),
    );
    put(
        dir,
        &format!("{root}/changes/audit-roles/specs/staff/spec.md"),
        &format!(
            "## MODIFIED Requirements\n\n### Requirement: Role changes are audited\nThe system SHALL record role changes with the actor.\n\n#### Scenario: {scenario}\n- **WHEN** a role changes\n- **THEN** it is recorded with the actor\n"
        ),
    );
    put(
        dir,
        &format!("{root}/changes/audit-roles/tasks.md"),
        "# Tasks\n\n- [ ] 1.1 Record the actor (STAFF-057)\n",
    );
}

/// The original without an ID, in an `openspec/` tree.
fn with_unnamed_original(dir: &Path, scenario: &str) {
    with_original(dir, "openspec", "Role change is recorded", scenario);
}

fn spec_045_warning(root: &str) -> String {
    format!(
        "validate: 0 errors, 1 warning\n\
         warn    STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario without an ID; run sj ids --write staff first ({root}/changes/audit-roles/specs/staff/spec.md:6)\n"
    )
}

#[test]
fn spec_045_an_id_the_original_lacks_is_a_warning_with_the_fix() {
    let dir = project();
    with_unnamed_original(dir.path(), "STAFF-057 Role change is recorded");
    let warning = spec_045_warning("openspec");
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(0)
        .stdout(warning.clone());
    sj(dir.path(), &["validate", "audit-roles", "--strict"])
        .assert()
        .code(1)
        .stdout(warning);
}

#[test]
fn spec_045_a_specjutsu_root_gives_the_same_warning() {
    let dir = tempfile::tempdir().unwrap();
    with_original(
        dir.path(),
        "specjutsu",
        "Role change is recorded",
        "STAFF-057 Role change is recorded",
    );
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(0)
        .stdout(spec_045_warning("specjutsu"));
}

#[test]
fn spec_045_the_keep_marker_is_ignored_when_matching_names() {
    let dir = project();
    with_unnamed_original(dir.path(), "STAFF-057 [keep] Role change is recorded");
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(0)
        .stdout(spec_045_warning("openspec"));
}

// Regression: pins behaviour validate had before this change, so it carries no scenario ID.
#[test]
fn regression_no_hint_or_warning_when_the_original_has_the_same_id() {
    let dir = project();
    with_original(
        dir.path(),
        "openspec",
        "STAFF-057 Role change is recorded",
        "STAFF-057 Role change is recorded",
    );
    sj(dir.path(), &["validate", "audit-roles", "--strict"])
        .assert()
        .success()
        .stdout("validate: ok\n");
}

#[test]
fn spec_047_different_ids_on_a_copy_and_its_original_are_a_warning() {
    let dir = project();
    with_original(
        dir.path(),
        "openspec",
        "STAFF-100 Role change is recorded",
        "STAFF-057 Role change is recorded",
    );
    let warning = "validate: 0 errors, 1 warning\n\
         warn    STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario as STAFF-100; give the delta copy STAFF-100 (openspec/changes/audit-roles/specs/staff/spec.md:6)\n";
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(0)
        .stdout(warning);
    sj(dir.path(), &["validate", "audit-roles", "--strict"])
        .assert()
        .code(1)
        .stdout(warning);
}

/// The delta of `audit-roles` with `body` under `## RENAMED` / `## MODIFIED` sections.
fn with_delta(dir: &Path, body: &str) {
    put(
        dir,
        "openspec/changes/audit-roles/specs/staff/spec.md",
        body,
    );
}

// Regression: an edge of SPEC-047 that base already passed, so it carries no scenario ID.
// The original's ID moved to a renamed scenario in the same block, so giving the copy that ID
// would make two.
#[test]
fn regression_no_different_id_warning_when_the_block_already_uses_the_originals_id() {
    let dir = project();
    with_original(
        dir.path(),
        "openspec",
        "STAFF-100 Role change is recorded",
        "STAFF-057 Role change is recorded",
    );
    with_delta(
        dir.path(),
        "## MODIFIED Requirements\n\n### Requirement: Role changes are audited\nThe system SHALL record role changes with the actor.\n\n\
         #### Scenario: STAFF-100 Role change is recorded with the actor\n- **WHEN** a role changes\n- **THEN** it is recorded with the actor\n\n\
         #### Scenario: STAFF-057 Role change is recorded\n- **WHEN** a role changes\n- **THEN** it is recorded\n",
    );
    sj(dir.path(), &["validate", "audit-roles", "--strict"])
        .assert()
        .success()
        .stdout("validate: ok\n");
}

// Regression: SPEC-047 through a RENAMED requirement, which matches by its old name.
#[test]
fn regression_a_renamed_requirement_gets_the_different_id_warning() {
    let dir = project();
    with_original(
        dir.path(),
        "openspec",
        "STAFF-100 Role change is recorded",
        "STAFF-057 Role change is recorded",
    );
    with_delta(
        dir.path(),
        "## RENAMED Requirements\n\
         - FROM: `### Requirement: Role changes are audited`\n\
         - TO: `### Requirement: Role changes are logged`\n\n\
         ## MODIFIED Requirements\n\n### Requirement: Role changes are logged\nThe system SHALL log role changes.\n\n\
         #### Scenario: STAFF-057 Role change is recorded\n- **WHEN** a role changes\n- **THEN** it is recorded\n",
    );
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "warn    STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario as STAFF-100; give the delta copy STAFF-100 (openspec/changes/audit-roles/specs/staff/spec.md:10)\n",
        ));
}

#[test]
fn spec_046_a_missing_id_in_a_modified_copy_names_the_same_fix() {
    let dir = project();
    with_unnamed_original(dir.path(), "Role change is recorded");
    sj(dir.path(), &["validate", "audit-roles"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    Role change is recorded has no ID; run sj ids --write staff first (openspec/changes/audit-roles/specs/staff/spec.md:6)\n",
        ));
}
