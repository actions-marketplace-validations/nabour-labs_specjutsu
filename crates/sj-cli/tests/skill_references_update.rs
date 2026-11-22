//! Updating and removing reference files next to a skill's `SKILL.md`
//! (specjutsu/changes/deep-skills).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env("HOME", root.join("home"))
        .args(args);
    c
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("specjutsu.toml"),
        "schema = 1\nmode = \"guided\"\n",
    )
    .unwrap();
    sj(dir.path(), &["init", "--agents", "claude"])
        .assert()
        .success();
    dir
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

// spec: ADAPT-046
#[test]
fn adapt_046_a_hand_edited_reference_is_kept_on_update() {
    let dir = project();
    let root = dir.path();
    let rel = ".claude/skills/sj-apply/task-reviewer-prompt.md";
    let edited = read(root, rel).replace("Your job is to find", "Your job is to find, twice,");
    assert_ne!(edited, read(root, rel));
    std::fs::write(root.join(rel), &edited).unwrap();

    sj(root, &["update"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(format!(
            "modified {rel} (kept; sj update --force replaces it)"
        )));
    assert_eq!(read(root, rel), edited);

    sj(root, &["update", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("updated {rel}")));
    assert!(!read(root, rel).contains("twice"));
}

// spec: ADAPT-047
#[test]
fn adapt_047_a_reference_no_longer_shipped_is_removed() {
    let dir = project();
    let root = dir.path();
    let rel = ".claude/skills/sj-debug/old-technique.md";
    std::fs::write(
        root.join(rel),
        sj_adapters::claude::generated("# An old technique\n"),
    )
    .unwrap();

    sj(root, &["update"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("removed {rel}")));
    assert!(!root.join(rel).exists());

    sj(root, &["init", "--remove-agents", "claude"])
        .assert()
        .success();
    let left: Vec<String> = std::fs::read_dir(root.join(".claude/skills"))
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with("sj-"))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(left, Vec::<String>::new());
}
