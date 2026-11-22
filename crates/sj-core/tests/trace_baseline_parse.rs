//! `Baseline::parse` reads back what `Baseline::render` writes.

use sj_core::trace::{Baseline, BaselineEntry};

#[test]
fn parse_skips_blank_lines_and_crlf() {
    let b = Baseline::parse("gap AUTH-002\r\n\r\n\norphan AUTH-009 tests/auth.rs auth_009_old\r\n")
        .unwrap();
    let expected: std::collections::BTreeSet<_> = [
        BaselineEntry::Gap("AUTH-002".into()),
        BaselineEntry::Orphan(
            "AUTH-009".into(),
            "tests/auth.rs".into(),
            "auth_009_old".into(),
        ),
    ]
    .into();
    assert_eq!(b.entries, expected);
}

#[test]
fn parse_keeps_spaces_in_an_orphan_test_name() {
    let b = Baseline::parse("orphan AUTH-009 src/auth.test.ts AUTH-009 logs in twice\n").unwrap();
    assert_eq!(
        b.entries.into_iter().collect::<Vec<_>>(),
        [BaselineEntry::Orphan(
            "AUTH-009".into(),
            "src/auth.test.ts".into(),
            "AUTH-009 logs in twice".into(),
        )]
    );
}

#[test]
fn parse_rejects_an_unrecognised_line_naming_its_number() {
    let err = Baseline::parse("gap AUTH-002\n\nmissing AUTH-003\n").unwrap_err();
    assert!(err.to_string().contains("line 3"), "{err}");
}

#[test]
fn parse_round_trips_render() {
    let text = "gap AUTH-002\ngap AUTH-005\norphan AUTH-009 tests/auth.rs auth_009_old\n";
    assert_eq!(Baseline::parse(text).unwrap().render(), text);
}
