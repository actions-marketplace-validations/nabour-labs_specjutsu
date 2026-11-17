//! IDS scenarios from openspec/changes/scenario-ids: planning and writing scenario IDs.

use std::collections::BTreeMap;
use std::path::Path;

use sj_core::ids::{plan, write};

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

const USER_AUTH: &str = "## Requirements\n\n### Requirement: Registration\nThe system SHALL register users.\n\n\
#### Scenario: Valid registration\n- **WHEN** a user registers\n- **THEN** an account exists\n\n\
#### Scenario: Password too short\n- **WHEN** the password has 3 characters\n- **THEN** it is rejected\n";

fn none() -> BTreeMap<String, String> {
    BTreeMap::new()
}

/// (id, name, line) of every planned assignment.
fn planned(spec_root: &Path, prefixes: &BTreeMap<String, String>) -> Vec<(String, String, usize)> {
    plan(spec_root, prefixes)
        .unwrap()
        .into_iter()
        .map(|a| (a.id, a.name, a.line))
        .collect()
}

#[test]
fn ids_001_plan_proposes_ids_in_file_order() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/user-auth/spec.md", USER_AUTH);
    assert_eq!(
        planned(dir.path(), &none()),
        [
            ("USERAUTH-001".into(), "Valid registration".into(), 6),
            ("USERAUTH-002".into(), "Password too short".into(), 10),
        ]
    );
    // Planning writes nothing.
    assert_eq!(read(dir.path(), "specs/user-auth/spec.md"), USER_AUTH);
}

#[test]
fn ids_001_scenarios_with_ids_are_left_out() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/user-auth/spec.md",
        &USER_AUTH.replace("Scenario: Valid", "Scenario: USERAUTH-004 Valid"),
    );
    assert_eq!(
        planned(dir.path(), &none()),
        [("USERAUTH-005".into(), "Password too short".into(), 10)]
    );
}

#[test]
fn ids_002_write_changes_only_the_headings_crlf() {
    let dir = tempfile::tempdir().unwrap();
    let crlf = USER_AUTH.replace('\n', "\r\n") + "trailing  \r\n";
    put(dir.path(), "specs/user-auth/spec.md", &crlf);
    let p = plan(dir.path(), &none()).unwrap();
    assert_eq!(write(&p).unwrap(), 1);
    let want = crlf
        .replace("#### Scenario: Valid", "#### Scenario: USERAUTH-001 Valid")
        .replace(
            "#### Scenario: Password",
            "#### Scenario: USERAUTH-002 Password",
        );
    assert_eq!(read(dir.path(), "specs/user-auth/spec.md"), want);
}

#[test]
fn ids_002_headings_in_fenced_code_are_not_touched() {
    let dir = tempfile::tempdir().unwrap();
    let text = "### Requirement: R\n\n```md\n#### Scenario: Example only\n```\n\n#### Scenario: Real one\n- **THEN** ok\n";
    put(dir.path(), "specs/docs/spec.md", text);
    write(&plan(dir.path(), &none()).unwrap()).unwrap();
    assert_eq!(
        read(dir.path(), "specs/docs/spec.md"),
        text.replace("Scenario: Real", "Scenario: DOCS-001 Real")
    );
}

#[test]
fn ids_003_second_plan_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/user-auth/spec.md", USER_AUTH);
    write(&plan(dir.path(), &none()).unwrap()).unwrap();
    let after = read(dir.path(), "specs/user-auth/spec.md");
    let p = plan(dir.path(), &none()).unwrap();
    assert!(p.is_empty(), "{p:?}");
    assert_eq!(write(&p).unwrap(), 0);
    assert_eq!(read(dir.path(), "specs/user-auth/spec.md"), after);
}

#[test]
fn ids_004_configured_prefix_overrides_the_folder_name() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/user-auth/spec.md", USER_AUTH);
    let prefixes = BTreeMap::from([("user-auth".to_string(), "AUTH".to_string())]);
    let ids: Vec<String> = planned(dir.path(), &prefixes)
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();
    assert_eq!(ids, ["AUTH-001", "AUTH-002"]);
}

#[test]
fn ids_004_derived_prefix_keeps_only_letters() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/oauth2-login/spec.md", USER_AUTH);
    assert_eq!(planned(dir.path(), &none())[0].0, "OAUTHLOGIN-001");
}

#[test]
fn ids_005_colliding_prefixes_are_an_error_naming_both() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/chat/spec.md", USER_AUTH);
    put(dir.path(), "specs/c-hat/spec.md", USER_AUTH);
    let err = format!("{:#}", plan(dir.path(), &none()).unwrap_err());
    assert!(err.contains("`c-hat`") && err.contains("`chat`"), "{err}");
    assert!(err.contains("CHAT"), "{err}");
}

#[test]
fn ids_005_a_mapping_resolves_the_collision() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/chat/spec.md", USER_AUTH);
    put(dir.path(), "specs/c-hat/spec.md", USER_AUTH);
    let prefixes = BTreeMap::from([("c-hat".to_string(), "HAT".to_string())]);
    let ids: Vec<String> = planned(dir.path(), &prefixes)
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();
    assert_eq!(ids, ["HAT-001", "HAT-002", "CHAT-001", "CHAT-002"]);
}

#[test]
fn ids_005_fully_identified_capabilities_never_collide() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/c-hat/spec.md",
        "### Requirement: R\n\n#### Scenario: HAT-001 Old\n- **THEN** ok\n",
    );
    put(dir.path(), "specs/chat/spec.md", USER_AUTH);
    assert_eq!(planned(dir.path(), &none()).len(), 2);
}

#[test]
fn ids_005_a_capability_without_letters_needs_a_mapping() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specs/2026/spec.md", USER_AUTH);
    let err = format!("{:#}", plan(dir.path(), &none()).unwrap_err());
    assert!(
        err.contains("`2026`") && err.contains("[ids.prefixes]"),
        "{err}"
    );
}

#[test]
fn ids_006_numbers_continue_after_archived_ids() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/auth/spec.md",
        "### Requirement: Login\n\n#### Scenario: AUTH-001 One\n- **THEN** a\n\n\
         #### Scenario: AUTH-002 Two\n- **THEN** b\n\n#### Scenario: New one\n- **THEN** c\n",
    );
    put(
        dir.path(),
        "changes/archive/2026-01-01-old/specs/auth/spec.md",
        "## REMOVED Requirements\n\n### Requirement: Gone\n\n#### Scenario: AUTH-007 Deleted\n- **THEN** x\n",
    );
    assert_eq!(planned(dir.path(), &none())[0].0, "AUTH-008");
}

#[test]
fn ids_006_numbers_wider_than_three_digits_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/auth/spec.md",
        "### Requirement: Login\n\n#### Scenario: AUTH-0999 Old\n- **THEN** a\n\n#### Scenario: New\n- **THEN** c\n",
    );
    assert_eq!(planned(dir.path(), &none())[0].0, "AUTH-1000");
}

#[test]
fn ids_007_a_modified_copy_gets_the_originals_id() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/auth/spec.md",
        "## Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
         #### Scenario: Valid login\n- **WHEN** credentials match\n- **THEN** a session starts\n\n\
         #### Scenario: Wrong password\n- **THEN** it is rejected\n",
    );
    put(
        dir.path(),
        "changes/sessions/specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
         #### Scenario: Valid login\n- **WHEN** credentials match\n- **THEN** a session starts for 30 days\n\n\
         ## ADDED Requirements\n\n### Requirement: Logout\nThe system SHALL log out.\n\n\
         #### Scenario: Valid login\n- **THEN** not the same requirement\n",
    );
    let p = plan(dir.path(), &none()).unwrap();
    let got: Vec<(String, String)> = p
        .iter()
        .map(|a| {
            let rel = a.file.strip_prefix(dir.path()).unwrap();
            (a.id.clone(), rel.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    assert_eq!(
        got,
        [
            ("AUTH-001".into(), "specs/auth/spec.md".into()),
            ("AUTH-002".into(), "specs/auth/spec.md".into()),
            (
                "AUTH-001".into(),
                "changes/sessions/specs/auth/spec.md".into()
            ),
            (
                "AUTH-003".into(),
                "changes/sessions/specs/auth/spec.md".into()
            ),
        ]
    );
}

#[test]
fn ids_007_a_copy_of_an_identified_scenario_takes_its_id() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specs/auth/spec.md",
        "### Requirement: Login\n\n#### Scenario: AUTH-004 Valid login\n- **THEN** a\n",
    );
    put(
        dir.path(),
        "changes/sessions/specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Login\n\n#### Scenario: Valid login\n- **THEN** b\n",
    );
    assert_eq!(planned(dir.path(), &none())[0].0, "AUTH-004");
}

#[test]
fn ids_002_archived_changes_are_never_written() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "changes/archive/2026-01-01-old/specs/auth/spec.md",
        USER_AUTH,
    );
    assert!(plan(dir.path(), &none()).unwrap().is_empty());
}
