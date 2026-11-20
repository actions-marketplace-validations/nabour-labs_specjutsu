//! `sj hook codex pre-edit`: Codex's `PreToolUse` protocol for `apply_patch`, and fail-open
//! input handling for Codex and Cursor (openspec/changes/more-adapters).

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
    put(root, "apps/api/src/auth.test.ts", "test('x', () => {})\n");
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

fn hook(root: &Path, agent: &str, input: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", agent, "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .assert()
}

/// Codex's `PreToolUse` input for an `apply_patch` call with this patch text.
fn apply_patch(patch: &str) -> String {
    serde_json::json!({
        "session_id": "s",
        "hook_event_name": "PreToolUse",
        "tool_name": "apply_patch",
        "tool_use_id": "t",
        "tool_input": {"command": patch},
    })
    .to_string()
}

#[test]
fn adapt_021_codex_hook_denies_a_patch_that_touches_one_source_file() {
    let dir = project();
    let root = dir.path();

    let both = "*** Begin Patch\n\
                *** Update File: apps/api/src/auth.test.ts\n@@\n-test('x', () => {})\n+test('y', () => {})\n\
                *** Update File: apps/api/src/auth.ts\n@@\n-export const auth = 1\n+export const auth = 2\n\
                *** End Patch\n";
    let out = hook(root, "codex", &apply_patch(both)).code(0);
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let h = &v["hookSpecificOutput"];
    assert_eq!(h["hookEventName"], "PreToolUse", "{v}");
    assert_eq!(h["permissionDecision"], "deny", "{v}");
    let reason = h["permissionDecisionReason"].as_str().unwrap();
    assert!(reason.contains("apps/api/src/auth.ts"), "{reason}");
    assert!(!reason.contains("auth.test.ts"), "{reason}");
    assert!(
        reason.contains("write a failing test for AUTH-005 first (sj test AUTH-005)"),
        "{reason}"
    );

    let test_only = "*** Begin Patch\n\
                     *** Add File: apps/api/src/auth.test.ts\n+test('z', () => {})\n\
                     *** End Patch\n";
    hook(root, "codex", &apply_patch(test_only))
        .code(0)
        .stdout("");
}

#[test]
fn adapt_018_an_unreadable_hook_input_never_blocks() {
    let dir = project();
    let root = dir.path();

    hook(root, "codex", "not json {")
        .code(0)
        .stdout("")
        .stderr(predicate::str::contains("not JSON"));

    hook(root, "cursor", r#"{"tool_name":"Write","tool_input":{}}"#)
        .code(0)
        .stdout("{\"permission\":\"allow\"}\n")
        .stderr(predicate::str::contains("no file path"));
}
