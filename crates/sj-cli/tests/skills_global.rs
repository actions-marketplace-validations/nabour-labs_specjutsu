//! `--global`: the `sj-*` skills in the user's home folders, and where `sj doctor` finds them
//! (specjutsu/changes/more-adapters-2).

use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn sj(root: &Path, home: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env("HOME", home)
        .env("USERPROFILE", home);
    c
}

/// Every file under `root`, by relative path, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(d: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// A repository with a project file and an AGENTS.md.
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("specjutsu.toml"),
        "schema = 1\nmode = \"guided\"\n",
    )
    .unwrap();
    std::fs::write(root.join("AGENTS.md"), "# Shared rules\n").unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    dir
}

#[test]
fn adapt_034_a_global_install_writes_skills_only() {
    let dir = repo();
    let root = dir.path();
    let home = tempfile::tempdir().unwrap();
    let before = snapshot(root);

    sj(root, home.path())
        .args(["init", "--agents", "claude,codex", "--global"])
        .assert()
        .success();

    assert!(home
        .path()
        .join(".agents/skills/sj-apply/SKILL.md")
        .exists());
    assert!(home
        .path()
        .join(".claude/skills/sj-apply/SKILL.md")
        .exists());
    assert!(!home.path().join(".claude/settings.json").exists());
    assert!(!home.path().join(".codex").exists());
    assert_eq!(snapshot(root), before, "no file in the repository changed");

    // Update and removal act on the home folders only.
    std::fs::remove_file(home.path().join(".claude/skills/sj-apply/SKILL.md")).unwrap();
    sj(root, home.path())
        .args(["update", "--global"])
        .assert()
        .success();
    assert!(home
        .path()
        .join(".claude/skills/sj-apply/SKILL.md")
        .exists());
    sj(root, home.path())
        .args(["init", "--remove-agents", "claude,codex", "--global"])
        .assert()
        .success();
    assert!(!home.path().join(".claude/skills/sj-apply").exists());
    assert!(!home.path().join(".agents/skills/sj-apply").exists());
    assert_eq!(snapshot(root), before, "no file in the repository changed");
}

#[test]
fn adapt_035_doctor_shows_where_the_skills_come_from() {
    let dir = repo();
    let root = dir.path();
    let home = tempfile::tempdir().unwrap();

    sj(root, home.path())
        .args(["init", "--agents", "claude", "--global"])
        .assert()
        .success();
    sj(root, home.path())
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("skills: global\n"));

    sj(root, home.path())
        .args(["init", "--agents", "claude"])
        .assert()
        .success();
    sj(root, home.path())
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("skills: project, global\n"));
}
