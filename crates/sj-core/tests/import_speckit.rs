//! `sj import speckit` conversion (specjutsu/changes/speckit-import).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sj_core::import::speckit::{self, Content, Plan};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/speckit")
}

fn plan() -> Plan {
    let root = fixture();
    speckit::plan(
        &root,
        &root.join("openspec"),
        &BTreeMap::new(),
        "2026-11-17",
    )
    .unwrap()
}

/// The generated text at `rel` (relative to the fixture root).
fn text(p: &Plan, rel: &str) -> String {
    let want = fixture().join(rel);
    for f in &p.features {
        for o in &f.outputs {
            if o.path == want {
                match &o.content {
                    Content::Text(t) => return t.clone(),
                    Content::Copy(src) => return std::fs::read_to_string(src).unwrap(),
                }
            }
        }
    }
    panic!("no output {rel}");
}

fn targets(p: &Plan, name: &str) -> Vec<String> {
    let f = p.features.iter().find(|f| f.name == name).unwrap();
    f.outputs
        .iter()
        .map(|o| {
            o.path
                .strip_prefix(fixture())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn imp_002_a_finished_feature_is_converted() {
    let p = plan();
    let f = p
        .features
        .iter()
        .find(|f| f.name == "chrome-ai-agent")
        .unwrap();
    assert!(f.finished);
    let mut got = targets(&p, "chrome-ai-agent");
    got.sort();
    let a = "openspec/changes/archive/2026-09-27-chrome-ai-agent";
    let mut want = vec![
        "openspec/specs/chrome-ai-agent/spec.md".to_string(),
        format!("{a}/proposal.md"),
        format!("{a}/design.md"),
        format!("{a}/research.md"),
        format!("{a}/data-model.md"),
        format!("{a}/quickstart.md"),
        format!("{a}/contracts/api.md"),
        format!("{a}/checklists/requirements.md"),
        format!("{a}/tasks.md"),
    ];
    want.sort();
    assert_eq!(got, want);
    // Everything but the spec and proposal is a byte copy; plan.md becomes design.md.
    let design = f
        .outputs
        .iter()
        .find(|o| o.path.ends_with("design.md"))
        .unwrap();
    assert_eq!(
        design.content,
        Content::Copy(fixture().join("specs/001-chrome-ai-agent/plan.md"))
    );
    let spec = text(&p, "openspec/specs/chrome-ai-agent/spec.md");
    assert!(spec.starts_with("# chrome-ai-agent Specification\n\n## Purpose\nA side-panel agent that takes plain-language instructions and works on the open tab.\n\n## Requirements\n"), "{spec}");
}

#[test]
fn imp_003_an_in_progress_feature_stays_in_progress() {
    let p = plan();
    let f = p.features.iter().find(|f| f.name == "history").unwrap();
    assert!(!f.finished);
    let mut got = targets(&p, "history");
    got.sort();
    assert_eq!(
        got,
        [
            "openspec/changes/history/design.md",
            "openspec/changes/history/proposal.md",
            "openspec/changes/history/specs/history/spec.md",
            "openspec/changes/history/tasks.md",
        ]
    );
    let delta = text(&p, "openspec/changes/history/specs/history/spec.md");
    assert!(delta.starts_with("## ADDED Requirements\n"), "{delta}");
    assert!(
        delta.contains("#### Scenario: HISTORY-001 US1.1 they open the sidebar\n"),
        "{delta}"
    );
    let tasks = text(&p, "openspec/changes/history/tasks.md");
    assert_eq!(tasks.matches("- [ ]").count(), 3);
    // No Created date: today's date would name an archive, but an open change has none.
    assert!(!got.iter().any(|t| t.contains("archive")));
}

#[test]
fn imp_004_an_acceptance_scenario_keeps_all_three_clauses() {
    let p = plan();
    let spec = text(&p, "openspec/specs/chrome-ai-agent/spec.md");
    let want = "### Requirement: Connect my Claude account with an API key\n\
A user opens the extension for the first time and pastes their own API key in Settings.\n\n\
**Why this priority**: Nothing else works without a valid key.\n\n\
**Independent Test**: Enter a valid and an invalid key and confirm only the first is accepted.\n\n\
The system SHALL support this story: Connect my Claude account with an API key.\n";
    assert!(spec.contains(want), "{spec}");
    let s2 = "#### Scenario: CHROMEAIAGENT-002 US1.2 they enter a valid key and save\n\
- **GIVEN** the user is in Settings\n\
- **WHEN** they enter a valid key and save\n\
- **THEN** the extension verifies it and shows a \"Connected\" status\n";
    assert!(spec.contains(s2), "{spec}");
    // A long When clause is cut at a word boundary within 60 characters.
    assert!(
        spec.contains("#### Scenario: CHROMEAIAGENT-004 US2.1 the user types an instruction that is rather long and keeps\n"),
        "{spec}"
    );
    // An item without When: the Given clause becomes the WHEN step and the name.
    let s5 = "#### Scenario: CHROMEAIAGENT-005 US2.2 Shift+Enter is pressed in the input\n\
- **WHEN** Shift+Enter is pressed in the input\n\
- **THEN** a new line is inserted instead of sending.\n";
    assert!(spec.contains(s5), "{spec}");
    let f = &p.features[0];
    assert_eq!((f.requirements, f.scenarios), (6, 5));
}

#[test]
fn imp_004_ids_continue_after_existing_ones() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy(&fixture(), root);
    let existing = root.join("openspec/specs/other/spec.md");
    std::fs::create_dir_all(existing.parent().unwrap()).unwrap();
    std::fs::write(
        &existing,
        "## Purpose\nX.\n\n## Requirements\n\n### Requirement: X\nThe system SHALL x.\n\n#### Scenario: CHROMEAIAGENT-041 Old\n- **WHEN** a\n- **THEN** b\n",
    )
    .unwrap();
    let mut prefixes = BTreeMap::new();
    prefixes.insert("history".to_string(), "HIST".to_string());
    let p = speckit::plan(root, &root.join("openspec"), &prefixes, "2026-11-17").unwrap();
    let spec = p.features[0]
        .outputs
        .iter()
        .find_map(|o| match &o.content {
            Content::Text(t) if o.path.ends_with("spec.md") => Some(t.clone()),
            _ => None,
        })
        .unwrap();
    assert!(
        spec.contains("Scenario: CHROMEAIAGENT-042 US1.1 "),
        "{spec}"
    );
    let delta = p.features[1]
        .outputs
        .iter()
        .find_map(|o| match &o.content {
            Content::Text(t) if o.path.ends_with("spec.md") => Some(t.clone()),
            _ => None,
        })
        .unwrap();
    assert!(delta.contains("Scenario: HIST-001 US1.1 "), "{delta}");
}

#[test]
fn imp_005_an_fr_without_a_scenario_is_flagged() {
    let p = plan();
    let f = &p.features[0];
    assert_eq!(
        f.gaps,
        ["FR-001", "FR-002", "FR-007"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    );
    let spec = text(&p, "openspec/specs/chrome-ai-agent/spec.md");
    assert!(
        spec.contains("### Requirement: FR-007 System MUST store keys encrypted at rest\nSystem MUST store keys encrypted at rest\n\n[NEEDS CLARIFICATION: add a scenario for FR-007]\n"),
        "{spec}"
    );
    // The first clause stops at `;`.
    assert!(
        spec.contains("### Requirement: FR-001 The extension MUST provide a Settings screen\n"),
        "{spec}"
    );
    // An acceptance item that names the FR covers it: no marker.
    assert!(
        spec.contains("### Requirement: FR-020a The agent MUST act only on the tab where the task started.\nThe agent MUST act only on the tab where the task started.\n"),
        "{spec}"
    );
    assert!(!spec.contains("scenario for FR-020a"), "{spec}");
}

#[test]
fn imp_006_clarifications_edge_cases_and_other_sections_go_to_the_proposal() {
    let p = plan();
    let proposal = text(
        &p,
        "openspec/changes/archive/2026-09-27-chrome-ai-agent/proposal.md",
    );
    assert!(proposal.starts_with("## Why\n\nA side-panel agent that takes plain-language instructions and works on the open tab.\n\nIt keeps a history of tasks.\n"), "{proposal}");
    for want in [
        "## Source\n\n# Feature Specification: Browser Agent\n\n**Feature Branch**: `001-chrome-ai-agent`\n",
        "## Clarifications\n\n### Session 2026-09-27\n\n- Q: Should the agent act on pages in v1? → A: Yes, with confirmation before high-impact actions.\n",
        "## Edge Cases\n\n- The key is revoked mid-conversation: the user sees a link to Settings.\n- The user is offline: sending is blocked.\n",
        "## Key Entities\n\n- **API Key Credential**",
        "## Success Criteria *(mandatory)*\n\n- **SC-001**",
        "## Assumptions\n\n- Users bring their own API key.\n",
        "## Functional Requirements\n\n**Settings**\n\n- **FR-001**",
    ] {
        assert!(proposal.contains(want), "{want}\n---\n{proposal}");
    }
    let principles = p.principles.as_ref().unwrap();
    assert_eq!(principles.path, fixture().join("openspec/principles.md"));
    assert_eq!(
        principles.content,
        Content::Copy(fixture().join(".specify/memory/constitution.md"))
    );
}

fn copy(from: &Path, to: &Path) {
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            std::fs::create_dir_all(&dest).unwrap();
            copy(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).unwrap();
        }
    }
}
