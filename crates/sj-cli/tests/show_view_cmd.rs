//! `sj spec show` and `sj view` scenarios (specjutsu/changes/spec-show-view): SPEC-013..019 and
//! LOOP-008..011.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(root);
    c
}

fn stdout(root: &Path, args: &[&str]) -> String {
    let out = sj(root, args).output().unwrap();
    assert!(
        out.status.success(),
        "sj {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

const CONFIG: &str = "schema = 1\n\n[[package]]\npath = \".\"\nsources = []\n\n\
                      [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\n";

/// A capability spec with `n` scenarios `PREFIX-001..`, one requirement each.
fn spec(cap: &str, prefix: &str, n: usize) -> String {
    let mut s =
        format!("# {cap} Specification\n\n## Purpose\nThe {cap} capability.\n\n## Requirements\n");
    for i in 1..=n {
        s.push_str(&format!(
            "\n### Requirement: {cap} rule {i}\nThe system SHALL follow {cap} rule {i}.\n\n\
             #### Scenario: {prefix}-{i:03} {cap} case {i}\n- **WHEN** {cap} case {i} happens\n- **THEN** {cap} rule {i} holds\n"
        ));
    }
    s
}

const AUTH: &str = "# auth Specification\n\n## Purpose\nTokens.\n\n## Requirements\n\n\
### Requirement: Token checks\nThe system SHALL reject expired tokens.\n\n\
#### Scenario: AUTH-003 Rejects expired token\n- **WHEN** a request carries a token that expired\n\
- **THEN** the API answers 401\n";

/// Write ledger lines: (tree, ID, result).
fn ledger(root: &Path, records: &[(&str, &str, &str)]) {
    let mut s = String::new();
    for (i, (tree, id, result)) in records.iter().enumerate() {
        s.push_str(&format!(
            "{{\"v\":1,\"run\":\"r{i}\",\"ts\":{i},\"tree\":\"{tree}\",\"test\":\"t\",\"ids\":[\"{id}\"],\"result\":\"{result}\"}}\n"
        ));
    }
    put(root, ".specjutsu/ledger/runs.jsonl", &s);
}

const DELTA: &str = "## ADDED Requirements\n\n### Requirement: OAuth\nThe system SHALL log in with OAuth.\n\n\
#### Scenario: AUTH-009 Google login\n- **WHEN** the user picks Google\n- **THEN** they are signed in\n";

const TASKS: &str = "# Tasks\n\n- [x] 1.1 a (AUTH-009)\n- [x] 1.2 b (no-scenario: x)\n- [x] 1.3 c (no-scenario: x)\n\
- [ ] 1.4 d (no-scenario: x)\n- [ ] 1.5 e (no-scenario: x)\n- [ ] 1.6 f (no-scenario: x)\n- [ ] 1.7 g (no-scenario: x)\n";

fn show_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(r, "specjutsu.toml", CONFIG);
    put(r, "openspec/specs/auth/spec.md", AUTH);
    put(
        r,
        "openspec/specs/audit/spec.md",
        &spec("audit", "AUDIT", 1),
    );
    put(
        r,
        "openspec/changes/add-oauth-login/proposal.md",
        "## Why\n\nUsers want Google login.\n",
    );
    put(
        r,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        DELTA,
    );
    put(r, "openspec/changes/add-oauth-login/tasks.md", TASKS);
    dir
}

// spec: SPEC-013
#[test]
fn spec_013_show_a_scenario_with_its_evidence() {
    let dir = show_project();
    let mut test = "//! auth tests\n".to_string();
    for _ in 0..9 {
        test.push_str("//\n");
    }
    test.push_str("#[test]\nfn auth_003_rejects_expired_token() {}\n");
    put(dir.path(), "tests/auth.rs", &test);
    ledger(
        dir.path(),
        &[("t1", "AUTH-003", "failed"), ("t2", "AUTH-003", "passed")],
    );
    let out = stdout(dir.path(), &["spec", "show", "AUTH-003"]);
    for want in [
        "Token checks",
        "The system SHALL reject expired tokens.",
        "- **WHEN** a request carries a token that expired",
        "- **THEN** the API answers 401",
        "tests/auth.rs:12 auth_003_rejects_expired_token",
        "latest passed, red-before-green proven",
    ] {
        assert!(out.contains(want), "missing {want:?}:\n{out}");
    }
}

// spec: SPEC-014
#[test]
fn spec_014_show_a_change() {
    let dir = show_project();
    let out = stdout(dir.path(), &["spec", "show", "add-oauth-login"]);
    assert!(out.contains("Users want Google login."), "{out}");
    let header = out
        .lines()
        .position(|l| l.contains("auth") && l.starts_with("---"))
        .unwrap_or_else(|| panic!("no auth delta header:\n{out}"));
    let rest: String = out.lines().skip(header + 1).collect::<Vec<_>>().join("\n");
    assert!(
        rest.contains("#### Scenario: AUTH-009 Google login"),
        "{out}"
    );
    assert!(out.contains("3/7 tasks"), "{out}");
}

// spec: SPEC-015
#[test]
fn spec_015_show_a_capability_byte_for_byte() {
    let dir = show_project();
    sj(dir.path(), &["spec", "show", "auth"])
        .assert()
        .success()
        .stdout(AUTH);
}

// spec: SPEC-016
#[test]
fn spec_016_json_names_the_kind() {
    let dir = show_project();
    let v: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["spec", "show", "AUTH-003", "--json"])).unwrap();
    assert_eq!(v["kind"], "scenario");
    assert_eq!(v["id"], "AUTH-003");
    assert!(v["tests"].is_array(), "{v}");
    assert!(v.get("latest").is_some(), "{v}");

    let v: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["spec", "show", "auth", "--json"])).unwrap();
    assert_eq!(v["kind"], "capability");
    let v: serde_json::Value = serde_json::from_str(&stdout(
        dir.path(),
        &["spec", "show", "add-oauth-login", "--json"],
    ))
    .unwrap();
    assert_eq!(v["kind"], "change");
}

// spec: SPEC-017
#[test]
fn spec_017_unknown_item_suggests_near_matches() {
    let dir = show_project();
    let out = sj(dir.path(), &["spec", "show", "aut"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("auth"), "{all}");
    assert!(all.contains("audit"), "{all}");
}

// spec: SPEC-018
#[test]
fn spec_018_a_scenario_in_an_open_change_is_found() {
    let dir = show_project();
    let out = stdout(dir.path(), &["spec", "show", "AUTH-009"]);
    assert!(out.contains("- **WHEN** the user picks Google"), "{out}");
    assert!(out.contains("- **THEN** they are signed in"), "{out}");
    assert!(out.contains("add-oauth-login"), "{out}");
    assert!(out.contains("untested"), "{out}");
}

// spec: SPEC-019
#[test]
fn spec_019_show_an_archived_change_by_name() {
    let dir = show_project();
    let a = "openspec/changes/archive/2026-11-18-audit-checks";
    put(
        dir.path(),
        &format!("{a}/proposal.md"),
        "## Why\n\nCatch tampered tests.\n",
    );
    put(
        dir.path(),
        &format!("{a}/specs/audit/spec.md"),
        "## ADDED Requirements\n\n### Requirement: Tamper\nThe system SHALL flag it.\n\n\
         #### Scenario: AUDIT-002 Tamper\n- **WHEN** a\n- **THEN** b\n",
    );
    put(
        dir.path(),
        &format!("{a}/tasks.md"),
        "# Tasks\n\n- [x] 1.1 a (AUDIT-002)\n",
    );
    let by_name = stdout(dir.path(), &["spec", "show", "audit-checks"]);
    assert!(by_name.contains("Catch tampered tests."), "{by_name}");
    assert!(
        by_name.contains("#### Scenario: AUDIT-002 Tamper"),
        "{by_name}"
    );
    assert!(by_name.contains("archived 2026-11-18"), "{by_name}");
    let by_folder = stdout(dir.path(), &["spec", "show", "2026-11-18-audit-checks"]);
    assert_eq!(by_name, by_folder);

    // A bare name with several archived matches lists them and exits 1.
    put(
        dir.path(),
        "openspec/changes/archive/2026-11-19-audit-checks/proposal.md",
        "## Why\n\nAgain.\n",
    );
    let out = sj(dir.path(), &["spec", "show", "audit-checks"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("2026-11-18-audit-checks"), "{all}");
    assert!(all.contains("2026-11-19-audit-checks"), "{all}");
}

/// LOOP-008's project: auth (6), billing (4), `add-oauth-login` at 3/7 with AUTH-007..009
/// (007 and 008 proven, 009 untested), one orphan test and five archived changes.
fn view_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(r, "specjutsu.toml", CONFIG);
    put(r, "openspec/specs/auth/spec.md", &spec("auth", "AUTH", 6));
    put(
        r,
        "openspec/specs/billing/spec.md",
        &spec("billing", "BILL", 4),
    );
    let mut delta = String::from(
        "## ADDED Requirements\n\n### Requirement: OAuth\nThe system SHALL do OAuth.\n",
    );
    for i in 7..=9 {
        delta.push_str(&format!(
            "\n#### Scenario: AUTH-00{i} OAuth {i}\n- **WHEN** a\n- **THEN** b\n"
        ));
    }
    put(
        r,
        "openspec/changes/add-oauth-login/proposal.md",
        "## Why\n\nOAuth.\n",
    );
    put(
        r,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &delta,
    );
    put(r, "openspec/changes/add-oauth-login/tasks.md", TASKS);
    for d in 1..=5 {
        put(
            r,
            &format!("openspec/changes/archive/2026-01-0{d}-old-{d}/proposal.md"),
            "## Why\n",
        );
    }
    put(
        r,
        "tests/auth.rs",
        "#[test]\nfn auth_007_a() {}\n\n#[test]\nfn auth_008_b() {}\n\n#[test]\nfn auth_099_gone() {}\n",
    );
    ledger(
        r,
        &[
            ("t1", "AUTH-007", "failed"),
            ("t1", "AUTH-008", "failed"),
            ("t2", "AUTH-007", "passed"),
            ("t2", "AUTH-008", "passed"),
        ],
    );
    dir
}

// spec: LOOP-008
#[test]
fn loop_008_dashboard_shows_changes_and_totals() {
    let dir = view_project();
    let out = stdout(dir.path(), &["view"]);
    assert!(out.contains("auth  6 scenarios"), "{out}");
    assert!(out.contains("billing  4 scenarios"), "{out}");
    let change = out
        .lines()
        .find(|l| l.contains("add-oauth-login"))
        .unwrap_or_else(|| panic!("no change line:\n{out}"));
    for want in ["3/7 tasks", "2 proven", "1 untested"] {
        assert!(change.contains(want), "missing {want:?}: {change}");
    }
    assert!(out.contains("1 orphan test"), "{out}");
    assert!(out.contains("5 archived"), "{out}");
}

// spec: LOOP-009
#[test]
fn loop_009_long_lists_are_cut_to_fit() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..30 {
        let cap = format!("cap{i:02}");
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            &spec(&cap, &format!("C{i:02}X"), 1),
        );
    }
    let out = stdout(dir.path(), &["view"]);
    let caps = out
        .lines()
        .filter(|l| l.contains(" scenario"))
        .filter(|l| l.contains("cap"))
        .count();
    assert_eq!(caps, 20, "{out}");
    assert!(out.contains("… +10 more"), "{out}");
    assert!(
        out.lines().count() <= 40,
        "{} lines:\n{out}",
        out.lines().count()
    );
}

// spec: LOOP-010
#[test]
fn loop_010_json_dashboard() {
    let dir = view_project();
    let v: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["view", "--json"])).unwrap();
    for key in ["capabilities", "changes", "totals"] {
        assert!(v.get(key).is_some(), "missing {key}: {v}");
    }
    assert_eq!(v["orphans"], 1);
    assert_eq!(v["archived"], 5);
}

// spec: LOOP-011
#[test]
fn loop_011_view_never_runs_tests() {
    let dir = view_project();
    let ledger_file = dir.path().join(".specjutsu/ledger/runs.jsonl");
    let before = std::fs::read(&ledger_file).unwrap();
    // An empty PATH: starting cargo or any other runner would fail.
    let empty = tempfile::tempdir().unwrap();
    sj(dir.path(), &["view"])
        .env("PATH", empty.path())
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("add-oauth-login"));
    assert_eq!(std::fs::read(&ledger_file).unwrap(), before);
}
