//! TRACE-009..012 from openspec/changes/polyglot-trace: vitest, Jest and Playwright tests.

use std::path::Path;

use sj_core::spec::parse_scenarios;
use sj_core::testscan::{scan_js, TestFn};
use sj_core::trace::build_report;

fn scan(src: &str) -> Vec<TestFn> {
    scan_js(Path::new("a.test.ts"), src)
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
fn trace_009_id_in_an_it_title() {
    let src = "import { it } from 'vitest'\n\n\
               it(\"AUTH-003 rejects expired token\", () => {\n  expect(1).toBe(1)\n})\n";
    let tests = scan(src);
    assert_eq!(names(&tests), ["AUTH-003 rejects expired token"]);
    assert_eq!(tests[0].line, 3);
    assert_eq!(tests[0].name_ids, ["AUTH-003"]);
    assert!(!tests[0].skipped);

    let r = build_report(&spec(&["AUTH-003"]), &tests);
    assert_eq!(r.mapped.len(), 1);
    assert_eq!(r.mapped[0].tests[0].name, "AUTH-003 rejects expired token");
    assert_eq!(r.mapped[0].tests[0].file, Path::new("a.test.ts"));
    assert_eq!(r.mapped[0].tests[0].line, 3);
}

#[test]
fn trace_010_id_in_a_describe_title_applies_to_every_test_inside() {
    let src = r#"
describe("AUTH-004 refresh", () => {
  it("issues a new token", () => {})
  it('revokes the old one', async () => {
    await revoke()
  })
})
it("outside", () => {})
"#;
    let tests = scan(src);
    assert_eq!(
        names(&tests),
        [
            "AUTH-004 refresh > issues a new token",
            "AUTH-004 refresh > revokes the old one",
            "outside"
        ]
    );
    assert_eq!(tests[0].name_ids, ["AUTH-004"]);
    assert_eq!(tests[1].name_ids, ["AUTH-004"]);
    assert!(tests[2].name_ids.is_empty());
    assert_eq!(
        build_report(&spec(&["AUTH-004"]), &tests).mapped[0]
            .tests
            .len(),
        2
    );
}

#[test]
fn trace_010_nested_and_playwright_describes() {
    let src = r#"
test.describe("CHAT-001 chat", () => {
  test.describe.serial(`history`, () => {
    test("loads", async ({ page }) => {})
  })
})
describe.each([[1], [2]])("UNIT-001 table %s", (n) => {
  it.each([1, 2])("case %d", (x) => {})
})
"#;
    let tests = scan(src);
    assert_eq!(
        names(&tests),
        [
            "CHAT-001 chat > history > loads",
            "UNIT-001 table %s > case %d"
        ]
    );
    assert_eq!(tests[0].name_ids, ["CHAT-001"]);
    assert_eq!(tests[1].name_ids, ["UNIT-001"]);
}

#[test]
fn trace_011_spec_comment_above_a_playwright_test() {
    let src = r#"
import { test } from '@playwright/test'

// spec: CHAT-002, CHAT-003
test("sends a message", async ({ page }) => {
  await page.goto('/')
})

// spec: CHAT-009
test.describe("group", () => {
  // a note
  test.only("inner", async () => {})
})
"#;
    let tests = scan(src);
    assert_eq!(names(&tests), ["sends a message", "group > inner"]);
    assert_eq!(tests[0].comment_ids, ["CHAT-002", "CHAT-003"]);
    assert_eq!(tests[1].comment_ids, ["CHAT-009"]);
    let r = build_report(&spec(&["CHAT-002", "CHAT-003"]), &tests[..1]);
    assert_eq!(r.mapped.len(), 2);
}

#[test]
fn trace_012_skipped_tests_are_found_and_marked() {
    let src = r#"
it.skip("AUTH-005 pending case", () => {})
xit("AUTH-006 x", () => {})
it.todo("AUTH-007 later")
describe.skip("off", () => { it("inner", () => {}) })
test.fixme("broken", async () => {})
it.only("focused", () => {})
"#;
    let tests = scan(src);
    let skipped: Vec<_> = tests.iter().map(|t| (t.name.as_str(), t.skipped)).collect();
    assert_eq!(
        skipped,
        [
            ("AUTH-005 pending case", true),
            ("AUTH-006 x", true),
            ("AUTH-007 later", true),
            ("off > inner", true),
            ("broken", true),
            ("focused", false),
        ]
    );
    let r = build_report(&spec(&["AUTH-005"]), &tests);
    assert!(r.mapped[0].tests[0].skipped);
}

#[test]
fn trace_012_non_tests_are_ignored() {
    let src = r#"
// it("commented out", () => {})
/* test("block comment", () => {}) */
const re = /it\("in a regex"\)/g
const s = "it('in a string')"
const t = `test("in a template ${x}")`
foo.it("method on another object", () => {})
it(name, () => {})
it(`dynamic ${x}`, () => {})
test.skip()
function Button() { return <div>it("jsx text")</div> }
const ok = items.filter((x) => x.test("y"))
it("real", () => { const r = /}/; const q = '}'; })
it("after", () => {})
"#;
    assert_eq!(names(&scan(src)), ["real", "after"]);
}
