//! SPEC-008..011 from openspec/changes/spec-authoring: `sj spec archive`.
//!
//! The golden files under `fixtures/authoring/<case>/expected/` were written by
//! `openspec archive` 1.14.0 from `before/` (see `fixtures/authoring/regen.sh`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sj_core::authoring::archive::{archive, merge, Options, Outcome};

const DATE: &str = "2026-11-16";

fn fixture(case: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/authoring")
        .join(case)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).unwrap();
        }
    }
}

/// Every file under `dir`, keyed by its forward-slash path relative to `dir`.
fn files(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                out.insert(rel, std::fs::read_to_string(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn opts() -> Options {
    Options {
        force: false,
        date: DATE.into(),
    }
}

/// Archive the case's only change in a copy of `before/`; return the temp dir.
fn run_case(case: &str, change: &str) -> (tempfile::TempDir, Outcome) {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture(case).join("before"), dir.path());
    let outcome = archive(&dir.path().join("openspec"), change, &opts()).unwrap();
    (dir, outcome)
}

fn assert_matches_golden(case: &str, dir: &Path) {
    let got = files(&dir.join("openspec/specs"));
    let want = files(&fixture(case).join("expected/specs"));
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>()
    );
    for (rel, text) in &want {
        assert_eq!(&got[rel], text, "{case}: specs/{rel} differs from openspec's output");
    }
}

#[test]
fn spec_008_archive_merges_all_four_delta_kinds() {
    let (dir, outcome) = run_case("four-kinds", "add-refresh");
    let Outcome::Archived { specs, dest } = outcome else {
        panic!("refused: {outcome:?}");
    };
    assert_matches_golden("four-kinds", dir.path());
    let root = dir.path().join("openspec");
    assert_eq!(dest, root.join(format!("changes/archive/{DATE}-add-refresh")));
    assert!(dest.join("proposal.md").is_file());
    assert!(!root.join("changes/add-refresh").exists());
    assert_eq!(specs.len(), 1);
    let s = &specs[0];
    assert!(!s.created);
    assert_eq!(
        (s.added, s.modified, s.removed, s.renamed),
        (1, 1, 1, 1),
        "{s:?}"
    );
}

#[test]
fn spec_010_a_new_capability_gets_its_purpose() {
    let (dir, outcome) = run_case("new-capability", "add-billing");
    let Outcome::Archived { specs, .. } = outcome else {
        panic!("refused: {outcome:?}");
    };
    assert_matches_golden("new-capability", dir.path());
    assert!(specs[0].created);
    assert_eq!(specs[0].added, 2);
}

#[test]
fn spec_011_an_openspec_change_archives_like_openspec() {
    let (dir, outcome) = run_case("layout", "chat-v2");
    assert!(matches!(outcome, Outcome::Archived { .. }), "{outcome:?}");
    assert_matches_golden("layout", dir.path());
}

#[test]
fn spec_011_metadata_moves_with_the_change() {
    let (dir, _) = run_case("four-kinds", "add-refresh");
    let archived = dir
        .path()
        .join(format!("openspec/changes/archive/{DATE}-add-refresh"));
    assert!(archived.join(".openspec.yaml").is_file());
}

#[test]
fn spec_008_a_spec_already_in_openspec_layout_keeps_other_bytes() {
    // The golden output is canonical: merging an empty-effect delta into it changes nothing.
    let golden = std::fs::read_to_string(fixture("four-kinds").join("expected/specs/auth/spec.md")).unwrap();
    let delta = "## ADDED Requirements\n\n### Requirement: Session lifetime\nThe system SHALL expire sessions after 30 days.\n\n\
                 #### Scenario: AUTH-004 An old session expires\n- **WHEN** a session is 31 days old\n- **THEN** it is rejected\n";
    let merged = merge(Some(&golden), delta, "auth", "noop").unwrap();
    assert_eq!(merged.text, golden);
    assert_eq!(merged.added, 0);
}

#[test]
fn spec_008_crlf_input_becomes_lf() {
    let golden = std::fs::read_to_string(fixture("four-kinds").join("expected/specs/auth/spec.md")).unwrap();
    let delta = "## REMOVED Requirements\r\n\r\n### Requirement: Refresh\r\n**Reason**: x\r\n**Migration**: y\r\n";
    let merged = merge(Some(&golden.replace('\n', "\r\n")), delta, "auth", "c").unwrap();
    assert!(!merged.text.contains('\r'));
    assert!(!merged.text.contains("Requirement: Refresh"));
    assert_eq!(merged.removed, 1);
}

// ---------- merge errors ----------

const AUTH: &str = "# auth Specification\n\n## Purpose\nSign in.\n\n## Requirements\n\n\
### Requirement: Login\nThe system SHALL log in.\n\n#### Scenario: AUTH-001 Valid login\n- **WHEN** a\n- **THEN** b\n\n\
#### Scenario: AUTH-002 Wrong password\n- **WHEN** c\n- **THEN** d\n\n\
### Requirement: Logout\nThe system SHALL log out.\n\n#### Scenario: AUTH-003 Logout\n- **WHEN** e\n- **THEN** f\n";

fn merge_err(delta: &str) -> String {
    format!("{:#}", merge(Some(AUTH), delta, "auth", "c").unwrap_err())
}

#[test]
fn spec_008_modified_must_exist() {
    let e = merge_err("## MODIFIED Requirements\n\n### Requirement: Login flow\nThe system SHALL x.\n\n#### Scenario: AUTH-009 X\n- **WHEN** a\n- **THEN** b\n");
    assert!(e.contains("Login flow") && e.contains("not found"), "{e}");
}

#[test]
fn spec_008_modified_may_not_drop_a_scenario() {
    let e = merge_err("## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n#### Scenario: AUTH-001 Valid login\n- **WHEN** a\n- **THEN** b2\n");
    assert!(e.contains("AUTH-002 Wrong password"), "{e}");
}

#[test]
fn spec_008_added_may_not_replace_a_different_block() {
    let e = merge_err("## ADDED Requirements\n\n### Requirement: Logout\nThe system SHALL log out twice.\n\n#### Scenario: AUTH-003 Logout\n- **WHEN** e\n- **THEN** f\n");
    assert!(e.contains("Logout") && e.contains("already exists"), "{e}");
}

#[test]
fn spec_008_rename_target_may_not_exist() {
    let e = merge_err("## RENAMED Requirements\n\n- FROM: `### Requirement: Login`\n- TO: `### Requirement: Logout`\n");
    assert!(e.contains("Logout") && e.contains("already exists"), "{e}");
}

#[test]
fn spec_008_unpaired_rename_is_an_error() {
    let e = merge_err("## RENAMED Requirements\n\n- FROM: `### Requirement: Login`\n");
    assert!(e.contains("RENAMED") && e.contains("Login"), "{e}");
}

#[test]
fn spec_008_a_name_in_two_sections_is_an_error() {
    let e = merge_err(
        "## MODIFIED Requirements\n\n### Requirement: Logout\nThe system SHALL log out.\n\n#### Scenario: AUTH-003 Logout\n- **WHEN** e\n- **THEN** g\n\n\
         ## REMOVED Requirements\n\n### Requirement: Logout\n**Reason**: r\n**Migration**: m\n",
    );
    assert!(e.contains("MODIFIED") && e.contains("REMOVED"), "{e}");
}

#[test]
fn spec_010_a_new_capability_may_not_modify() {
    let e = format!(
        "{:#}",
        merge(None, "## MODIFIED Requirements\n\n### Requirement: X\nThe system SHALL x.\n", "billing", "c").unwrap_err()
    );
    assert!(e.contains("billing"), "{e}");
}

#[test]
fn spec_010_without_a_purpose_the_placeholder_is_written() {
    let merged = merge(
        None,
        "## ADDED Requirements\n\n### Requirement: X\nThe system SHALL x.\n\n#### Scenario: B-001 X\n- **WHEN** a\n- **THEN** b\n",
        "billing",
        "add-billing",
    )
    .unwrap();
    assert!(
        merged.text.starts_with(
            "# billing Specification\n\n## Purpose\nTBD - created by archiving change add-billing. Update Purpose after archive.\n\n## Requirements\n\n### Requirement: X\n"
        ),
        "{}",
        merged.text
    );
}

// ---------- refusals ----------

#[test]
fn spec_009_archive_refuses_open_tasks() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("four-kinds").join("before"), dir.path());
    let root = dir.path().join("openspec");
    std::fs::write(
        root.join("changes/add-refresh/tasks.md"),
        "# Tasks\n\n- [x] 1.1 Refresh (AUTH-005)\n- [ ] 1.2 Sign out (AUTH-001)\n",
    )
    .unwrap();
    let before = files(dir.path());
    let outcome = archive(&root, "add-refresh", &opts()).unwrap();
    let Outcome::Refused(reasons) = outcome else {
        panic!("archived despite an open task");
    };
    assert!(
        reasons.iter().any(|r| r.message == "open task 1.2 Sign out (AUTH-001)"
            && r.file.ends_with("changes/add-refresh/tasks.md")
            && r.line == 4),
        "{reasons:?}"
    );
    assert_eq!(files(dir.path()), before, "nothing may change");

    let forced = archive(
        &root,
        "add-refresh",
        &Options {
            force: true,
            date: DATE.into(),
        },
    )
    .unwrap();
    assert!(matches!(forced, Outcome::Archived { .. }), "{forced:?}");
}

#[test]
fn spec_009_archive_refuses_an_invalid_change() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("four-kinds").join("before"), dir.path());
    let root = dir.path().join("openspec");
    let delta = root.join("changes/add-refresh/specs/auth/spec.md");
    let text = std::fs::read_to_string(&delta).unwrap();
    std::fs::write(&delta, text.replace("- **THEN** a new access token is returned\n", "")).unwrap();
    let before = files(dir.path());
    let Outcome::Refused(reasons) = archive(&root, "add-refresh", &opts()).unwrap() else {
        panic!("archived an invalid change");
    };
    assert!(
        reasons.iter().any(|r| r.message == "AUTH-005 has no THEN step"),
        "{reasons:?}"
    );
    assert_eq!(files(dir.path()), before);
}

#[test]
fn spec_008_a_merge_error_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("four-kinds").join("before"), dir.path());
    let root = dir.path().join("openspec");
    // A second delta whose MODIFIED target does not exist; forced past validation.
    std::fs::create_dir_all(root.join("changes/add-refresh/specs/chat")).unwrap();
    std::fs::create_dir_all(root.join("specs/chat")).unwrap();
    std::fs::write(
        root.join("specs/chat/spec.md"),
        "# chat Specification\n\n## Purpose\nChat.\n\n## Requirements\n\n### Requirement: Send\nThe system SHALL send.\n\n#### Scenario: CHAT-001 Send\n- **WHEN** a\n- **THEN** b\n",
    )
    .unwrap();
    std::fs::write(
        root.join("changes/add-refresh/specs/chat/spec.md"),
        "## MODIFIED Requirements\n\n### Requirement: Receive\nThe system SHALL receive.\n\n#### Scenario: CHAT-002 Receive\n- **WHEN** a\n- **THEN** b\n",
    )
    .unwrap();
    let before = files(dir.path());
    let forced = Options {
        force: true,
        date: DATE.into(),
    };
    assert!(archive(&root, "add-refresh", &forced).is_err());
    assert_eq!(files(dir.path()), before, "auth must not be written when chat fails");
}

#[test]
fn spec_008_an_existing_archive_destination_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("four-kinds").join("before"), dir.path());
    let root = dir.path().join("openspec");
    std::fs::create_dir_all(root.join(format!("changes/archive/{DATE}-add-refresh"))).unwrap();
    let before = files(dir.path());
    let e = format!("{:#}", archive(&root, "add-refresh", &opts()).unwrap_err());
    assert!(e.contains("already exists"), "{e}");
    assert_eq!(files(dir.path()), before);
}
