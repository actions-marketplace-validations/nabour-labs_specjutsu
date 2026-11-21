//! `sj hook claude session-start` and `sj hook claude pre-edit`
//! (specjutsu/changes/claude-adapter).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn hook(root: &Path, event: &str) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(["hook", "claude", event, "--root"])
        .arg(root)
        .env_remove("CLAUDE_PROJECT_DIR");
    c
}

fn delta(ids: &[&str]) -> String {
    let mut s = String::from(
        "## ADDED Requirements\n\n### Requirement: Tokens\nThe system SHALL refresh tokens.\n\n",
    );
    for id in ids {
        s.push_str(&format!(
            "#### Scenario: {id} Refresh\n- **WHEN** a token expires\n- **THEN** {id} holds\n\n"
        ));
    }
    s
}

/// A vitest package with one open change whose next task names AUTH-005.
fn project(mode: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(
        root,
        "specjutsu.toml",
        &format!(
            "schema = 1\nmode = \"{mode}\"\n\n[[package]]\npath = \"apps/api\"\n\n\
             [[package.suite]]\nrunner = \"vitest\"\ntests = [\"**/*.test.ts\"]\n"
        ),
    );
    put(root, "apps/api/src/auth.ts", "export const auth = 1\n");
    put(root, "openspec/specs/.gitkeep", "");
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &delta(&["AUTH-005"]),
    );
    put(
        root,
        "openspec/changes/add-oauth-login/tasks.md",
        "- [x] 1.1 Setup (no-scenario: setup)\n- [ ] 2.1 Refresh tokens (AUTH-005)\n",
    );
    dir
}

fn edit(root: &Path, rel: &str) -> String {
    let path = root.join(rel).to_string_lossy().replace('\\', "/");
    format!(
        r#"{{"session_id":"s","hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{{"file_path":"{path}","old_string":"a","new_string":"b"}}}}"#
    )
}

fn ledger(root: &Path, id: &str, result: &str) {
    put(
        root,
        ".specjutsu/ledger/runs.jsonl",
        &format!(
            "{{\"v\":1,\"run\":\"r\",\"ts\":1,\"tree\":\"t\",\"test\":\"auth.test.ts::AUTH-005 refreshes\",\"ids\":[\"{id}\"],\"result\":\"{result}\"}}\n"
        ),
    );
}

#[test]
fn adapt_005_bootstrap_is_small() {
    let dir = project("guided");
    put(
        dir.path(),
        "openspec/changes/zz-billing/specs/billing/spec.md",
        &delta(&["BILL-001", "BILL-002"]),
    );
    put(
        dir.path(),
        "openspec/changes/zz-billing/tasks.md",
        &format!(
            "- [ ] 1.1 {} (BILL-001, BILL-002)\n",
            "Invoice ".repeat(200)
        ),
    );
    let out = hook(dir.path(), "session-start")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(out).unwrap();
    assert!(out.len() <= 1200, "{} bytes: {out}", out.len());
    assert!(out.contains("add-oauth-login"), "{out}");
    assert!(out.contains("zz-billing"), "{out}");
    assert!(
        out.contains("active add-oauth-login: next 2.1 Refresh tokens (AUTH-005)"),
        "{out}"
    );
}

#[test]
fn adapt_005_bootstrap_without_open_changes() {
    let dir = tempfile::tempdir().unwrap();
    hook(dir.path(), "session-start")
        .assert()
        .success()
        .stdout("specjutsu: no open changes\n");
}

#[test]
fn adapt_006_editing_source_before_a_red_test_is_blocked() {
    let dir = project("guided");
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "write a failing test for AUTH-005 first (sj test AUTH-005)",
        ));
    // Strict blocks too, and a passing record alone is not red.
    let dir = project("strict");
    ledger(dir.path(), "AUTH-005", "passed");
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(2);
}

#[test]
fn adapt_007_writing_the_test_is_always_allowed() {
    let dir = project("guided");
    for rel in [
        "apps/api/src/auth.test.ts",
        "openspec/changes/add-oauth-login/tasks.md",
        "README.md",
        "apps/api/package.json",
    ] {
        hook(dir.path(), "pre-edit")
            .write_stdin(edit(dir.path(), rel))
            .assert()
            .code(0);
    }
    // A file outside the project.
    let other = tempfile::tempdir().unwrap();
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(other.path(), "src/main.ts"))
        .assert()
        .code(0);
}

#[test]
fn adapt_008_after_a_red_run_source_edits_are_allowed() {
    let dir = project("guided");
    ledger(dir.path(), "AUTH-005", "failed");
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(0);
    let dir = project("guided");
    ledger(dir.path(), "AUTH-005", "build-failed");
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(0);
}

#[test]
fn adapt_008_advisory_mode_warns_and_allows() {
    let dir = project("advisory");
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "specjutsu: write a failing test for AUTH-005 first (sj test AUTH-005)",
        ));
}

#[test]
fn adapt_009_a_hook_failure_never_blocks_work() {
    let dir = project("strict");
    // The ledger path is a directory, so it cannot be read.
    std::fs::create_dir_all(dir.path().join(".specjutsu/ledger/runs.jsonl")).unwrap();
    hook(dir.path(), "pre-edit")
        .write_stdin(edit(dir.path(), "apps/api/src/auth.ts"))
        .assert()
        .code(0)
        .stderr(predicate::str::contains("specjutsu: hook error:"))
        .stderr(predicate::str::contains("runs.jsonl"));
    // Garbage on stdin.
    hook(dir.path(), "pre-edit")
        .write_stdin("not json")
        .assert()
        .code(0)
        .stderr(predicate::str::contains("specjutsu: hook error:"));
    // A broken project file.
    put(dir.path(), "specjutsu.toml", "schema = 99\n");
    hook(dir.path(), "session-start").assert().code(0);
}
