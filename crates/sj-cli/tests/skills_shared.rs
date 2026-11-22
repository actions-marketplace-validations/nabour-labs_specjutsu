//! The `sj-*` skills in the shared `.agents/skills/` folder, beside Claude Code's copy
//! (specjutsu/changes/more-adapters-2).

use std::path::Path;

use assert_cmd::Command;

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env("HOME", root.join("home"));
    c
}

/// The `sj-*` skill folders under `dir`, sorted.
fn sj_skills(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("sj-"))
        .collect();
    out.sort();
    out
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("specjutsu.toml"),
        "schema = 1\nmode = \"guided\"\n",
    )
    .unwrap();
    dir
}

#[test]
fn adapt_032_codex_and_gemini_share_one_skills_folder() {
    let dir = project();
    let root = dir.path();

    sj(root)
        .args(["init", "--agents", "claude,codex,gemini"])
        .assert()
        .success();

    let shared = sj_skills(&root.join(".agents/skills"));
    let claude = sj_skills(&root.join(".claude/skills"));
    assert!(shared.contains(&"sj-apply".to_string()), "{shared:?}");
    assert_eq!(shared, claude);
    for name in &shared {
        let files: Vec<_> = std::fs::read_dir(root.join(".agents/skills").join(name))
            .unwrap()
            .collect();
        assert!(
            files
                .iter()
                .filter(|e| e.as_ref().unwrap().file_name() == "SKILL.md")
                .count()
                == 1,
            "{name}: one SKILL.md"
        );
    }
    assert_eq!(
        std::fs::read(root.join(".agents/skills/sj-apply/SKILL.md")).unwrap(),
        std::fs::read(root.join(".claude/skills/sj-apply/SKILL.md")).unwrap(),
    );

    // `sj update` keeps both copies in step: an older copy is refreshed in each.
    for d in [".agents", ".claude"] {
        std::fs::remove_file(root.join(d).join("skills/sj-apply/SKILL.md")).unwrap();
    }
    sj(root).arg("update").assert().success();
    assert_eq!(
        std::fs::read(root.join(".agents/skills/sj-apply/SKILL.md")).unwrap(),
        std::fs::read(root.join(".claude/skills/sj-apply/SKILL.md")).unwrap(),
    );
}

#[test]
fn adapt_033_removing_the_last_reader_removes_the_shared_skills() {
    let dir = project();
    let root = dir.path();
    let own = root.join(".agents/skills/team-style/SKILL.md");
    std::fs::create_dir_all(own.parent().unwrap()).unwrap();
    std::fs::write(&own, "# Our style\n").unwrap();

    sj(root)
        .args(["init", "--agents", "codex,gemini"])
        .assert()
        .success();
    let apply = root.join(".agents/skills/sj-apply/SKILL.md");
    assert!(apply.exists());

    sj(root)
        .args(["init", "--remove-agents", "codex"])
        .assert()
        .success();
    assert!(apply.exists(), "gemini still reads .agents/skills");

    sj(root)
        .args(["init", "--remove-agents", "gemini"])
        .assert()
        .success();
    assert!(!apply.exists());
    assert!(sj_skills(&root.join(".agents/skills")).is_empty());
    assert_eq!(std::fs::read_to_string(&own).unwrap(), "# Our style\n");
}
