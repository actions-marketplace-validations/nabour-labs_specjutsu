//! LOOP-022 to LOOP-026 from specjutsu/changes/add-sj-gain: `sj test`, `sj verify`, `sj next`
//! and `sj status` record their gain.
//!
//! The fixture uses the generic junit runner: its command copies a canned report and prints a
//! payload file of a fixed size with the OS shell, so the runner's output size is exact on every
//! platform.

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
    fixture_with(git, RUNNER_BYTES)
}

/// A junit-suite project whose runner writes exactly `bytes` bytes; a git repo if `git`.
fn fixture_with(git: bool, bytes: usize) -> Repo {
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
    repo.write("svc/payload.txt", &"x".repeat(bytes));
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

const VERIFY_RUNNER_BYTES: usize = 25_000;

/// A project whose open change `add-svc` adds SVC-001 and SVC-002, each with a test, so verify
/// starts the runner twice; each run writes 12,500 bytes, 25,000 in all. In git, `main` holds
/// the project and the change sits on a `feature` branch.
fn verify_fixture(git: bool) -> Repo {
    let repo = fixture_with(git, VERIFY_RUNNER_BYTES / 2);
    repo.write(
        "svc/tests/a.test.js",
        "it(\"SVC-001 works\", () => {})\nit(\"SVC-002 answers\", () => {})\n",
    );
    repo.write(
        "svc/report.src.xml",
        "<testsuite><testcase classname=\"svc\" name=\"SVC-001 works\"/>\
         <testcase classname=\"svc\" name=\"SVC-002 answers\"/></testsuite>",
    );
    if git {
        repo.commit("base");
        repo.branch("feature");
    }
    repo.write("openspec/changes/add-svc/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/add-svc/specs/all/spec.md",
        "## ADDED Requirements\n\n### Requirement: S\nThe system SHALL serve.\n\n\
         #### Scenario: SVC-001 Service works\n- **WHEN** x\n- **THEN** y\n\n\
         #### Scenario: SVC-002 Service answers\n- **WHEN** x\n- **THEN** y\n",
    );
    repo.write(
        "openspec/changes/add-svc/tasks.md",
        "- [ ] 1.1 Serve (SVC-001, SVC-002)\n",
    );
    if git {
        repo.commit("change");
    }
    repo
}

fn sj_verify(root: &Path) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("verify")
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn loop_023_sj_verify_records_runner_output_as_raw() {
    let repo = verify_fixture(true);
    let out = sj_verify(repo.path());
    assert!(out.status.code().is_some(), "{out:?}");
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "verify");
    assert!(!out.stdout.is_empty());
    assert_eq!(r["printed"], out.stdout.len() as u64);
    assert_eq!(r["raw"], VERIFY_RUNNER_BYTES as u64);
    assert!(out.stdout.len() < VERIFY_RUNNER_BYTES);
}

#[test]
fn loop_026_no_gain_record_from_verify_outside_git() {
    // `sj verify` needs git: outside a repository it stops with an error, as before, and
    // records nothing.
    let outside = verify_fixture(false);
    let out = sj_verify(outside.path());
    assert!(!gain_path(outside.path()).exists());
    assert!(out.stdout.is_empty(), "{out:?}");
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("is not in a git repository"),
        "{out:?}"
    );
}

const NEXT_CHANGE_BYTES: usize = 12_000;

/// A project with one open change, `add-oauth-login`, whose files (specs/ included) total
/// exactly 12,000 bytes; a git repo if `git`.
fn next_fixture(git: bool) -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    if git {
        repo.git(&["init", "-q", "-b", "main"]);
    }
    repo.write("openspec/specs/all/spec.md", SPEC);
    let dir = "openspec/changes/add-oauth-login";
    let files = [
        ("proposal.md", "## Why\n\nLog in with OAuth.\n".to_string()),
        (
            "specs/auth/spec.md",
            "## ADDED Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
             #### Scenario: AUTH-001 Logs in with OAuth\n- **WHEN** x\n- **THEN** y\n"
                .to_string(),
        ),
        ("tasks.md", "- [ ] 1.1 Log in (AUTH-001)\n".to_string()),
    ];
    let mut total = 0;
    for (name, text) in &files {
        repo.write(&format!("{dir}/{name}"), text);
        total += text.len();
    }
    repo.write(
        &format!("{dir}/design.md"),
        &"x".repeat(NEXT_CHANGE_BYTES - total),
    );
    repo
}

fn sj_next(root: &Path) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["next", "add-oauth-login", "--root"])
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn loop_024_sj_next_records_change_folder_as_raw() {
    let repo = next_fixture(true);
    let out = sj_next(repo.path());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "next");
    assert!(!out.stdout.is_empty());
    assert_eq!(r["printed"], out.stdout.len() as u64);
    assert_eq!(r["raw"], NEXT_CHANGE_BYTES as u64);
    assert!(out.stdout.len() < NEXT_CHANGE_BYTES);
}

#[test]
fn loop_026_no_gain_record_from_next_outside_git() {
    let on = next_fixture(true);
    let with = sj_next(on.path());
    assert_eq!(gain_records(on.path()).len(), 1);

    let outside = next_fixture(false);
    let without = sj_next(outside.path());
    assert!(!gain_path(outside.path()).exists());
    assert_eq!(without.stdout, with.stdout);
    assert_eq!(without.status.code(), with.status.code());
}

#[test]
fn loop_024_sj_next_with_no_open_task_records_raw_zero() {
    let repo = next_fixture(true);
    repo.write(
        "openspec/changes/add-oauth-login/tasks.md",
        "- [x] 1.1 Log in (AUTH-001)\n",
    );
    let out = sj_next(repo.path());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "next: no open tasks\n"
    );
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "next");
    assert_eq!(r["printed"], out.stdout.len() as u64);
    assert_eq!(r["raw"], 0);
}

/// A project with two open changes, `add-a` and `add-b`, whose folders total exactly 8,000 and
/// 5,000 bytes; a git repo if `git`.
fn status_fixture(git: bool) -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    if git {
        repo.git(&["init", "-q", "-b", "main"]);
    }
    repo.write("openspec/specs/all/spec.md", SPEC);
    for (name, bytes) in [("add-a", 8_000), ("add-b", 5_000)] {
        let dir = format!("openspec/changes/{name}");
        let tasks = "- [ ] 1.1 Work (SVC-001)\n";
        repo.write(&format!("{dir}/tasks.md"), tasks);
        repo.write(
            &format!("{dir}/proposal.md"),
            &"x".repeat(bytes - tasks.len()),
        );
    }
    repo
}

fn sj_status(root: &Path) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["status", "--root"])
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn loop_025_sj_status_records_every_open_change_as_raw() {
    let repo = status_fixture(true);
    let out = sj_status(repo.path());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let records = gain_records(repo.path());
    assert_eq!(records.len(), 1, "{records:?}");
    let r = &records[0];
    assert_eq!(r["command"], "status");
    assert!(!out.stdout.is_empty());
    assert_eq!(r["printed"], out.stdout.len() as u64);
    assert_eq!(r["raw"], 13_000);
    assert!(out.stdout.len() < 13_000);
}

#[test]
fn loop_026_no_gain_record_from_status_outside_git() {
    let on = status_fixture(true);
    let with = sj_status(on.path());
    assert_eq!(gain_records(on.path()).len(), 1);

    let outside = status_fixture(false);
    let without = sj_status(outside.path());
    assert!(!gain_path(outside.path()).exists());
    assert_eq!(without.stdout, with.stdout);
    assert_eq!(without.status.code(), with.status.code());
}

fn gain_line(command: &str, printed: u64, raw: u64) -> String {
    gain_line_at(command, "2026-11-22T10:00:00Z", printed, raw)
}

fn gain_line_at(command: &str, ts: &str, printed: u64, raw: u64) -> String {
    format!(
        "{{\"v\":1,\"command\":\"{command}\",\"ts\":\"{ts}\",\"printed\":{printed},\"raw\":{raw}}}\n"
    )
}

fn sj_gain(root: &Path) -> std::process::Output {
    sj_gain_with(root, &[])
}

fn sj_gain_with(root: &Path, extra: &[&str]) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("gain")
        .args(extra)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

/// Each stdout line split on whitespace, so column padding doesn't matter.
fn tokens(stdout: &[u8]) -> Vec<Vec<String>> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|l| l.split_whitespace().map(String::from).collect())
        .collect()
}

fn words(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

#[test]
fn loop_027_gain_totals_per_command_and_overall() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text = [
        gain_line("test", 300, 40_000),
        gain_line("next", 1_000, 4_000),
        gain_line("test", 200, 10_000),
    ]
    .concat();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let out = sj_gain(repo.path());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        tokens(&out.stdout),
        vec![
            words("test 2 500 50000 99%"),
            words("next 1 1000 4000 75%"),
            words("total 3 1500 54000 97% ~13125 tokens saved"),
        ]
    );
    // In columns: the numbers end at the same place on every line.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let ends: Vec<usize> = stdout
        .lines()
        .map(|l| l.find('%').expect("a percent column"))
        .collect();
    assert!(ends.windows(2).all(|w| w[0] == w[1]), "{stdout}");
}

#[test]
fn loop_031_damaged_gain_record_does_not_block_report() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text = [
        gain_line("test", 300, 40_000),
        "{not json\n".to_string(),
        gain_line("test", 200, 10_000),
        gain_line("next", 1_000, 4_000),
    ]
    .concat();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let out = sj_gain(repo.path());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        tokens(&out.stdout),
        vec![
            words("test 2 500 50000 99%"),
            words("next 1 1000 4000 75%"),
            words("total 3 1500 54000 97% ~13125 tokens saved"),
        ],
        "{out:?}"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("skipped 1 unreadable gain record"),
        "{out:?}"
    );
}

#[test]
fn loop_028_gain_json_totals_for_a_harness() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text = [
        gain_line("test", 300, 40_000),
        gain_line("next", 1_000, 4_000),
        gain_line("test", 200, 10_000),
    ]
    .concat();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let out = sj_gain_with(repo.path(), &["--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    // Exactly one JSON value on stdout, followed by a newline.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.ends_with('\n'), "{stdout}");
    let mut values = serde_json::Deserializer::from_str(&stdout).into_iter::<serde_json::Value>();
    let v = values.next().expect("a JSON value").expect("valid JSON");
    assert!(values.next().is_none(), "one JSON value: {stdout}");
    assert_eq!(
        v,
        serde_json::json!({
            "commands": [
                {"command": "test", "runs": 2, "printed": 500, "raw": 50000, "saved_pct": 99},
                {"command": "next", "runs": 1, "printed": 1000, "raw": 4000, "saved_pct": 75},
            ],
            "total": {
                "runs": 3, "printed": 1500, "raw": 54000, "saved_pct": 97, "tokens_saved": 13125
            },
            "skipped": 0,
        })
    );
}

#[test]
fn loop_028_gain_json_reports_skipped_records() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text = [
        gain_line("test", 300, 40_000),
        "{not json\n".to_string(),
        gain_line("test", 200, 10_000),
        gain_line("next", 1_000, 4_000),
    ]
    .concat();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let out = sj_gain_with(repo.path(), &["--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(v["skipped"], 1, "{v}");
    assert_eq!(v["total"]["tokens_saved"], 13125, "{v}");
}

#[test]
fn loop_029_gain_since_counts_one_session() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text = [
        gain_line_at("test", "2026-11-22T10:00:00Z", 300, 40_000),
        gain_line_at("test", "2026-11-22T12:00:00Z", 200, 10_000),
    ]
    .concat();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let out = sj_gain_with(repo.path(), &["--since", "2026-11-22T11:00:00Z"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        tokens(&out.stdout),
        vec![
            words("test 1 200 10000 98%"),
            words("total 1 200 10000 98% ~2450 tokens saved"),
        ],
        "{out:?}"
    );
}

#[test]
fn loop_030_gain_reset_clears_the_records() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    let text: String = (0..7).map(|i| gain_line("test", 100 + i, 1_000)).collect();
    repo.write(".specjutsu/ledger/gain.jsonl", &text);
    let reset = sj_gain_with(repo.path(), &["--reset"]);
    assert_eq!(reset.status.code(), Some(0), "{reset:?}");
    assert_eq!(
        String::from_utf8_lossy(&reset.stdout),
        "removed 7 gain records\n"
    );
    assert!(!gain_path(repo.path()).exists());
    let after = sj_gain(repo.path());
    assert_eq!(after.status.code(), Some(0), "{after:?}");
    assert_eq!(
        String::from_utf8_lossy(&after.stdout),
        "no gain records yet\n"
    );
}
