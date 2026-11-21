//! `sj hook copilot pre-edit` and `.github/hooks/specjutsu.json`
//! (openspec/changes/more-adapters-3).

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

/// What `sj hook copilot pre-edit` prints for `input`, after checking it exits 0, with
/// its stderr.
fn answer(root: &Path, input: &str) -> (String, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", "copilot", "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn input(tool: &str, key: &str, path: &str) -> String {
    format!(
        r#"{{"hook_event_name":"PreToolUse","tool_name":"{tool}","tool_input":{{"{key}":"{path}"}}}}"#
    )
}

#[test]
fn adapt_026_copilot_hook_denies_a_source_edit_before_red() {
    let dir = project();
    let root = dir.path();
    let reason = "write a failing test for AUTH-005 first (sj test AUTH-005)";
    let deny =
        serde_json::json!({"permissionDecision": "deny", "permissionDecisionReason": reason});

    for (tool, key) in [("Write", "file_path"), ("edit", "path")] {
        let (out, _) = answer(root, &input(tool, key, "apps/api/src/auth.ts"));
        let v: serde_json::Value =
            serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out:?}"));
        assert_eq!(v, deny, "{tool}");
    }
    let (out, _) = answer(
        root,
        &input("Write", "file_path", "apps/api/src/auth.test.ts"),
    );
    assert_eq!(out, "");

    // An apply_patch patch is checked file by file.
    let patch = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "apply_patch",
        "tool_input": {"command": "*** Begin Patch\n*** Add File: apps/api/src/auth.test.ts\n+x\n*** Update File: apps/api/src/auth.ts\n+y\n*** End Patch\n"},
    });
    let (out, _) = answer(root, &patch.to_string());
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["permissionDecision"], "deny");
    assert!(v["permissionDecisionReason"]
        .as_str()
        .unwrap()
        .contains("apps/api/src/auth.ts"));

    // Unreadable input, or no path, prints nothing and exits 0 with a warning.
    for bad in ["not json", r#"{"tool_name":"Write","tool_input":{}}"#] {
        let (out, err) = answer(root, bad);
        assert_eq!(out, "", "{bad}");
        assert!(err.contains("specjutsu: hook error"), "{bad}: {err}");
    }
}

#[test]
fn adapt_027_copilot_install_warns_about_the_default_branch() {
    let dir = project();
    let root = dir.path();
    std::process::Command::new("git")
        .args(["init", "-q", "-b", "feature/x"])
        .current_dir(root)
        .status()
        .unwrap();

    sj(root)
        .args(["init", "--agents", "copilot"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "copilot: hooks take effect for the cloud agent once .github/hooks/ is on the default branch\n",
        ));

    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".github/hooks/specjutsu.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["version"], 1);
    assert_eq!(
        v["hooks"]["PreToolUse"][0],
        serde_json::json!({
            "type": "command",
            "bash": "sj hook copilot pre-edit",
            "matcher": "^(Write|Edit|create|edit)$",
        })
    );
    assert!(std::fs::read_to_string(root.join("AGENTS.md"))
        .unwrap()
        .contains("<!-- specjutsu:begin -->"));

    sj(root)
        .args(["init", "--remove-agents", "copilot"])
        .assert()
        .success();
    assert!(!root.join(".github").exists());
}
