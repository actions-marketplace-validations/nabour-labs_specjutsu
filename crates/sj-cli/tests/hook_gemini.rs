//! `sj hook gemini pre-edit` and the `.gemini/settings.json` entries
//! (openspec/changes/more-adapters-2).

use std::path::Path;

use assert_cmd::Command;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
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

/// What `sj hook gemini pre-edit` prints for `input`, after checking it exits 0.
fn answer(root: &Path, input: &str) -> String {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["hook", "gemini", "pre-edit", "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn adapt_025_gemini_hook_blocks_source_edits_before_red() {
    let dir = project();
    let root = dir.path();

    let source = r#"{"tool_name":"write_file","tool_input":{"file_path":"apps/api/src/auth.ts"}}"#;
    assert_eq!(
        answer(root, source),
        "{\"decision\":\"deny\",\"reason\":\"write a failing test for AUTH-005 first (sj test AUTH-005)\"}\n"
    );
    let test_file =
        r#"{"tool_name":"write_file","tool_input":{"file_path":"apps/api/src/auth.test.ts"}}"#;
    assert_eq!(answer(root, test_file), "{\"decision\":\"allow\"}\n");
    assert_eq!(answer(root, "not json"), "{\"decision\":\"allow\"}\n");
}

#[test]
fn adapt_025_gemini_install_adds_the_hook_and_reads_agents_md() {
    let dir = project();
    let root = dir.path();
    let before =
        "{\n  \"theme\": \"GitHub\",\n  \"context\": {\n    \"fileName\": \"CONTEXT.md\"\n  }\n}\n";
    put(root, ".gemini/settings.json", before);

    sj(root)
        .args(["init", "--agents", "gemini"])
        .assert()
        .success();

    let v: serde_json::Value = serde_json::from_str(&read(root, ".gemini/settings.json")).unwrap();
    assert_eq!(v["theme"], "GitHub");
    assert_eq!(
        v["context"]["fileName"],
        serde_json::json!(["CONTEXT.md", "AGENTS.md"])
    );
    let before_tool = v["hooks"]["BeforeTool"].as_array().unwrap();
    assert!(
        before_tool
            .iter()
            .any(|e| e["matcher"] == "write_file|replace"
                && e["hooks"][0]["type"] == "command"
                && e["hooks"][0]["command"] == "sj hook gemini pre-edit"),
        "{v}"
    );
    assert!(read(root, "AGENTS.md").contains("<!-- specjutsu:begin -->"));

    // Installing again changes nothing.
    let once = read(root, ".gemini/settings.json");
    sj(root)
        .args(["init", "--agents", "gemini"])
        .assert()
        .success();
    assert_eq!(read(root, ".gemini/settings.json"), once);

    // No setting yet: Gemini's default GEMINI.md is kept beside AGENTS.md, and removal
    // restores the file.
    let fresh = project();
    let root = fresh.path();
    let theme_only = "{\n  \"theme\": \"GitHub\"\n}\n";
    put(root, ".gemini/settings.json", theme_only);
    sj(root)
        .args(["init", "--agents", "gemini"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(&read(root, ".gemini/settings.json")).unwrap();
    assert_eq!(
        v["context"]["fileName"],
        serde_json::json!(["GEMINI.md", "AGENTS.md"])
    );
    sj(root)
        .args(["init", "--remove-agents", "gemini"])
        .assert()
        .success();
    assert_eq!(read(root, ".gemini/settings.json"), theme_only);
    assert!(!root.join("AGENTS.md").exists());
}
