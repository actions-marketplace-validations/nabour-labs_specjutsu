//! AUDIT-013 from openspec/changes/audit-checks: `sj gate` includes the audit's test checks.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_TEST: &str = "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n";
const TRACE_TEST: &str = "#[test]\nfn trace_004_matches_ids() {\n    assert_eq!(1 + 1, 2);\n}\n";

#[test]
fn audit_013_skipping_a_test_fails_the_gate() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.write("tests/trace.rs", TRACE_TEST);
    repo.commit("test: trace");
    repo.branch("feature");
    repo.add_change("math", &[("MATH-001", "Add")]);
    repo.write("src/lib.rs", ADD_OK);
    repo.write("tests/add.rs", ADD_TEST);
    repo.write(
        "tests/trace.rs",
        &TRACE_TEST.replace("#[test]\n", "#[test]\n#[ignore]\n"),
    );
    repo.commit("feat(MATH-001): green");

    let out = Command::cargo_bin("sj")
        .unwrap()
        .arg("gate")
        .arg("--root")
        .arg(repo.path())
        .args(["--base", "main"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(stdout.starts_with("gate: fail — 1 problem\n"), "{stdout}");
    assert!(
        stdout.contains("fail    skipped-test tests/trace.rs › trace_004_matches_ids"),
        "{stdout}"
    );
}
