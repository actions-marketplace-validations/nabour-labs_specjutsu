//! GATE scenarios from openspec/changes/gate-open-change-base: base and head read an open
//! change's copy of a scenario the same way.

mod support;

use std::path::PathBuf;

use sj_core::gate::{run, Change, Options};
use support::Repo;

fn opts(repo: &Repo) -> Options {
    Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    }
}

const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_TEST: &str = "#[test]\nfn auth_001_valid_login() { assert_eq!(demo::add(2, 2), 4); }\n";

const OLD: &str = "## Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
                   #### Scenario: AUTH-001 Valid login\n- **WHEN** credentials match\n- **THEN** a session starts\n";

fn modified() -> String {
    "## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
     #### Scenario: AUTH-001 Valid login\n- **WHEN** credentials match\n- **THEN** a session starts and is remembered\n"
        .to_string()
}

#[test]
fn gate_045_a_merged_unarchived_change_is_not_modified_again() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", OLD);
    repo.write("openspec/changes/remember/proposal.md", "## Why\n\nTest.\n");
    repo.write("openspec/changes/remember/specs/auth/spec.md", &modified());
    repo.write("tests/auth.rs", ADD_TEST);
    repo.commit("feat(AUTH-001): merged without archive");
    repo.branch("next");
    repo.write("docs/notes.md", "Notes.\n");
    repo.commit("docs: notes");

    let r = run(&opts(&repo)).unwrap();
    assert_eq!(r.changes.get("AUTH-001"), None, "{:?}", r.changes);
    assert!(r.pass, "{:?}", r.failures);

    // A new change modifying a scenario in `specs/` is still modified.
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", OLD);
    repo.write("tests/auth.rs", ADD_TEST);
    repo.commit("feat(AUTH-001): login");
    repo.branch("remember");
    repo.write("openspec/changes/remember/proposal.md", "## Why\n\nTest.\n");
    repo.write("openspec/changes/remember/specs/auth/spec.md", &modified());
    repo.commit("spec(remember)");
    let r = run(&opts(&repo)).unwrap();
    assert_eq!(r.changes.get("AUTH-001"), Some(&Change::Modified));
}
