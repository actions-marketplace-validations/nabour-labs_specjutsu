//! `sj spec infer` dry run (specjutsu/changes/specs-from-tests).

use std::path::Path;
use std::process::Command as Std;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = Std::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(["spec", "infer"]).args(args).arg("--root").arg(root);
    c
}

const CONFIG: &str = r#"schema = 1

[[package]]
path = "."
sources = ["src/**"]

[[package.suite]]
runner = "cargo"
tests = ["tests/**/*.rs"]

[[package.suite]]
runner = "vitest"
tests = ["tests/**/*.test.ts"]
"#;

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "specjutsu.toml", CONFIG);
    dir
}

#[test]
fn spec_048_a_dry_run_lists_the_drafts_and_writes_nothing() {
    let dir = project();
    let r = dir.path();
    put(
        r,
        "tests/refunds.rs",
        "#[test]\nfn refund_under_100_auto_approves() {}\n\n\
         #[test]\nfn refund_over_limit_needs_review() {}\n",
    );
    // Untraced, but outside the target path.
    put(r, "tests/other.rs", "#[test]\nfn other_thing_works() {}\n");
    git(r, &["init", "-q"]);
    git(r, &["add", "."]);
    git(
        r,
        &["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false", "commit", "-qm", "init"],
    );

    let out = sj(r, &["tests/refunds.rs", "--capability", "refunds"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        "REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds.rs:2)\n\
         REFUNDS-002 [keep] Refund over limit needs review  (tests/refunds.rs:5)\n\
         2 tests drafted\n"
    );
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}

#[test]
fn spec_049_a_target_with_no_untraced_test_is_refused() {
    let dir = project();
    let r = dir.path();
    put(
        r,
        "specjutsu/specs/billing/spec.md",
        "# billing Specification\n\n## Purpose\nCharge customers.\n\n## Requirements\n\n\
         ### Requirement: Charge\nThe system SHALL charge.\n\n\
         #### Scenario: BILL-001 Charge a card\n- **WHEN** a card is charged\n- **THEN** it is billed\n\n\
         #### Scenario: BILL-002 Refund a card\n- **WHEN** a card is refunded\n- **THEN** it is credited\n",
    );
    put(
        r,
        "tests/billing/charges.rs",
        "#[test]\nfn bill_001_charges_a_card() {}\n\n\
         // spec: BILL-002\n#[test]\nfn refunds_a_card() {}\n",
    );
    // Untraced, but outside the target path.
    put(r, "tests/other.rs", "#[test]\nfn other_thing_works() {}\n");
    let before = std::fs::read_to_string(r.join("tests/billing/charges.rs")).unwrap();

    sj(r, &["tests/billing"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("no untraced tests under tests/billing"));
    assert_eq!(
        std::fs::read_to_string(r.join("tests/billing/charges.rs")).unwrap(),
        before
    );
    assert!(!r.join("specjutsu/changes").exists());
}

#[test]
fn spec_056_skipped_tests_are_counted_not_drafted() {
    let dir = project();
    let r = dir.path();
    put(
        r,
        "tests/refunds/refunds.rs",
        "#[test]\nfn refund_under_100_auto_approves() {}\n\n\
         #[test]\n#[ignore]\nfn refund_over_limit_needs_review() {}\n",
    );
    put(
        r,
        "tests/refunds/refunds.test.ts",
        "describe.skip(\"refunds\", () => {\n  it(\"rejects a closed order\", () => {})\n})\n",
    );

    let out = sj(r, &["tests/refunds", "--capability", "refunds"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        "REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds/refunds.rs:2)\n\
         1 test drafted, 2 skipped left out\n"
    );
}
