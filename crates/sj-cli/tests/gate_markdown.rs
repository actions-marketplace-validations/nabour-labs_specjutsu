//! CI scenarios: the gate rendered as a PR comment (specjutsu/changes/gate-action).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const MUL: &str = "pub fn mul(a: i32, b: i32) -> i32 { a * b }\n";
const ADD_TEST: &str = "#[test]\nfn auth_004_adds() { assert_eq!(demo::add(2, 2), 4); }\n";
const MUL_TEST: &str = "#[test]\nfn auth_005_multiplies() { assert_eq!(demo::mul(2, 3), 6); }\n";

fn markdown(repo: &Repo) -> (Option<i32>, String) {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--root"])
        .arg(repo.path())
        .args(["--base", "main", "--format", "markdown"])
        .output()
        .unwrap();
    (out.status.code(), String::from_utf8(out.stdout).unwrap())
}

fn row<'a>(body: &'a str, id: &str) -> &'a str {
    body.lines()
        .find(|l| l.starts_with('|') && l.contains(id))
        .unwrap_or_else(|| panic!("no table row for {id} in:\n{body}"))
}

#[test]
fn ci_001_passing_gate_renders_a_short_comment() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.branch("feature");
    repo.add_change("auth", &[("AUTH-004", "Add"), ("AUTH-005", "Multiply")]);
    repo.write("src/lib.rs", &format!("{ADD_OK}{MUL}"));
    repo.write("tests/add.rs", ADD_TEST);
    repo.write("tests/mul.rs", MUL_TEST);
    repo.commit("feat(AUTH-004, AUTH-005): green");

    let (code, body) = markdown(&repo);
    assert_eq!(code, Some(0), "{body}");
    assert!(body.starts_with("<!-- specjutsu:gate -->\n"), "{body}");
    assert!(
        body.contains("✅ gate: pass — 2 scenarios proven"),
        "{body}"
    );
    let rows: Vec<&str> = body.lines().filter(|l| l.starts_with("| AUTH-")).collect();
    assert_eq!(rows.len(), 2, "{body}");
    assert!(row(&body, "AUTH-004").contains("red:assert"), "{body}");
    assert!(row(&body, "AUTH-004").contains("auth_004_adds"), "{body}");
    assert!(row(&body, "AUTH-005").contains("red:compile"), "{body}");
}

#[test]
fn ci_002_failing_gate_lists_each_reason_with_a_fix_hint() {
    let repo = Repo::with_lib(&format!("{ADD_OK}{MUL}"));
    repo.branch("feature");
    repo.add_change("auth", &[("AUTH-004", "Add"), ("AUTH-005", "Multiply")]);
    repo.write("tests/mul.rs", MUL_TEST);
    repo.commit("test(AUTH-005): red");

    let (code, body) = markdown(&repo);
    assert_eq!(code, Some(1), "{body}");
    assert!(body.contains("❌ gate: fail"), "{body}");
    let r4 = row(&body, "AUTH-004");
    assert!(r4.contains("untested"), "{r4}");
    assert!(r4.contains("add a test that names AUTH-004"), "{r4}");
    let r5 = row(&body, "AUTH-005");
    assert!(r5.contains("green-on-base"), "{r5}");
    assert!(
        r5.contains("the test passes without your change; make it assert the new behaviour"),
        "{r5}"
    );
}

#[test]
fn ci_003_a_very_large_result_is_truncated() {
    let repo = Repo::with_lib(ADD_OK);
    repo.branch("feature");
    let ids: Vec<(String, String)> = (1..=2000)
        .map(|n| {
            (
                format!("BIG-{n:04}"),
                format!("Scenario number {n} with a long-ish name"),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = ids.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    repo.add_change("big", &refs);
    repo.commit("docs: propose big");

    let (code, body) = markdown(&repo);
    assert_eq!(code, Some(1));
    assert!(body.len() <= 60_000, "body is {} bytes", body.len());
    let first_row = body.lines().find(|l| l.starts_with("| BIG-")).unwrap();
    assert!(first_row.contains("untested"), "{first_row}");
    let last = body.trim_end().lines().last().unwrap();
    let shown = body.lines().filter(|l| l.starts_with("| BIG-")).count();
    assert!(shown < 2000);
    assert!(
        last.contains(&format!("{} more rows not shown", 2000 - shown)),
        "{last}"
    );
    assert!(last.contains("run `sj gate"), "{last}");
}
