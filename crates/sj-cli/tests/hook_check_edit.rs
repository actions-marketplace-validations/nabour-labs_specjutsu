//! `sj hook check-edit <path>`: the pre-edit decision with no agent-specific JSON
//! (specjutsu/changes/more-adapters).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A guided vitest package whose next task names AUTH-005, with no red record.
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(
        root,
        "specjutsu.toml",
        "schema = 1\nmode = \"guided\"\n\n[[package]]\npath = \"apps/api\"\n\n\
         [[package.suite]]\nrunner = \"vitest\"\ntests = [\"**/*.test.ts\"]\n",
    );
    put(root, "apps/api/src/auth.ts", "export const auth = 1\n");
    put(root, "openspec/specs/.gitkeep", "");
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Tokens\nThe system SHALL refresh tokens.\n\n\
         #### Scenario: AUTH-005 Refresh\n- **WHEN** a token expires\n- **THEN** it is refreshed\n",
    );
    put(
        root,
        "openspec/changes/add-oauth-login/tasks.md",
        "- [ ] 2.1 Refresh tokens (AUTH-005)\n",
    );
    dir
}

fn check_edit(root: &Path, path: &str) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root)
        .args(["hook", "check-edit", path])
        .env_remove("CLAUDE_PROJECT_DIR");
    c
}

#[test]
fn adapt_019_generic_check_blocks_and_allows_by_exit_code() {
    let dir = project();
    let root = dir.path();

    check_edit(root, "apps/api/src/auth.ts")
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "write a failing test for AUTH-005 first (sj test AUTH-005)",
        ));

    check_edit(root, "apps/api/src/auth.test.ts")
        .assert()
        .code(0)
        .stdout("")
        .stderr("");
}

/// A guided vitest package whose next task names `ids`, with `scenarios` as the change's
/// spec body and no red record (specjutsu/changes/fix-hook-keep).
fn keep_project(scenarios: &str, ids: &str) -> tempfile::TempDir {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &format!(
            "## ADDED Requirements\n\n### Requirement: Tokens\nThe system SHALL refresh tokens.\n\n{scenarios}"
        ),
    );
    put(
        root,
        "openspec/changes/add-oauth-login/tasks.md",
        &format!("- [ ] 2.1 Refresh tokens ({ids})\n"),
    );
    dir
}

const KEEP_005: &str = "#### Scenario: AUTH-005 [keep] Refresh\n- **WHEN** a token expires\n- **THEN** it is refreshed\n\n";
const PLAIN_006: &str =
    "#### Scenario: AUTH-006 Revoke\n- **WHEN** a token is revoked\n- **THEN** it is refused\n";

#[test]
fn adapt_043_task_naming_only_a_keep_scenario_allows_source_edits() {
    let dir = keep_project(KEEP_005, "AUTH-005");
    check_edit(dir.path(), "apps/api/src/auth.ts")
        .assert()
        .code(0)
        .stdout("")
        .stderr("");

    let dir = keep_project(&format!("{KEEP_005}{PLAIN_006}"), "AUTH-005, AUTH-006");
    check_edit(dir.path(), "apps/api/src/auth.ts")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AUTH-005").not());
}

#[test]
fn adapt_044_non_keep_scenario_without_a_red_run_still_blocks() {
    let dir = keep_project(&format!("{KEEP_005}{PLAIN_006}"), "AUTH-005, AUTH-006");
    check_edit(dir.path(), "apps/api/src/auth.ts")
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "write a failing test for AUTH-006 first (sj test AUTH-006)",
        ));
}
