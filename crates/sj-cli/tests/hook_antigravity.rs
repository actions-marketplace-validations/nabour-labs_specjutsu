//! `sj hook antigravity pre-edit` and the `.agents/hooks.json` entry
//! (specjutsu/changes/more-adapters-2).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root).env_remove("CLAUDE_PROJECT_DIR");
    c
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

/// The JSON `sj hook antigravity pre-edit` prints for `input`, after checking it exits 0,
/// with its stderr.
fn answer(root: &Path, input: &str) -> (serde_json::Value, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", "antigravity", "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout:?}"));
    (v, String::from_utf8(out.stderr).unwrap())
}

const USER_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {"matcher": "run_command", "hooks": [{"type": "command", "command": "./audit.sh"}]}
    ]
  }
}
"#;

#[test]
fn adapt_030_antigravity_hook_blocks_source_edits_before_red() {
    let dir = project();
    let root = dir.path();
    put(root, ".agents/hooks.json", USER_HOOKS);

    sj(root)
        .args(["init", "--agents", "antigravity"])
        .assert()
        .success();

    // The user hook is unchanged, and ours is added beside it.
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(".agents/hooks.json")).unwrap())
            .unwrap();
    let entries = v["hooks"]["PreToolUse"].as_array().unwrap();
    let user: serde_json::Value = serde_json::from_str(USER_HOOKS).unwrap();
    assert_eq!(entries[0], user["hooks"]["PreToolUse"][0]);
    assert!(
        entries.iter().any(|e| e["matcher"]
            == "write_to_file|replace_file_content|multi_replace_file_content"
            && e["hooks"][0]["command"] == "sj hook antigravity pre-edit"),
        "{v}"
    );
    // It implies agents-md.
    assert!(std::fs::read_to_string(root.join("AGENTS.md"))
        .unwrap()
        .contains("<!-- specjutsu:begin -->"));

    let reason = "write a failing test for AUTH-005 first (sj test AUTH-005)";
    for key in ["TargetFile", "file_path", "path"] {
        let input = format!(
            r#"{{"toolCall":{{"name":"write_to_file","args":{{"{key}":"apps/api/src/auth.ts"}}}}}}"#
        );
        let (v, _) = answer(root, &input);
        assert_eq!(
            v,
            serde_json::json!({"decision": "deny", "reason": reason}),
            "{key}"
        );
    }
    let test_file = r#"{"toolCall":{"name":"write_to_file","args":{"TargetFile":"apps/api/src/auth.test.ts"}}}"#;
    assert_eq!(
        answer(root, test_file).0,
        serde_json::json!({"decision": "allow"})
    );

    // Unreadable input, or no path, allows with a warning.
    for bad in [
        "not json",
        r#"{"toolCall":{"name":"write_to_file","args":{}}}"#,
    ] {
        let (v, err) = answer(root, bad);
        assert_eq!(v, serde_json::json!({"decision": "allow"}), "{bad}");
        assert!(err.contains("specjutsu: hook error"), "{bad}: {err}");
    }

    sj(root)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("antigravity: hook\n"));

    // Removal restores the user's file byte for byte.
    sj(root)
        .args(["init", "--remove-agents", "antigravity"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(root.join(".agents/hooks.json")).unwrap(),
        USER_HOOKS
    );
}
