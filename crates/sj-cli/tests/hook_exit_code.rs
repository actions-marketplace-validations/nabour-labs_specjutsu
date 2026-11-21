//! `sj hook windsurf pre-edit` and `sj hook kiro pre-edit`: the exit-code protocol
//! (openspec/changes/more-adapters-3).

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

/// The exit code, stdout and stderr of `sj hook <agent> pre-edit` for `input`.
fn run(root: &Path, agent: &str, input: &str) -> (Option<i32>, String, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", agent, "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

const REASON: &str = "write a failing test for AUTH-005 first (sj test AUTH-005)\n";

fn windsurf(path: &str) -> String {
    format!(r#"{{"agent_action_name":"pre_write_code","tool_info":{{"file_path":"{path}"}}}}"#)
}

fn kiro(key: &str, path: &str) -> String {
    format!(
        r#"{{"hook_event_name":"PreToolUse","tool_name":"fs_write","tool_input":{{"{key}":"{path}"}}}}"#
    )
}

#[test]
fn adapt_028_windsurf_hook_blocks_with_exit_code_2() {
    let dir = project();
    let root = dir.path();
    assert_eq!(
        run(root, "windsurf", &windsurf("apps/api/src/auth.ts")),
        (Some(2), String::new(), REASON.to_string())
    );
    assert_eq!(
        run(root, "windsurf", &windsurf("apps/api/src/auth.test.ts")).0,
        Some(0)
    );

    // The entry goes in .windsurf/hooks.json under pre_write_code.
    Command::cargo_bin("sj")
        .unwrap()
        .current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .args(["init", "--agents", "windsurf"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".windsurf/hooks.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        v["hooks"]["pre_write_code"][0]["command"],
        "sj hook windsurf pre-edit"
    );
}

#[test]
fn adapt_029_kiro_hook_blocks_with_exit_code_2() {
    let dir = project();
    let root = dir.path();
    for key in ["path", "file_path"] {
        assert_eq!(
            run(root, "kiro", &kiro(key, "apps/api/src/auth.ts")),
            (Some(2), String::new(), REASON.to_string()),
            "{key}"
        );
    }
    assert_eq!(
        run(root, "kiro", &kiro("path", "apps/api/src/auth.test.ts")).0,
        Some(0)
    );

    // The hook file names Kiro's `write` category.
    Command::cargo_bin("sj")
        .unwrap()
        .current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .args(["init", "--agents", "kiro"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".kiro/hooks/specjutsu.json")).unwrap(),
    )
    .unwrap();
    let entry = &v["hooks"]["PreToolUse"][0];
    assert_eq!(entry["matcher"], "write");
    assert_eq!(entry["command"], "sj hook kiro pre-edit");
}

#[test]
fn adapt_031_an_unreadable_exit_code_hook_input_never_blocks() {
    let dir = project();
    let root = dir.path();
    for agent in ["windsurf", "kiro"] {
        for (bad, problem) in [
            ("not json", "not JSON"),
            (r#"{"tool_info":{},"tool_input":{}}"#, "no file path"),
        ] {
            let (code, out, err) = run(root, agent, bad);
            assert_eq!(code, Some(0), "{agent} {bad}");
            assert_eq!(out, "", "{agent} {bad}");
            assert!(err.contains("specjutsu: hook error"), "{agent} {bad}: {err}");
            assert!(err.contains(problem), "{agent} {bad}: {err}");
        }
    }
}
