//! `sj doctor` reports each installed agent's enforcement level
//! (openspec/changes/more-adapters).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root).env_remove("CLAUDE_PROJECT_DIR");
    c
}

#[test]
fn adapt_020_doctor_shows_how_strongly_each_agent_is_held() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("specjutsu.toml"),
        "schema = 1\nmode = \"guided\"\n",
    )
    .unwrap();

    sj(root)
        .args(["init", "--agents", "claude,cursor,agents-md"])
        .assert()
        .success();
    sj(root)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("claude: hook\n"))
        .stdout(predicate::str::contains("cursor: hook\n"))
        .stdout(predicate::str::contains("agents-md: instructions\n"));

    // The Cursor hook entry is deleted by hand.
    std::fs::write(
        root.join(".cursor/hooks.json"),
        "{\n  \"version\": 1,\n  \"hooks\": {}\n}\n",
    )
    .unwrap();
    sj(root)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("claude: hook\n"))
        .stdout(predicate::str::contains("cursor: hook missing\n"))
        .stdout(predicate::str::contains("agents-md: instructions\n"));
}
