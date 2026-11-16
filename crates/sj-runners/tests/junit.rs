//! RUN-011 and RUN-013 from openspec/changes/polyglot-runners: reading JUnit reports.
//! The samples in tests/data/ are real reports from vitest 5, pytest and Playwright.

use sj_runners::junit::{parse, Case, CaseOutcome};

fn case<'a>(cases: &'a [Case], name: &str) -> &'a Case {
    cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"))
}

#[test]
fn run_011_vitest_report() {
    let cases = parse(include_str!("data/vitest.xml")).unwrap();
    assert_eq!(cases.len(), 6);
    let ok = case(&cases, "AUTH-004 refresh > issues a new token");
    assert_eq!(ok.classname, "src/auth.test.ts");
    assert_eq!(ok.outcome, CaseOutcome::Passed);
    let bad = case(&cases, "AUTH-004 refresh > rejects expired token");
    assert_eq!(bad.outcome, CaseOutcome::Failed);
    assert!(
        bad.message.contains("expected 200 to be 401"),
        "{}",
        bad.message
    );
    assert!(
        bad.message.contains("src/auth.test.ts:3:51"),
        "{}",
        bad.message
    );
    assert_eq!(
        case(&cases, "AUTH-005 pending").outcome,
        CaseOutcome::Skipped
    );
    assert_eq!(case(&cases, "case 2").outcome, CaseOutcome::Passed);
}

#[test]
fn run_013_vitest_file_that_fails_to_load() {
    let cases = parse(include_str!("data/vitest.xml")).unwrap();
    let broken = case(&cases, "src/broken.test.ts");
    assert_eq!(broken.classname, "src/broken.test.ts");
    assert_eq!(broken.outcome, CaseOutcome::Failed);
    assert!(
        broken.message.contains("Cannot find module './nope'"),
        "{}",
        broken.message
    );
}

#[test]
fn run_011_pytest_reports() {
    for xml in [
        include_str!("data/pytest-xunit2.xml"),
        include_str!("data/pytest-xunit1.xml"),
    ] {
        let cases = parse(xml).unwrap();
        assert_eq!(cases.len(), 5);
        let bad = case(&cases, "test_inv_002_removes_item");
        assert_eq!(bad.classname, "tests.test_inv");
        assert_eq!(bad.outcome, CaseOutcome::Failed);
        assert!(
            bad.message.contains("expected 401, got 200"),
            "{}",
            bad.message
        );
        assert_eq!(
            case(&cases, "test_added").classname,
            "tests.test_inv.TestDiff"
        );
        assert_eq!(case(&cases, "test_param[2]").outcome, CaseOutcome::Passed);
        assert_eq!(case(&cases, "test_skipped").outcome, CaseOutcome::Skipped);
    }
    let x1 = parse(include_str!("data/pytest-xunit1.xml")).unwrap();
    let added = case(&x1, "test_added");
    assert_eq!(added.file.as_deref(), Some("tests/test_inv.py"));
    assert_eq!(added.line, Some(6));
}

#[test]
fn run_011_playwright_report() {
    let cases = parse(include_str!("data/playwright.xml")).unwrap();
    assert_eq!(cases.len(), 3);
    let bad = case(&cases, "CHAT-001 chat › fails");
    assert_eq!(bad.classname, "e2e/chat.spec.ts");
    assert_eq!(bad.outcome, CaseOutcome::Failed);
    assert!(bad.message.contains("Received: 200"), "{}", bad.message);
    assert_eq!(case(&cases, "later").outcome, CaseOutcome::Skipped);
    assert_eq!(
        case(&cases, "CHAT-001 chat › sends a message").outcome,
        CaseOutcome::Passed
    );
}

#[test]
fn run_013_a_broken_report_is_an_error() {
    assert!(parse("<testsuites><testsuite><testcase name=\"x\"").is_err());
    assert!(parse("not xml at all").is_err());
    assert!(parse("").is_err());
}

#[test]
fn run_011_errors_count_as_failures() {
    let xml = r#"<testsuite><testcase classname="" name="tests.test_x"><error message="collection failure">ImportError: no module named x</error></testcase></testsuite>"#;
    let cases = parse(xml).unwrap();
    assert_eq!(cases[0].outcome, CaseOutcome::Failed);
    assert!(cases[0].message.contains("collection failure"));
    assert!(cases[0].message.contains("ImportError"));
}

#[test]
fn run_011_a_message_repeated_in_the_body_appears_once() {
    let cases = parse(include_str!("data/vitest.xml")).unwrap();
    let bad = case(&cases, "AUTH-004 refresh > rejects expired token");
    assert_eq!(
        bad.message.matches("expected 200 to be 401").count(),
        1,
        "{}",
        bad.message
    );
    let xml = r#"<testsuite><testcase name="t"><failure message="boom"/></testcase>
<testcase name="u"><failure message="short">longer body text</failure></testcase></testsuite>"#;
    let cases = parse(xml).unwrap();
    assert_eq!(cases[0].message, "boom");
    assert_eq!(cases[1].message, "short\nlonger body text");
}
