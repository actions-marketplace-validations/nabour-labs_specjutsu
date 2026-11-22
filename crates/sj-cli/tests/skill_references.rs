//! Reference files shipped next to a skill's `SKILL.md` (specjutsu/changes/deep-skills).

use std::path::Path;

use assert_cmd::Command;

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env("HOME", root.join("home"));
    c
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

fn shipped(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills")
        .join(rel);
    std::fs::read_to_string(p).unwrap().replace("\r\n", "\n")
}

// spec: ADAPT-045
#[test]
fn adapt_045_init_installs_the_reference_files() {
    let dir = project();
    let root = dir.path();
    sj(root)
        .args(["init", "--agents", "claude,codex"])
        .assert()
        .success();

    let want = sj_adapters::claude::generated(&shipped("sj-apply/task-reviewer-prompt.md"));
    for base in [".claude/skills", ".agents/skills"] {
        let p = root.join(base).join("sj-apply/task-reviewer-prompt.md");
        let got = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        assert_eq!(got, want, "{}", p.display());
    }
    sj(root)
        .arg("update")
        .assert()
        .success()
        .stdout("adapters up to date\n");
}
