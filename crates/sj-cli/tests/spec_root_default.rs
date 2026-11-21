//! CONF-032 from openspec/changes/own-spec-root: which folder commands use
//! when `spec_root` is not set.

use std::path::Path;

use assert_cmd::Command;

fn change(root: &Path, rel: &str) {
    let dir = root.join(rel);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    std::fs::write(dir.join("tasks.md"), "# Tasks\n").unwrap();
    std::fs::write(dir.join("proposal.md"), "# Proposal\n").unwrap();
}

fn list(root: &Path) -> String {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["spec", "list", "--root"])
        .arg(root)
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

#[test]
fn conf_032_sj_spec_list_moves_to_the_specjutsu_folder_once_it_exists() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("specjutsu.toml"), "schema = 1\n").unwrap();
    change(dir.path(), "openspec/changes/old-change");
    let out = list(dir.path());
    assert!(out.contains("old-change"), "{out}");

    change(dir.path(), "specjutsu/changes/new-change");
    let out = list(dir.path());
    assert!(out.contains("new-change"), "{out}");
    assert!(!out.contains("old-change"), "{out}");
}
