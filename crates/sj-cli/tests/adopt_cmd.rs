//! CONF-008..010 CLI scenarios from specjutsu/changes/project-config: `sj adopt`.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;

fn adopt(root: &std::path::Path, extra: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("adopt")
        .args(extra)
        .arg("--root")
        .arg(root)
        .assert()
}

#[test]
fn conf_008_dry_run_prints_the_config_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    adopt(dir.path(), &["--dry-run"])
        .success()
        .stdout(predicate::str::starts_with("schema = 1\n"))
        .stdout(predicate::str::contains("path = \"apps/app\""))
        .stdout(predicate::str::contains("runner = \"playwright\""))
        .stdout(predicate::str::contains("path = \"python/worker\""));
    assert!(!dir.path().join("specjutsu.toml").exists());
}

#[test]
fn conf_009_adopt_writes_a_file_every_command_accepts() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    adopt(dir.path(), &[]).success();
    let c = std::fs::read_to_string(dir.path().join("specjutsu.toml")).unwrap();
    assert!(c.contains("spec_root = \"openspec\""), "{c}");
    Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--root"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn conf_010_adopt_reports_packages_without_a_runner() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    adopt(dir.path(), &[])
        .success()
        .stderr(predicate::str::contains("no runner detected: packages/ui"));
}

#[test]
fn conf_010_adopt_never_overwrites_silently() {
    let dir = tempfile::tempdir().unwrap();
    support::monorepo(dir.path());
    std::fs::write(dir.path().join("specjutsu.toml"), "schema = 1\n").unwrap();
    adopt(dir.path(), &[])
        .code(2)
        .stderr(predicate::str::contains("--force"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("specjutsu.toml")).unwrap(),
        "schema = 1\n"
    );
    adopt(dir.path(), &["--force"]).success();
}

#[test]
fn conf_034_adopt_creates_the_spec_folders() {
    let cargo = "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    let dry = tempfile::tempdir().unwrap();
    std::fs::write(dry.path().join("Cargo.toml"), cargo).unwrap();
    adopt(dry.path(), &["--dry-run"]).success();
    assert!(!dry.path().join("specjutsu").exists());

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), cargo).unwrap();
    adopt(dir.path(), &[]).success();
    assert!(dir.path().join("specjutsu/specs").is_dir());
    assert!(dir.path().join("specjutsu/changes").is_dir());
    Command::cargo_bin("sj")
        .unwrap()
        .args(["doctor", "--root"])
        .arg(dir.path())
        .assert()
        .stdout(predicate::str::contains("missing spec root").not());
}
