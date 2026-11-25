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
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "init",
        ],
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
        .stderr(predicate::str::contains(
            "no untraced tests under tests/billing",
        ));
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
         #[test]\n#[ignore]\nfn refund_over_limit_needs_review() {}\n\n\
         // spec: ORD-001\n#[test]\nfn refund_closes_the_order() {}\n",
    );
    put(
        r,
        "specjutsu/specs/orders/spec.md",
        "# orders Specification\n\n## Purpose\nClose orders.\n\n## Requirements\n\n\
         ### Requirement: Close\nThe system SHALL close orders.\n\n\
         #### Scenario: ORD-001 Close an order\n- **WHEN** an order is refunded\n- **THEN** it is closed\n",
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
         1 test drafted, 1 linked and 2 skipped left out\n"
    );
}

#[test]
fn spec_057_an_existing_capability_keeps_its_prefix_and_numbering() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        &format!("{CONFIG}\n[ids.prefixes]\nbilling = \"BILL\"\n"),
    );
    put(
        r,
        "specjutsu/specs/billing/spec.md",
        "# billing Specification\n\n## Purpose\nCharge customers.\n\n## Requirements\n\n\
         ### Requirement: Charge\nThe system SHALL charge.\n\n\
         #### Scenario: BILL-014 Charge a card\n- **WHEN** a card is charged\n- **THEN** it is billed\n",
    );
    put(
        r,
        "specjutsu/changes/archive/2026-01-01-refunds/specs/billing/spec.md",
        "## ADDED Requirements\n\n### Requirement: Refund\nThe system SHALL refund.\n\n\
         #### Scenario: BILL-015 Refund a card\n- **WHEN** a card is refunded\n- **THEN** it is credited\n",
    );
    put(r, "tests/invoices.rs", "#[test]\nfn invoice_is_sent() {}\n");

    let out = sj(r, &["tests/invoices.rs", "--capability", "billing"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.starts_with("BILL-016 [keep] "),
        "first draft is not BILL-016: {stdout}"
    );
}

#[test]
fn spec_058_the_capability_is_taken_from_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \"packages/checkout\"\nsources = [\"packages/checkout/src/**\"]\n\n\
         [[package.suite]]\nrunner = \"vitest\"\ntests = [\"packages/checkout/tests/**/*.test.ts\"]\n",
    );
    put(
        r,
        "packages/checkout/tests/cart.test.ts",
        "describe(\"cart\", () => {\n  it(\"adds an item\", () => {})\n})\n",
    );

    let out = sj(r, &["packages/checkout/tests/cart.test.ts"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        "CHECKOUT-001 [keep] Cart adds an item  (packages/checkout/tests/cart.test.ts:2)\n\
         1 test drafted\n"
    );
}

#[test]
fn spec_059_a_colliding_prefix_stops_the_run() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        &format!("{CONFIG}\n[ids.prefixes]\npayments = \"PAY\"\npayroll = \"PAY\"\n"),
    );
    put(
        r,
        "specjutsu/specs/payments/spec.md",
        "# payments Specification\n\n## Purpose\nTake payments.\n\n## Requirements\n\n\
         ### Requirement: Pay\nThe system SHALL take payments.\n\n\
         #### Scenario: PAY-001 Take a payment\n- **WHEN** a card is charged\n- **THEN** it is paid\n",
    );
    put(r, "tests/payroll/runs.rs", "#[test]\nfn payroll_runs_monthly() {}\n");
    git(r, &["init", "-q"]);
    git(r, &["add", "."]);
    git(
        r,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "init",
        ],
    );

    sj(r, &["tests/payroll", "--capability", "payroll"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "capabilities `payments` and `payroll` both use prefix PAY",
        ));
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}
