//! RUN-011 and RUN-013 from openspec/changes/polyglot-runners: joining JUnit cases to traced tests.

use std::path::PathBuf;

use sj_core::config::Runner;
use sj_core::run::suite::{match_cases, Joined, JoinedOutcome};
use sj_core::testscan::TestFn;
use sj_runners::junit::parse;

fn traced(file: &str, names: &[&str]) -> Vec<TestFn> {
    names
        .iter()
        .map(|n| TestFn {
            name: n.to_string(),
            file: PathBuf::from(file),
            line: 1,
            ..TestFn::default()
        })
        .collect()
}

fn outcome_of(joined: &[Joined], tests: &[TestFn], name: &str) -> Option<JoinedOutcome> {
    let idx = tests.iter().position(|t| t.name == name).unwrap();
    joined
        .iter()
        .find(|j| j.test == Some(idx))
        .map(|j| j.outcome)
}

#[test]
fn run_011_vitest_cases_join_by_file_suffix_and_titles() {
    let cases = parse(include_str!("../../sj-runners/tests/data/vitest.xml")).unwrap();
    let mut tests = traced(
        "apps/api/src/auth.test.ts",
        &[
            "AUTH-004 refresh > issues a new token",
            "AUTH-004 refresh > rejects expired token",
            "AUTH-005 pending",
            "case %i",
        ],
    );
    tests.extend(traced("apps/api/src/broken.test.ts", &["uses it"]));
    let joined = match_cases(Runner::Vitest, &tests, &cases);

    use JoinedOutcome::*;
    assert_eq!(
        outcome_of(&joined, &tests, "AUTH-004 refresh > issues a new token"),
        Some(Passed)
    );
    assert_eq!(
        outcome_of(&joined, &tests, "AUTH-005 pending"),
        Some(Skipped)
    );
    // Two expanded .each cases aggregate into one traced test.
    assert_eq!(outcome_of(&joined, &tests, "case %i"), Some(Passed));
    assert_eq!(joined.iter().filter(|j| j.test == Some(3)).count(), 1);

    let bad = joined.iter().find(|j| j.test == Some(1)).unwrap();
    assert_eq!(bad.outcome, Failed);
    assert_eq!(
        bad.path,
        "apps/api/src/auth.test.ts::AUTH-004 refresh > rejects expired token"
    );
    assert!(bad
        .message
        .as_deref()
        .unwrap()
        .contains("expected 200 to be 401"));

    // RUN-013: the file that failed to load turns its tests into build failures.
    let b = joined.iter().find(|j| j.test == Some(4)).unwrap();
    assert_eq!(b.outcome, BuildFailed);
    assert!(b.message.as_deref().unwrap().contains("Cannot find module"));
    // Every case was claimed: nothing left over.
    assert!(joined.iter().all(|j| j.test.is_some()), "{joined:?}");
}

#[test]
fn run_011_unmatched_cases_are_still_reported() {
    let cases = parse(include_str!("../../sj-runners/tests/data/vitest.xml")).unwrap();
    let tests = traced("apps/api/src/auth.test.ts", &["AUTH-005 pending"]);
    let joined = match_cases(Runner::Vitest, &tests, &cases);
    let loose: Vec<_> = joined
        .iter()
        .filter(|j| j.test.is_none())
        .map(|j| j.path.as_str())
        .collect();
    assert!(
        loose.contains(&"src/auth.test.ts::AUTH-004 refresh > rejects expired token"),
        "{loose:?}"
    );
    assert!(loose.contains(&"src/auth.test.ts::case 1"), "{loose:?}");
    // A file-level error with no traced test in it is a build failure of its own.
    let broken = joined
        .iter()
        .find(|j| j.path.starts_with("src/broken.test.ts"))
        .unwrap();
    assert_eq!(broken.outcome, JoinedOutcome::BuildFailed);
}

#[test]
fn run_011_pytest_cases_join_by_module_class_and_parameters() {
    for xml in [
        include_str!("../../sj-runners/tests/data/pytest-xunit2.xml"),
        include_str!("../../sj-runners/tests/data/pytest-xunit1.xml"),
    ] {
        let cases = parse(xml).unwrap();
        let tests = traced(
            "backend/tests/test_inv.py",
            &[
                "test_inv_002_removes_item",
                "TestDiff::test_added",
                "test_param",
                "test_skipped",
            ],
        );
        let joined = match_cases(Runner::Pytest, &tests, &cases);
        use JoinedOutcome::*;
        assert_eq!(
            outcome_of(&joined, &tests, "test_inv_002_removes_item"),
            Some(Failed)
        );
        assert_eq!(
            outcome_of(&joined, &tests, "TestDiff::test_added"),
            Some(Passed)
        );
        assert_eq!(outcome_of(&joined, &tests, "test_param"), Some(Passed));
        assert_eq!(outcome_of(&joined, &tests, "test_skipped"), Some(Skipped));
        assert!(joined.iter().all(|j| j.test.is_some()), "{joined:?}");
    }
}

#[test]
fn run_011_playwright_cases_join_across_its_separator() {
    let cases = parse(include_str!("../../sj-runners/tests/data/playwright.xml")).unwrap();
    let tests = traced(
        "apps/app/e2e/chat.spec.ts",
        &[
            "CHAT-001 chat > sends a message",
            "CHAT-001 chat > fails",
            "later",
        ],
    );
    let joined = match_cases(Runner::Playwright, &tests, &cases);
    use JoinedOutcome::*;
    assert_eq!(
        outcome_of(&joined, &tests, "CHAT-001 chat > sends a message"),
        Some(Passed)
    );
    assert_eq!(
        outcome_of(&joined, &tests, "CHAT-001 chat > fails"),
        Some(Failed)
    );
    assert_eq!(outcome_of(&joined, &tests, "later"), Some(Skipped));
}

#[test]
fn run_011_same_title_in_another_file_does_not_match() {
    let cases = parse(include_str!("../../sj-runners/tests/data/vitest.xml")).unwrap();
    let tests = traced("apps/api/src/other.test.ts", &["AUTH-005 pending"]);
    let joined = match_cases(Runner::Vitest, &tests, &cases);
    assert!(joined.iter().all(|j| j.test.is_none()));
}
