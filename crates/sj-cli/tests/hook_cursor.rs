//! `sj hook cursor pre-edit`: Cursor's `preToolUse` protocol (openspec/changes/more-adapters).

use std::path::Path;

use assert_cmd::Command;

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

/// The JSON `sj hook cursor pre-edit` prints for `input`, after checking it exits 0.
fn answer(root: &Path, input: &str) -> serde_json::Value {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", "cursor", "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout:?}"))
}

#[test]
fn adapt_016_cursor_hook_blocks_source_edits_before_red() {
    let dir = project();
    let root = dir.path();
    let reason = "write a failing test for AUTH-005 first (sj test AUTH-005)";

    for key in ["file_path", "path", "target_file"] {
        let input =
            format!(r#"{{"tool_name":"Write","tool_input":{{"{key}":"apps/api/src/auth.ts"}}}}"#);
        let v = answer(root, &input);
        assert_eq!(v["permission"], "deny", "{key}: {v}");
        assert_eq!(v["agent_message"], reason, "{key}: {v}");
        assert_eq!(v["user_message"], reason, "{key}: {v}");
    }

    let test_file =
        r#"{"tool_name":"Write","tool_input":{"file_path":"apps/api/src/auth.test.ts"}}"#;
    assert_eq!(
        answer(root, test_file),
        serde_json::json!({"permission": "allow"})
    );
}
