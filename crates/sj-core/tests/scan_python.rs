//! TRACE-013 and TRACE-014 from openspec/changes/polyglot-trace: pytest tests.

use std::path::Path;

use sj_core::spec::parse_scenarios;
use sj_core::testscan::{scan_python, TestFn};
use sj_core::trace::build_report;

fn scan(src: &str) -> Vec<TestFn> {
    scan_python(Path::new("tests/test_inventory.py"), src)
}

fn names(tests: &[TestFn]) -> Vec<&str> {
    tests.iter().map(|t| t.name.as_str()).collect()
}

fn spec(ids: &[&str]) -> Vec<sj_core::spec::Scenario> {
    let mut md = String::from("### Requirement: R\nThe system SHALL work.\n\n");
    for id in ids {
        md.push_str(&format!(
            "#### Scenario: {id} Case\n- **WHEN** x\n- **THEN** y\n\n"
        ));
    }
    parse_scenarios(Path::new("spec.md"), &md)
}

#[test]
fn trace_013_id_in_a_pytest_function_name() {
    let src = "import pytest\n\n\ndef test_inv_002_removes_item():\n    assert True\n\n\
               def helper_inv_003():\n    pass\n\n\
               async def test_INV_005_async(client):\n    await client.get('/')\n";
    let tests = scan(src);
    assert_eq!(
        names(&tests),
        ["test_inv_002_removes_item", "test_INV_005_async"]
    );
    assert_eq!(tests[0].line, 4);
    assert_eq!(tests[0].name_ids, ["INV-002"]);
    assert_eq!(tests[1].name_ids, ["INV-005"]);

    let r = build_report(&spec(&["INV-002", "INV-005"]), &tests);
    assert_eq!(r.mapped.len(), 2);
    assert_eq!(r.mapped[0].tests[0].name, "test_inv_002_removes_item");
}

#[test]
fn trace_014_pytest_marker_and_class_level_comment() {
    let src = r#"
import pytest

# spec: INV-003
class TestDiff:
    """Diffs."""

    @pytest.mark.spec("INV-004")
    def test_added(self):
        assert True

    @pytest.mark.parametrize("x", [1, 2])
    @pytest.mark.spec("INV-006", "INV-007")
    # spec: INV-008
    def test_many(self, x):
        pass

    def helper(self):
        def test_nested_inner():
            pass


class Helpers:
    def test_not_collected(self):
        pass


@pytest.mark.skip(reason="later")
def test_skipped():
    pass


@pytest.mark.skipif(True, reason="no")
def test_conditionally_skipped():
    pass
"#;
    let tests = scan(src);
    assert_eq!(
        names(&tests),
        [
            "TestDiff::test_added",
            "TestDiff::test_many",
            "test_skipped",
            "test_conditionally_skipped"
        ]
    );
    assert_eq!(tests[0].comment_ids, ["INV-003", "INV-004"]);
    assert_eq!(
        tests[1].comment_ids,
        ["INV-003", "INV-006", "INV-007", "INV-008"]
    );
    assert!(!tests[0].skipped);
    assert!(tests[2].skipped);
    assert!(tests[3].skipped);

    let r = build_report(&spec(&["INV-003", "INV-004"]), &tests[..1]);
    let mapped: Vec<_> = r.mapped.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(mapped, ["INV-003", "INV-004"]);
    assert_eq!(r.mapped[0].tests[0].name, "TestDiff::test_added");
}

#[test]
fn trace_014_class_marker_applies_to_methods() {
    let src = "import pytest\n\n@pytest.mark.spec(\"INV-010\")\nclass TestCart:\n    \
               def test_a(self):\n        pass\n\n    async def test_b(self):\n        pass\n";
    let tests = scan(src);
    assert_eq!(names(&tests), ["TestCart::test_a", "TestCart::test_b"]);
    assert!(tests.iter().all(|t| t.comment_ids == ["INV-010"]));
}

#[test]
fn trace_014_brackets_inside_decorator_strings_do_not_hide_later_tests() {
    let src = r#"
@pytest.mark.parametrize(
    "body",
    [b"{not json", b"[]", json.dumps({"a": "(x"}).encode()],  # a ] comment
    ids=["not-json", "not-object", "no-request-id"],
)
def test_malformed(body):
    pass


def test_after():
    pass
"#;
    assert_eq!(names(&scan(src)), ["test_malformed", "test_after"]);
}
