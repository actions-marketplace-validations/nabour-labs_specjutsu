//! GATE-039..043 (openspec/changes/bugfix-changes): `[keep]` scenarios must pass on base and at head.

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;

use sj_core::ci::render_markdown;
use sj_core::gate::{evaluate, run, Change, Input, Options, Red};
use sj_core::trace::{Mapped, Report, TestRef};
use sj_runners::cargo::{Outcome, Run, TestResult};
use support::Repo;

fn tref(name: &str) -> TestRef {
    TestRef {
        name: name.into(),
        file: PathBuf::from("tests/pay.rs"),
        line: 1,
        ..TestRef::default()
    }
}

fn head(results: &[(&str, Outcome)]) -> Option<Run> {
    Some(Run {
        success: results.iter().all(|(_, o)| *o != Outcome::Failed),
        results: results
            .iter()
            .map(|(p, o)| TestResult {
                path: p.to_string(),
                outcome: *o,
            })
            .collect(),
    })
}

/// PAY-012 added as a keep scenario with one test, `pay_012_small_refunds`.
fn keep_input(base: Red, at_head: Outcome) -> Input {
    let mut i = Input {
        base_branch: "main".into(),
        spec_root: "openspec".into(),
        ..Input::default()
    };
    i.changes.insert("PAY-012".into(), Change::Added);
    i.keep.insert("PAY-012".into());
    i.trace = Report::default();
    i.trace.mapped.push(Mapped {
        id: "PAY-012".into(),
        name: "Refunds under $100 auto-approve".into(),
        tests: vec![tref("pay_012_small_refunds")],
    });
    i.head_run = head(&[("tests::pay_012_small_refunds", at_head)]);
    i.base = BTreeMap::from([(("PAY-012".into(), "pay_012_small_refunds".into()), base)]);
    i
}

#[test]
fn gate_039_keep_test_green_on_base_and_head_is_accepted() {
    let r = evaluate(&keep_input(Red::Green, Outcome::Passed));
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(r.proven.len(), 1);
    assert_eq!(r.proven[0].id, "PAY-012");
    assert_eq!(r.proven[0].red, Red::Keep);
    assert_eq!(r.rows[0].evidence, "green:keep");
}

#[test]
fn gate_039_keep_scenario_end_to_end() {
    let repo = Repo::with_lib("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
    repo.branch("feature");
    repo.add_change("keep-math", &[("MATH-001", "[keep] Adds numbers")]);
    repo.write(
        "tests/math.rs",
        "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n",
    );
    repo.commit("test(MATH-001): keep");
    let r = run(&Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    })
    .unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert!(
        r.proven
            .iter()
            .any(|p| p.id == "MATH-001" && p.red == Red::Keep),
        "{:?}",
        r.proven
    );
}

#[test]
fn gate_040_keep_test_red_on_base_fails() {
    let r = evaluate(&keep_input(Red::Assert, Outcome::Passed));
    assert!(!r.pass);
    let f = r
        .failures
        .iter()
        .find(|f| f.starts_with("keep-red-on-base PAY-012 pay_012_small_refunds"))
        .unwrap_or_else(|| panic!("{:?}", r.failures));
    assert!(f.contains("new behaviour"), "{f}");
    assert_eq!(r.rows[0].evidence, "keep-red-on-base");
}

#[test]
fn gate_041_keep_test_failing_at_head_fails() {
    let r = evaluate(&keep_input(Red::Green, Outcome::Failed));
    assert!(!r.pass);
    assert!(
        r.failures
            .iter()
            .any(|f| f == "keep-broken PAY-012 pay_012_small_refunds"),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_042_keep_scenario_without_a_test_fails() {
    let mut i = keep_input(Red::Green, Outcome::Passed);
    i.trace.mapped.clear();
    let r = evaluate(&i);
    assert!(!r.pass);
    assert!(r.failures.iter().any(|f| f == "untested PAY-012"), "{:?}", r.failures);
}

#[test]
fn gate_043_keep_evidence_in_json_and_markdown() {
    let mut i = keep_input(Red::Green, Outcome::Passed);
    i.changes.insert("PAY-013".into(), Change::Added);
    i.trace.mapped.push(Mapped {
        id: "PAY-013".into(),
        name: "Rounds half up".into(),
        tests: vec![tref("pay_013_rounds_half_up")],
    });
    i.head_run = head(&[
        ("tests::pay_012_small_refunds", Outcome::Passed),
        ("tests::pay_013_rounds_half_up", Outcome::Passed),
    ]);
    i.base.insert(
        ("PAY-013".into(), "pay_013_rounds_half_up".into()),
        Red::Assert,
    );
    let r = evaluate(&i);
    assert!(r.pass, "{:?}", r.failures);
    let json = serde_json::to_value(&r).unwrap();
    let proven = json["proven"].as_array().unwrap();
    assert!(proven
        .iter()
        .any(|p| p["id"] == "PAY-012" && p["red"] == "green:keep"));
    assert!(proven
        .iter()
        .any(|p| p["id"] == "PAY-013" && p["red"] == "red:assert"));

    let md = render_markdown(&r, "origin/main");
    let kept = md.find("**Kept behaviour**").unwrap_or_else(|| panic!("{md}"));
    let table = md.find("| Scenario |").unwrap_or_else(|| panic!("{md}"));
    let kept_section = &md[kept..];
    assert!(kept_section.contains("PAY-012"), "{md}");
    let table_section = &md[table..kept.max(table)];
    assert!(table_section.contains("PAY-013"), "{md}");
    assert!(!table_section.contains("PAY-012"), "{md}");
}
