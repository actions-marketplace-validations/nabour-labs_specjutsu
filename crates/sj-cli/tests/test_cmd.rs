//! `sj test` CLI scenarios (specjutsu/changes/test-ledger).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::{spec_md, Repo};

const PASSING: &str = "#[test]\nfn foo_001_adds() {}\n\n#[test]\nfn foo_002_subtracts() {}\n";
const ONE_FAILING: &str =
    "#[test]\nfn foo_001_adds() { panic!(\"boom 001\"); }\n\n#[test]\nfn foo_002_subtracts() {}\n";

fn fixture(tests: &str) -> Repo {
    let repo = Repo::with_lib("");
    repo.write(
        "openspec/specs/math/spec.md",
        &spec_md(&[("FOO-001", "Add"), ("FOO-002", "Subtract")]),
    );
    repo.write("tests/math.rs", tests);
    repo
}

fn sj(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("test").arg("--root").arg(root);
    c
}

#[test]
fn run_005_all_passing_prints_one_line() {
    let repo = fixture(PASSING);
    sj(repo.path())
        .assert()
        .success()
        .stdout("test: pass — 2 passed\n");
}

#[test]
fn run_006_failing_test_is_listed_with_ids_and_excerpt() {
    let repo = fixture(ONE_FAILING);
    sj(repo.path())
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with(
            "test: fail — 1 failed, 1 passed\n",
        ))
        .stdout(predicate::str::contains("fail    foo_001_adds  FOO-001\n"))
        .stdout(predicate::str::contains("boom 001"))
        .stdout(predicate::str::contains("foo_002").not());
}

#[test]
fn run_008_build_failure_shows_compiler_errors() {
    let repo = fixture("#[test]\nfn foo_001_adds() { nope(); }\n");
    sj(repo.path())
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with("test: fail — build failed\n"))
        .stdout(predicate::str::contains("error"))
        .stdout(predicate::str::contains("0 passed").not());
}

#[test]
fn run_009_exit_codes_distinguish_pass_fail_and_error() {
    sj(fixture(PASSING).path()).assert().code(0);
    sj(fixture(ONE_FAILING).path()).assert().code(1);
    let empty = tempfile::tempdir().unwrap();
    sj(empty.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no Cargo.toml"));
}

#[test]
fn run_003_unmapped_scenario_exits_two() {
    let repo = fixture(PASSING);
    sj(repo.path())
        .arg("FOO-009")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("FOO-009 has no mapped test"));
}

#[test]
fn run_010_json_lists_every_test_with_ids_and_result() {
    let repo = fixture(ONE_FAILING);
    let out = sj(repo.path())
        .arg("--json")
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("stdout is one JSON object");
    assert_eq!(v["pass"], false);
    assert_eq!(v["build_failed"], false);
    let tests = v["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 2);
    let get = |name: &str| tests.iter().find(|t| t["path"] == name).unwrap();
    assert_eq!(get("foo_001_adds")["result"], "failed");
    assert_eq!(get("foo_001_adds")["ids"], serde_json::json!(["FOO-001"]));
    assert_eq!(get("foo_002_subtracts")["result"], "passed");
}

#[test]
fn ledger_006_no_ledger_writes_nothing() {
    let repo = fixture(PASSING);
    sj(repo.path()).arg("--no-ledger").assert().success();
    assert!(!repo.path().join(".specjutsu").exists());
    sj(repo.path()).assert().success();
    assert!(repo.path().join(".specjutsu/ledger/runs.jsonl").is_file());
}

#[test]
fn ledger_007_outside_git_tests_run_and_recording_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let w = |rel: &str, s: &str| {
        let p = dir.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, s).unwrap();
    };
    w(
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    w("src/lib.rs", "");
    w("tests/t.rs", "#[test]\nfn t() {}\n");
    sj(dir.path())
        .assert()
        .success()
        .stdout("test: pass — 1 passed\n")
        .stderr(predicate::str::contains(
            "not a git repository; run not recorded",
        ));
    assert!(!dir.path().join(".specjutsu").exists());
}
