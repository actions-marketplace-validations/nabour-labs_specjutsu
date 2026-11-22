//! LOOP-022 and LOOP-026 from specjutsu/changes/add-sj-gain: `sj test` records its gain.
//!
//! The fixture uses the generic junit runner: its command copies a canned report and prints a
//! 40,000-byte file with the OS shell, so the runner's output size is exact on every platform.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::Path;

use assert_cmd::Command;
use support::Repo;

const RUNNER_BYTES: usize = 40_000;

const SPEC: &str = "## Requirements\n\n### Requirement: R\nThe system SHALL work.\n\n\
    #### Scenario: SVC-001 Service works\n- **WHEN** x\n- **THEN** y\n";

/// A junit-suite project whose runner writes exactly `RUNNER_BYTES` bytes; a git repo if `git`.
fn fixture(git: bool) -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    if git {
        repo.git(&["init", "-q", "-b", "main"]);
    }
    let command = if cfg!(windows) {
        r#"(if not exist "build" mkdir "build") && copy /Y "report.src.xml" "build\junit.xml" >NUL && type "payload.txt""#
    } else {
        "mkdir -p build && cp report.src.xml build/junit.xml && cat payload.txt"
    };
    repo.write(
        "specjutsu.toml",
        &format!(
            "schema = 1\n\n[[package]]\npath = \"svc\"\nsources = []\n\n[[package.suite]]\n\
             runner = \"junit\"\ntests = [\"svc/tests/**\"]\ncommand = '{command}'\nreport = \"build/junit.xml\"\n"
        ),
    );
    repo.write("openspec/specs/all/spec.md", SPEC);
    repo.write("svc/tests/a.test.js", "it(\"SVC-001 works\", () => {})\n");
    repo.write(
        "svc/report.src.xml",
        "<testsuite><testcase classname=\"svc\" name=\"SVC-001 works\"/></testsuite>",
    );
    repo.write("svc/payload.txt", &"x".repeat(RUNNER_BYTES));
    repo
}

fn sj_test(root: &Path, extra: &[&str]) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("test")
        .args(extra)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

fn gain_path(root: &Path) -> std::path::PathBuf {
    root.join(".specjutsu/ledger/gain.jsonl")
}

fn gain_records(root: &Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(gain_path(root))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).expect("each gain line is JSON"))
        .collect()
}

#[test]
fn loop_022_sj_test_records_printed_and_raw_bytes() {
    let repo = fixture(true);
    let out = sj_test(repo.path(), &[]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "test");
    assert!(!out.stdout.is_empty());
    assert!(out.stdout.len() < RUNNER_BYTES);
    assert_eq!(r["printed"], out.stdout.len() as u64);
    assert_eq!(r["raw"], RUNNER_BYTES as u64);
    let ts = r["ts"].as_str().expect("ts is a string");
    assert!(
        ts.len() == 20 && &ts[10..11] == "T" && ts.ends_with('Z'),
        "ts {ts} is RFC 3339 UTC"
    );
}

#[test]
fn loop_022_sj_test_records_cargo_output_as_raw() {
    // Cargo's output size varies by toolchain, so assert it was counted and exceeds what sj printed.
    let repo = Repo::with_lib("");
    repo.write("openspec/specs/all/spec.md", SPEC);
    repo.write("tests/t.rs", "#[test]\nfn svc_001_works() {}\n");
    let out = sj_test(repo.path(), &[]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "test");
    assert!(!out.stdout.is_empty());
    assert_eq!(r["printed"], out.stdout.len() as u64);
    let raw = r["raw"].as_u64().expect("raw is a number");
    assert!(
        raw > out.stdout.len() as u64,
        "raw {raw} counts cargo's output"
    );
}

#[test]
fn loop_026_no_gain_record_when_no_ledger() {
    let on = fixture(true);
    let with = sj_test(on.path(), &[]);
    assert_eq!(gain_records(on.path()).len(), 1);

    let off = fixture(true);
    let without = sj_test(off.path(), &["--no-ledger"]);
    assert!(!gain_path(off.path()).exists());
    assert_eq!(without.stdout, with.stdout);
    assert_eq!(without.status.code(), with.status.code());
}

#[test]
fn loop_026_no_gain_record_outside_git() {
    let on = fixture(true);
    let with = sj_test(on.path(), &[]);
    assert_eq!(gain_records(on.path()).len(), 1);

    let outside = fixture(false);
    let without = sj_test(outside.path(), &[]);
    assert!(!gain_path(outside.path()).exists());
    assert_eq!(without.stdout, with.stdout);
    assert_eq!(without.status.code(), with.status.code());
}
