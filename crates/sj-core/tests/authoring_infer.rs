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
}
