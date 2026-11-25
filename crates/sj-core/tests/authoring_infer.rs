//! SPEC-052 from specjutsu/changes/specs-from-tests: scenario headings from test names.

use sj_core::authoring::infer::heading_from_test;
use sj_core::config::Runner;

#[test]
fn spec_052_names_from_rust_pytest_and_js_become_headings() {
    assert_eq!(
        heading_from_test("refund_under_100_auto_approves", Some(Runner::Cargo)),
        "Refund under 100 auto approves"
    );
    assert_eq!(
        heading_from_test("TestRefunds::test_partial_refund", Some(Runner::Pytest)),
        "Refunds partial refund"
    );
    assert_eq!(
        heading_from_test("Refunds > auto-approves under $100", Some(Runner::Vitest)),
        "Refunds auto-approves under $100"
    );
    assert_eq!(
        heading_from_test("testimony_is_recorded", Some(Runner::Cargo)),
        "Testimony is recorded"
    );
    assert_eq!(
        heading_from_test("TestimonyRecords::test_saved", Some(Runner::Pytest)),
        "TestimonyRecords saved"
    );
    assert_eq!(
        heading_from_test("TestOuter::TestInner::test_x", Some(Runner::Pytest)),
        "Outer Inner x"
    );
    assert_eq!(heading_from_test("test_", Some(Runner::Cargo)), "Test");
}
