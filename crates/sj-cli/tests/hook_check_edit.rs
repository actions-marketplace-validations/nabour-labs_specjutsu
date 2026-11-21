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
