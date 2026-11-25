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
        &format!("{CONFIG}\n[ids.prefixes]\npayments = \"PAY\"\n"),
    );
    put(
        r,
        "specjutsu/specs/payments/spec.md",
        "# payments Specification\n\n## Purpose\nTake payments.\n\n## Requirements\n\n\
         ### Requirement: Pay\nThe system SHALL take payments.\n\n\
         #### Scenario: PAY-001 Take a payment\n- **WHEN** a card is charged\n- **THEN** it is paid\n",
    );
    put(
        r,
        "tests/pay/runs.rs",
        "#[test]\nfn pay_runs_monthly() {}\n",
    );
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

    sj(r, &["tests/pay", "--capability", "pay"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "capabilities `pay` and `payments` both use prefix PAY",
        ));
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}

fn commit_all(r: &Path) {
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
}

#[test]
fn spec_061_test_folders_are_skipped_and_the_name_is_kebab_cased() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \"packages/CartItems\"\n\
         sources = [\"packages/CartItems/src/**\"]\n\n\
         [[package.suite]]\nrunner = \"vitest\"\n\
         tests = [\"packages/CartItems/Tests/**/*.test.ts\"]\n",
    );
    put(
        r,
        "packages/CartItems/Tests/unit/cart.test.ts",
        "describe(\"cart\", () => {\n  it(\"adds an item\", () => {})\n})\n",
    );

    let out = sj(r, &["packages/CartItems/Tests/unit/cart.test.ts"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.starts_with("CARTITEMS-001 [keep] "),
        "first draft is not CARTITEMS-001: {stdout}"
    );
}

#[test]
fn spec_062_tests_in_different_capability_folders_need_capability() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n\
         [[package.suite]]\nrunner = \"vitest\"\n\
         tests = [\"packages/*/tests/**/*.test.ts\"]\n",
    );
    put(
        r,
        "packages/cart/tests/cart.test.ts",
        "it(\"adds an item\", () => {})\n",
    );
    put(
        r,
        "packages/orders/tests/orders.test.ts",
        "it(\"places an order\", () => {})\n",
    );
    commit_all(r);

    sj(r, &["packages/cart", "packages/orders"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "the tests' folders name capabilities cart, orders; pass --capability to pick one",
        ));
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}

#[test]
fn spec_063_a_test_with_no_capability_folder_needs_capability() {
    let dir = project();
    let r = dir.path();
    put(r, "tests/top.rs", "#[test]\nfn top_level_works() {}\n");
    commit_all(r);

    sj(r, &["tests/top.rs"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "tests/top.rs is in no capability folder; pass --capability",
        ));
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}

#[test]
fn spec_064_a_capability_with_no_letters_is_refused() {
    let dir = project();
    let r = dir.path();
    put(r, "tests/x.rs", "#[test]\nfn x_works() {}\n");
    commit_all(r);

    sj(r, &["tests/x.rs", "--capability", "2024"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "capability `2024` has no letters to make a prefix; map it in [ids.prefixes]",
        ));
    assert_eq!(git(r, &["status", "--porcelain"]), "");
}

/// SPEC-048's fixture: two untraced Rust tests in `tests/refunds.rs`, committed.
fn refunds_project() -> tempfile::TempDir {
    let dir = project();
    put(
        dir.path(),
        "tests/refunds.rs",
        "#[test]\nfn refund_under_100_auto_approves() {}\n\n\
         #[test]\nfn refund_over_limit_needs_review() {}\n",
    );
    commit_all(dir.path());
    dir
}

fn write_refunds(r: &Path) -> std::process::Output {
    sj(
        r,
        &["tests/refunds.rs", "--capability", "refunds", "--write"],
    )
    .output()
    .unwrap()
}

fn sj_spec(r: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("spec").args(args).arg("--root").arg(r);
    c
}

#[test]
fn spec_050_writing_creates_a_change_that_validates() {
    let dir = refunds_project();
    let r = dir.path();

    let out = write_refunds(r);
    assert!(out.status.success(), "{out:?}");
    let change = r.join("specjutsu/changes/infer-refunds");
    let spec = std::fs::read_to_string(change.join("specs/refunds/spec.md")).unwrap();
    assert_eq!(spec.matches("### Requirement: ").count(), 1, "{spec}");
    assert!(
        spec.contains(
            "### Requirement: Refunds\nThe system SHALL keep the behaviour that tests/refunds.rs tests.\n"
        ),
        "{spec}"
    );
    let scenarios: Vec<&str> = spec
        .lines()
        .filter(|l| l.starts_with("#### Scenario: "))
        .collect();
    assert_eq!(scenarios.len(), 2, "{spec}");
    assert!(
        scenarios[0].starts_with("#### Scenario: REFUNDS-001 [keep] "),
        "{spec}"
    );
    assert!(
        scenarios[1].starts_with("#### Scenario: REFUNDS-002 [keep] "),
        "{spec}"
    );
    let tasks = std::fs::read_to_string(change.join("tasks.md")).unwrap();
    assert!(tasks.contains("(REFUNDS-001..002)"), "{tasks}");
    assert!(change.join("proposal.md").is_file());

    sj_spec(r, &["validate", "infer-refunds"]).assert().code(0);
}

#[test]
fn spec_051_an_existing_change_folder_is_refused() {
    let dir = refunds_project();
    let r = dir.path();
    put(
        r,
        "specjutsu/changes/infer-refunds/proposal.md",
        "## Why\nMine.\n",
    );
    commit_all(r);

    let out = write_refunds(r);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("change infer-refunds already exists"),
        "{stderr}"
    );
    assert_eq!(
        git(r, &["status", "--porcelain", "--untracked-files=all"]),
        ""
    );
}

#[test]
fn spec_053_check_reports_the_drafted_markers() {
    let dir = refunds_project();
    let r = dir.path();
    let out = write_refunds(r);
    assert!(out.status.success(), "{out:?}");

    let out = sj_spec(r, &["check", "infer-refunds"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let clarify: Vec<&str> = stdout
        .lines()
        .filter(|l| l.starts_with("clarify"))
        .collect();
    assert_eq!(clarify.len(), 4, "{stdout}");
    for line in [2, 5] {
        let naming = format!("tests/refunds.rs:{line}");
        let n = clarify.iter().filter(|l| l.contains(&naming)).count();
        assert_eq!(n, 2, "expected two clarify lines naming {naming}: {stdout}");
    }
}

/// The text of a proposal's `### <title>` section, up to the next heading.
fn proposal_section<'a>(proposal: &'a str, title: &str) -> &'a str {
    let start = proposal
        .find(&format!("### {title}\n"))
        .unwrap_or_else(|| panic!("no {title} section: {proposal}"))
        + title.len()
        + 5;
    let rest = &proposal[start..];
    &rest[..rest.find("\n#").unwrap_or(rest.len())]
}

fn requirement_names(spec: &str) -> Vec<&str> {
    spec.lines()
        .filter_map(|l| l.strip_prefix("### Requirement: "))
        .collect()
}

#[test]
fn spec_065_test_files_with_the_same_stem_get_distinct_requirements() {
    let dir = project();
    let r = dir.path();
    put(r, "tests/a/refunds.rs", "#[test]\nfn refund_a_works() {}\n");
    put(r, "tests/b/refunds.rs", "#[test]\nfn refund_b_works() {}\n");
    commit_all(r);

    let out = sj(r, &["tests/a", "tests/b", "--capability", "dup", "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let spec =
        std::fs::read_to_string(r.join("specjutsu/changes/infer-dup/specs/dup/spec.md")).unwrap();
    assert_eq!(
        requirement_names(&spec),
        ["Tests a refunds", "Tests b refunds"],
        "{spec}"
    );

    sj_spec(r, &["archive", "infer-dup", "--force"])
        .assert()
        .success();
}

#[test]
fn spec_066_drafts_for_an_existing_capability_modify_it_without_clashing() {
    let dir = project();
    let r = dir.path();
    put(
        r,
        "specjutsu/specs/refunds/spec.md",
        "# refunds Specification\n\n## Purpose\nRefund orders.\n\n## Requirements\n\n\
         ### Requirement: Refunds\nThe system SHALL refund.\n\n\
         #### Scenario: REFUNDS-001 Refund an order\n- **WHEN** an order is refunded\n- **THEN** it is credited\n",
    );
    put(
        r,
        "tests/refunds.rs",
        "#[test]\nfn refund_over_limit_needs_review() {}\n",
    );
    commit_all(r);

    let out = write_refunds(r);
    assert!(out.status.success(), "{out:?}");
    let change = r.join("specjutsu/changes/infer-refunds");
    let spec = std::fs::read_to_string(change.join("specs/refunds/spec.md")).unwrap();
    assert_eq!(requirement_names(&spec), ["Tests refunds"], "{spec}");
    assert!(
        spec.contains("#### Scenario: REFUNDS-002 [keep] "),
        "{spec}"
    );
    let proposal = std::fs::read_to_string(change.join("proposal.md")).unwrap();
    assert!(
        proposal_section(&proposal, "Modified Capabilities").contains("`refunds`"),
        "{proposal}"
    );
    assert!(
        !proposal_section(&proposal, "New Capabilities").contains("`refunds`"),
        "{proposal}"
    );

    sj_spec(r, &["archive", "infer-refunds", "--force"])
        .assert()
        .success();
}

#[test]
fn spec_054_tests_gain_a_spec_comment_and_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        &format!(
            "{CONFIG}\n[[package.suite]]\nrunner = \"pytest\"\ntests = [\"tests/**/test_*.py\"]\n"
        ),
    );
    put(
        r,
        "tests/refunds.rs",
        "#[test]\nfn refund_under_100_auto_approves() {}\n\n\
         #[test]\nfn refund_over_limit_needs_review() {}\n",
    );
    put(
        r,
        "tests/test_refunds.py",
        "import pytest\n\n\nclass TestRefunds:\n    @pytest.mark.slow\n    \
         def test_refund_is_logged(self):\n        pass\n",
    );
    commit_all(r);

    let out = sj(
        r,
        &[
            "tests/refunds.rs",
            "tests/test_refunds.py",
            "--capability",
            "refunds",
            "--write",
        ],
    )
    .output()
    .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("REFUNDS-003 [keep] "), "{stdout}");

    assert_eq!(
        std::fs::read_to_string(r.join("tests/refunds.rs")).unwrap(),
        "// spec: REFUNDS-001\n#[test]\nfn refund_under_100_auto_approves() {}\n\n\
         // spec: REFUNDS-002\n#[test]\nfn refund_over_limit_needs_review() {}\n"
    );
    assert_eq!(
        std::fs::read_to_string(r.join("tests/test_refunds.py")).unwrap(),
        "import pytest\n\n\nclass TestRefunds:\n    # spec: REFUNDS-003\n    \
         @pytest.mark.slow\n    def test_refund_is_logged(self):\n        pass\n"
    );
    assert_eq!(
        git(r, &["diff", "--numstat", "--", "tests"]),
        "2\t0\ttests/refunds.rs\n1\t0\ttests/test_refunds.py\n"
    );

    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--json", "--root"])
        .arg(r)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mapped: Vec<(&str, &str)> = v["mapped"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["id"].as_str().unwrap(),
                m["tests"][0]["file"].as_str().unwrap(),
            )
        })
        .collect();
    assert!(mapped.contains(&("REFUNDS-001", "tests/refunds.rs")), "{v}");
    assert!(
        mapped.contains(&("REFUNDS-003", "tests/test_refunds.py")),
        "{v}"
    );
}

#[test]
fn spec_055_a_second_run_drafts_nothing_new() {
    let dir = refunds_project();
    let r = dir.path();
    let out = write_refunds(r);
    assert!(out.status.success(), "{out:?}");

    sj(r, &["tests/refunds.rs"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "no untraced tests under tests/refunds.rs",
        ));
}

/// SPEC-054's "nothing else": a CRLF file without a final newline keeps both, and a pytest
/// decorator spanning lines gets the comment above its first line.
#[test]
fn spec_054_line_endings_and_multi_line_decorators_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(
        r,
        "specjutsu.toml",
        &format!(
            "{CONFIG}\n[[package.suite]]\nrunner = \"pytest\"\ntests = [\"tests/**/test_*.py\"]\n"
        ),
    );
    put(
        r,
        "tests/refunds.rs",
        "#[test]\r\n#[should_panic]\r\nfn refund_over_limit_panics() { panic!() }",
    );
    put(
        r,
        "tests/test_refunds.py",
        "@pytest.mark.parametrize(\n    \"amount\",\n    [1, 2],\n)\ndef test_refund(amount):\n    pass\n",
    );
    commit_all(r);

    let out = sj(
        r,
        &[
            "tests/refunds.rs",
            "tests/test_refunds.py",
            "--capability",
            "refunds",
            "--write",
        ],
    )
    .output()
    .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        std::fs::read_to_string(r.join("tests/refunds.rs")).unwrap(),
        "// spec: REFUNDS-001\r\n#[test]\r\n#[should_panic]\r\nfn refund_over_limit_panics() { panic!() }"
    );
    assert_eq!(
        std::fs::read_to_string(r.join("tests/test_refunds.py")).unwrap(),
        "# spec: REFUNDS-002\n@pytest.mark.parametrize(\n    \"amount\",\n    [1, 2],\n)\n\
         def test_refund(amount):\n    pass\n"
    );
}
