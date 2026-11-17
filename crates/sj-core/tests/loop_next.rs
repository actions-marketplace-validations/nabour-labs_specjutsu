//! `sj next --parallel` batching (openspec/changes/agent-loop).

use std::path::Path;

use sj_core::agent_loop::next;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn loop_003_parallel_tasks_share_no_test_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    put(root, "src/lib.rs", "");
    let mut spec = String::from(
        "## ADDED Requirements\n\n### Requirement: Claims\nThe system SHALL check claims.\n\n",
    );
    for id in ["AUTH-007", "AUTH-008", "AUTH-009", "AUTH-010"] {
        spec.push_str(&format!(
            "#### Scenario: {id} Claim\n- **WHEN** a\n- **THEN** b\n\n"
        ));
    }
    put(root, "openspec/changes/claims/specs/auth/spec.md", &spec);
    put(
        root,
        "openspec/changes/claims/tasks.md",
        "- [ ] 3.1 [P] Scopes (AUTH-007)\n\
         - [ ] 3.2 [P] Audience (AUTH-008)\n\
         - [ ] 3.3 [P] Issuer (AUTH-009)\n\
         - [x] 3.4 [P] Done already (AUTH-010)\n\
         - [ ] 3.5 Not marked (AUTH-010)\n",
    );
    put(
        root,
        "tests/a.rs",
        "#[test]\nfn auth_007_scopes() {}\n\n#[test]\nfn auth_009_issuer() {}\n",
    );
    put(root, "tests/b.rs", "#[test]\nfn auth_008_audience() {}\n");

    let batches = next::parallel(root, &root.join("openspec"), "claims").unwrap();
    assert_eq!(
        batches,
        vec![
            vec![
                "3.1 [P] Scopes (AUTH-007)".to_string(),
                "3.2 [P] Audience (AUTH-008)".to_string()
            ],
            vec!["3.3 [P] Issuer (AUTH-009)".to_string()],
        ]
    );
}
