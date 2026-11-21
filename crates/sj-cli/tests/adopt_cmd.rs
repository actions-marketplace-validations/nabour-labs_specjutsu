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
