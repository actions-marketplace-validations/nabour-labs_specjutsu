//! CONF-006 and CONF-007 from openspec/changes/project-config: `sj init`.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

fn empty_repo() -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    repo
}

fn init(repo: &Repo, extra: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("init")
        .args(extra)
        .arg("--root")
        .arg(repo.path())
        .assert()
}

#[test]
fn conf_006_init_in_an_empty_repository() {
    let repo = empty_repo();
    init(&repo, &[]).success();
    let written = std::fs::read_to_string(repo.path().join("specjutsu.toml")).unwrap();
    assert_eq!(
        written,
        "schema = 1\nspec_root = \"openspec\"\nmode = \"guided\"\n"
    );
    assert!(repo.path().join("openspec/specs").is_dir());
    assert!(repo.path().join("openspec/changes").is_dir());
}

#[test]
fn conf_006_init_in_a_rust_repository_keeps_the_cargo_suite() {
    let repo = empty_repo();
    repo.write("Cargo.toml", "[workspace]\n");
    init(&repo, &[]).success();
    let c = sj_core_config(&repo);
    assert!(c.contains("[[package]]\npath = \".\""), "{c}");
    assert!(c.contains("runner = \"cargo\""), "{c}");
}

fn sj_core_config(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join("specjutsu.toml")).unwrap()
}

#[test]
fn conf_007_init_never_overwrites_silently() {
    let repo = empty_repo();
    repo.write("specjutsu.toml", "# mine\nschema = 1\n");
    init(&repo, &[])
        .code(2)
        .stderr(predicate::str::contains("specjutsu.toml"))
        .stderr(predicate::str::contains("--force"));
    assert_eq!(sj_core_config(&repo), "# mine\nschema = 1\n");

    init(&repo, &["--force"]).success();
    assert!(sj_core_config(&repo).starts_with("schema = 1\nspec_root"));
}
