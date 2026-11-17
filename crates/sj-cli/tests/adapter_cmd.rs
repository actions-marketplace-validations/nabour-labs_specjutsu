//! `sj init --agents claude`, `sj update` and `sj init --remove-agents claude`
//! (openspec/changes/claude-adapter).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(root);
    c
}

/// The shape of ellesmere.ai's settings: a worktree guard before every edit, a
/// SessionStart hook and three deny rules, indented with two spaces.
const SETTINGS: &str = r#"{
  "permissions": {
    "deny": [
      "Bash(git push --force:*)",
      "Bash(rm -rf:*)",
      "Read(./.env)"
    ]
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "node scripts/hooks/require-worktree.mjs"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo hello"
          }
        ]
      }
    ]
  }
}
"#;

fn claude_md() -> String {
    (1..=40)
        .map(|i| format!("- Project rule {i}: keep it tidy."))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), ".claude/settings.json", SETTINGS);
    put(dir.path(), "CLAUDE.md", &claude_md());
    put(dir.path(), "openspec/specs/.gitkeep", "");
    dir
}

fn install(root: &Path) {
    sj(root, &["init", "--agents", "claude"]).assert().success();
}

#[test]
fn adapt_001_existing_hooks_and_permissions_survive() {
    let dir = project();
    install(dir.path());
    let text = read(dir.path(), ".claude/settings.json");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let before: serde_json::Value = serde_json::from_str(SETTINGS).unwrap();
    assert_eq!(v["permissions"], before["permissions"]);
    let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre[0], before["hooks"]["PreToolUse"][0]);
    assert_eq!(pre[1]["hooks"][0]["command"], "sj hook claude pre-edit");
    let start = v["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(start[0], before["hooks"]["SessionStart"][0]);
    assert_eq!(
        start[1]["hooks"][0]["command"],
        "sj hook claude session-start"
    );
    // Key order is the original's.
    assert!(
        text.find("\"permissions\"") < text.find("\"hooks\""),
        "{text}"
    );
    // Skills are installed, with nothing outside .claude/ but CLAUDE.md touched.
    for s in ["sj-apply", "sj-verify", "sj-finish"] {
        let skill = read(dir.path(), &format!(".claude/skills/{s}/SKILL.md"));
        assert!(skill.starts_with("---\nname: "), "{s}");
        assert!(skill.contains("<!-- specjutsu:generated "), "{s}");
    }
    // An existing specjutsu.toml is not an error with --agents, and one is written when missing.
    assert!(dir.path().join("specjutsu.toml").is_file());
    sj(dir.path(), &["init", "--agents", "claude"])
        .assert()
        .success();
}

#[test]
fn adapt_002_claude_md_content_outside_the_block_is_kept() {
    let dir = project();
    install(dir.path());
    let text = read(dir.path(), "CLAUDE.md");
    assert!(text.starts_with(&claude_md()), "{text}");
    let block = &text[claude_md().len()..];
    assert!(block.starts_with("\n<!-- specjutsu:begin -->\n"), "{block}");
    assert!(block.ends_with("<!-- specjutsu:end -->\n"), "{block}");
    assert!(block.len() <= 1200 + 60, "{} bytes", block.len());
    assert!(block.contains("sj next"), "{block}");
    // Generated content is machine-independent.
    assert!(!text.contains(&dir.path().to_string_lossy().to_string()));
    assert!(
        !block.contains("2026") && !block.contains("0.2."),
        "no dates or versions: {block}"
    );
}

#[test]
fn adapt_003_update_is_idempotent() {
    let dir = project();
    install(dir.path());
    sj(dir.path(), &["update"])
        .assert()
        .success()
        .stdout("adapters up to date\n");
    sj(dir.path(), &["update"])
        .assert()
        .success()
        .stdout("adapters up to date\n");
    // A removed skill comes back.
    std::fs::remove_dir_all(dir.path().join(".claude/skills/sj-debug")).unwrap();
    sj(dir.path(), &["update"])
        .assert()
        .success()
        .stdout("created .claude/skills/sj-debug/SKILL.md\n");
}

#[test]
fn adapt_004_a_hand_edited_generated_skill_is_reported_not_overwritten() {
    let dir = project();
    install(dir.path());
    let rel = ".claude/skills/sj-apply/SKILL.md";
    let edited = read(dir.path(), rel).replace("test first", "test first, always");
    put(dir.path(), rel, &edited);
    sj(dir.path(), &["update"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "modified .claude/skills/sj-apply/SKILL.md (kept; sj update --force replaces it)",
        ));
    assert_eq!(read(dir.path(), rel), edited);
    sj(dir.path(), &["update", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "updated .claude/skills/sj-apply/SKILL.md",
        ));
    assert!(!read(dir.path(), rel).contains("test first, always"));
}

#[test]
fn adapt_010_install_then_remove_restores_the_files() {
    let dir = project();
    install(dir.path());
    sj(dir.path(), &["init", "--remove-agents", "claude"])
        .assert()
        .success();
    assert_eq!(read(dir.path(), ".claude/settings.json"), SETTINGS);
    assert_eq!(read(dir.path(), "CLAUDE.md"), claude_md());
    assert!(!dir.path().join(".claude/skills").exists());
}

#[test]
fn adapt_010_files_the_adapter_created_are_deleted_on_remove() {
    let dir = tempfile::tempdir().unwrap();
    install(dir.path());
    assert!(dir.path().join("CLAUDE.md").is_file());
    sj(dir.path(), &["init", "--remove-agents", "claude"])
        .assert()
        .success();
    assert!(!dir.path().join("CLAUDE.md").exists());
    assert!(!dir.path().join(".claude/settings.json").exists());
}

#[test]
fn adapt_001_unknown_agents_are_refused() {
    let dir = project();
    sj(dir.path(), &["init", "--agents", "emacs"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("claude"));
    assert_eq!(read(dir.path(), ".claude/settings.json"), SETTINGS);
}
