//! SPEC-004..007 from openspec/changes/spec-authoring: `sj spec validate` and `sj spec check`.

use std::path::Path;

use sj_core::authoring::check::check;
use sj_core::authoring::validate::{validate, Problem};

const AUTH: &str = "# auth Specification\n\n## Purpose\nSign users in and out with tokens the API checks.\n\n## Requirements\n\n\
### Requirement: Login\nThe system SHALL start a session for valid credentials.\n\n\
#### Scenario: AUTH-001 Valid login\n- **WHEN** the credentials match\n- **THEN** a session starts\n\n\
### Requirement: Logout\nThe system SHALL end the session.\n\n\
#### Scenario: AUTH-002 Logout\n- **WHEN** the user logs out\n- **THEN** the session ends\n";

const DELTA: &str = "## ADDED Requirements\n\n### Requirement: Refresh\nThe system SHALL exchange refresh tokens.\n\n\
#### Scenario: AUTH-003 Refresh\n- **WHEN** a refresh token is sent\n- **THEN** a new token is returned\n\n\
## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL start a session and return a refresh token.\n\n\
#### Scenario: AUTH-001 Valid login\n- **WHEN** the credentials match\n- **THEN** a session starts with a refresh token\n";

const TASKS: &str = "# Tasks\n\n## 1. Refresh\n\n- [ ] 1.1 Token exchange (AUTH-003)\n- [x] 1.2 Login returns it (AUTH-001)\n- [ ] 1.3 README (no-scenario: docs)\n";

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A spec root with `specs/auth` and an open change `add-refresh`, all valid.
fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/auth/spec.md", AUTH);
    put(dir.path(), "changes/add-refresh/proposal.md", "## Why\n\nTokens expire.\n");
    put(dir.path(), "changes/add-refresh/tasks.md", TASKS);
    put(dir.path(), "changes/add-refresh/specs/auth/spec.md", DELTA);
    dir
}

fn delta(dir: &Path, text: &str) {
    put(dir, "changes/add-refresh/specs/auth/spec.md", text);
}

/// (message, spec-root-relative file, line).
fn msgs(ps: &[Problem], root: &Path) -> Vec<(String, String, usize)> {
    ps.iter()
        .map(|p| {
            let rel = p.file.strip_prefix(root).unwrap_or(&p.file);
            (
                p.message.clone(),
                rel.to_string_lossy().replace('\\', "/"),
                p.line,
            )
        })
        .collect()
}

fn errors(dir: &Path, change: Option<&str>) -> Vec<(String, String, usize)> {
    msgs(&validate(dir, change).unwrap().errors, dir)
}

const D: &str = "changes/add-refresh/specs/auth/spec.md";

#[test]
fn spec_004_a_valid_tree_has_no_problems() {
    let dir = tree();
    let r = validate(dir.path(), None).unwrap();
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let r = validate(dir.path(), Some("add-refresh")).unwrap();
    assert!(r.errors.is_empty(), "{:?}", r.errors);
}

#[test]
fn spec_004_a_scenario_without_then_is_reported() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("- **THEN** a new token is returned\n", ""));
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("AUTH-003 has no THEN step".into(), D.into(), 6)]
    );
}

#[test]
fn spec_004_a_scenario_without_when_is_reported() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("- **WHEN** a refresh token is sent\n", ""));
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("AUTH-003 has no WHEN step".into(), D.into(), 6)]
    );
}

#[test]
fn spec_004_a_scenario_without_an_id_is_reported() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("AUTH-003 Refresh", "Refresh works"));
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("Refresh works has no ID".into(), D.into(), 6)]
    );
}

#[test]
fn spec_004_a_duplicate_id_is_reported() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("AUTH-003 Refresh", "AUTH-002 Refresh"));
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [(
            "duplicate ID AUTH-002 (first at specs/auth/spec.md:18)".into(),
            D.into(),
            6
        )]
    );
}

#[test]
fn spec_004_two_open_changes_adding_one_id_is_a_duplicate() {
    let dir = tree();
    put(dir.path(), "changes/other/tasks.md", "- [ ] 1.1 Other (AUTH-003)\n");
    put(
        dir.path(),
        "changes/other/specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Other\nThe system SHALL x.\n\n#### Scenario: AUTH-003 Other\n- **WHEN** a\n- **THEN** b\n",
    );
    let e = errors(dir.path(), None);
    assert!(
        e.iter().any(|(m, f, _)| m.starts_with("duplicate ID AUTH-003") && f == "changes/other/specs/auth/spec.md"),
        "{e:?}"
    );
}

#[test]
fn spec_004_a_requirement_needs_a_scenario_and_shall() {
    let dir = tree();
    delta(
        dir.path(),
        &format!("{DELTA}\n## ADDED Requirements\n\n### Requirement: Rotation\nTokens rotate.\n\n### Requirement: Revoke\nThe system MUST revoke.\n"),
    );
    let e = errors(dir.path(), Some("add-refresh"));
    assert!(e.contains(&("requirement Rotation has no SHALL or MUST".into(), D.into(), 21)), "{e:?}");
    assert!(e.contains(&("requirement Rotation has no scenario".into(), D.into(), 21)), "{e:?}");
    assert!(e.contains(&("requirement Revoke has no scenario".into(), D.into(), 24)), "{e:?}");
    assert_eq!(e.len(), 3, "{e:?}");
}

#[test]
fn spec_004_delta_requirements_must_sit_in_a_delta_section() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("## ADDED Requirements", "## Requirements"));
    let e = errors(dir.path(), Some("add-refresh"));
    assert!(
        e.contains(&(
            "requirement Refresh is outside ADDED/MODIFIED/REMOVED/RENAMED Requirements".into(),
            D.into(),
            3
        )),
        "{e:?}"
    );
}

#[test]
fn spec_004_a_delta_without_operations_is_reported() {
    let dir = tree();
    delta(dir.path(), "## Notes\n\nNothing yet.\n");
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("no ADDED, MODIFIED, REMOVED or RENAMED requirements".into(), D.into(), 0)]
    );
}

#[test]
fn spec_004_removed_needs_reason_and_migration() {
    let dir = tree();
    delta(
        dir.path(),
        &format!("{DELTA}\n## REMOVED Requirements\n\n### Requirement: Logout\n**Reason**: Sessions expire instead.\n"),
    );
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [(
            "REMOVED requirement Logout needs **Reason** and **Migration**".into(),
            D.into(),
            21
        )]
    );
}

#[test]
fn spec_005_modified_must_name_an_existing_requirement() {
    let dir = tree();
    delta(dir.path(), &DELTA.replace("### Requirement: Login\n", "### Requirement: Login flow\n"));
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("MODIFIED requirement not found: Login flow".into(), D.into(), 12)]
    );
}

#[test]
fn spec_005_removed_and_renamed_must_name_existing_requirements() {
    let dir = tree();
    delta(
        dir.path(),
        "## REMOVED Requirements\n\n- `### Requirement: Gone`\n\n**Reason**: r\n**Migration**: m\n\n\
         ## RENAMED Requirements\n\n- FROM: `### Requirement: Nope`\n- TO: `### Requirement: Still nope`\n",
    );
    let e = errors(dir.path(), Some("add-refresh"));
    assert!(e.contains(&("REMOVED requirement not found: Gone".into(), D.into(), 3)), "{e:?}");
    assert!(e.contains(&("RENAMED requirement not found: Nope".into(), D.into(), 10)), "{e:?}");
}

#[test]
fn spec_005_modified_may_use_a_name_renamed_in_the_same_delta() {
    let dir = tree();
    delta(
        dir.path(),
        &format!(
            "## RENAMED Requirements\n\n- FROM: `### Requirement: Login`\n- TO: `### Requirement: Sign in`\n\n{}",
            DELTA.replace("### Requirement: Login\n", "### Requirement: Sign in\n")
        ),
    );
    assert_eq!(errors(dir.path(), Some("add-refresh")), []);
}

#[test]
fn spec_005_an_unpaired_rename_is_reported() {
    let dir = tree();
    delta(
        dir.path(),
        &format!("{DELTA}\n## RENAMED Requirements\n\n- FROM: `### Requirement: Logout`\n"),
    );
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [("RENAMED FROM without its pair: Logout".into(), D.into(), 21)]
    );
}

#[test]
fn spec_006_a_task_without_a_scenario_id_is_reported() {
    let dir = tree();
    put(
        dir.path(),
        "changes/add-refresh/tasks.md",
        &format!("{TASKS}\n## 2. Router\n\n- [ ] 2.1 Wire up the router\n- [ ] 2.2 Range (AUTH-001..003)\n"),
    );
    assert_eq!(
        errors(dir.path(), Some("add-refresh")),
        [(
            "task names no scenario: 2.1 Wire up the router".into(),
            "changes/add-refresh/tasks.md".into(),
            11
        )]
    );
}

#[test]
fn spec_006_fenced_lines_are_not_tasks_or_headings() {
    let dir = tree();
    put(
        dir.path(),
        "changes/add-refresh/tasks.md",
        &format!("{TASKS}\n```md\n- [ ] 9.9 An example task\n#### Scenario: Example\n```\n"),
    );
    assert_eq!(errors(dir.path(), Some("add-refresh")), []);
}

#[test]
fn spec_004_specs_are_checked_too_when_no_change_is_named() {
    let dir = tree();
    put(
        dir.path(),
        "specs/auth/spec.md",
        &AUTH.replace("- **WHEN** the user logs out\n", ""),
    );
    assert_eq!(
        errors(dir.path(), None),
        [("AUTH-002 has no WHEN step".into(), "specs/auth/spec.md".into(), 18)]
    );
    // Naming a change checks only that change.
    assert_eq!(errors(dir.path(), Some("add-refresh")), []);
}

#[test]
fn spec_004_warnings_for_placeholder_purpose_and_missing_tasks() {
    let dir = tree();
    put(
        dir.path(),
        "specs/auth/spec.md",
        &AUTH.replace(
            "Sign users in and out with tokens the API checks.",
            "TBD - created by archiving change x. Update Purpose after archive.",
        ),
    );
    std::fs::remove_file(dir.path().join("changes/add-refresh/tasks.md")).unwrap();
    let r = validate(dir.path(), None).unwrap();
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    assert_eq!(
        msgs(&r.warnings, dir.path()),
        [
            ("Purpose is a placeholder".into(), "specs/auth/spec.md".into(), 4),
            ("change has no tasks.md".into(), "changes/add-refresh".into(), 0),
        ]
    );
}

#[test]
fn spec_004_an_unknown_change_is_an_error() {
    let dir = tree();
    let e = format!("{:#}", validate(dir.path(), Some("nope")).unwrap_err());
    assert!(e.contains("nope"), "{e}");
}

// ---------- check ----------

#[test]
fn spec_007_an_uncovered_scenario_and_an_open_question_are_reported() {
    let dir = tree();
    delta(
        dir.path(),
        &format!("{DELTA}\n## ADDED Requirements\n\n### Requirement: Revoke\nThe system SHALL revoke.\n\n#### Scenario: AUTH-004 Revoke\n- **WHEN** a\n- **THEN** b\n"),
    );
    put(
        dir.path(),
        "changes/add-refresh/proposal.md",
        "## Why\n\nTokens expire.\n\n## Open\n\n- How long? [NEEDS CLARIFICATION: token lifetime?]\n",
    );
    put(
        dir.path(),
        "changes/add-refresh/tasks.md",
        &format!("{TASKS}- [ ] 1.4 Audit (AUTH-099)\n"),
    );
    let r = check(dir.path(), "add-refresh").unwrap();
    assert_eq!(r.uncovered, ["AUTH-004"]);
    let unknown: Vec<(String, String, usize)> = r
        .unknown
        .iter()
        .map(|u| (u.id.clone(), u.file.to_string_lossy().replace('\\', "/"), u.line))
        .collect();
    assert_eq!(unknown, [("AUTH-099".to_string(), "tasks.md".to_string(), 8)]);
    let clarify: Vec<(String, String, usize)> = r
        .clarify
        .iter()
        .map(|c| (c.question.clone(), c.file.to_string_lossy().replace('\\', "/"), c.line))
        .collect();
    assert_eq!(clarify, [("token lifetime?".to_string(), "proposal.md".to_string(), 7)]);
    assert!(!r.is_clean());
}

#[test]
fn spec_007_a_covered_change_is_clean() {
    let dir = tree();
    let r = check(dir.path(), "add-refresh").unwrap();
    assert!(r.is_clean(), "{r:?}");
}

#[test]
fn spec_007_questions_in_fenced_code_are_ignored() {
    let dir = tree();
    put(
        dir.path(),
        "changes/add-refresh/design.md",
        "## Context\n\n```md\n[NEEDS CLARIFICATION: example only]\n```\n",
    );
    assert!(check(dir.path(), "add-refresh").unwrap().is_clean());
}
