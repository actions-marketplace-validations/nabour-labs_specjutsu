//! SPEC-038..042 (openspec/changes/bugfix-changes): `[keep]` markers and bug changes.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(root);
    c
}

const CONFIG: &str = "schema = 1\n\n[[package]]\npath = \".\"\nsources = []\n\n\
                      [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\n";

/// A project with an open change `fix-refunds` whose delta holds `scenarios` (heading titles).
fn project(scenarios: &[&str], bug: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(r, "specjutsu.toml", CONFIG);
    put(
        r,
        "openspec/specs/pay/spec.md",
        "# pay Specification\n\n## Purpose\nPayments.\n\n## Requirements\n\n\
         ### Requirement: Charge\nThe system SHALL charge cards.\n\n\
         #### Scenario: PAY-001 Charge a card\n- **WHEN** a card is charged\n- **THEN** it succeeds\n",
    );
    let mut delta = String::from(
        "## ADDED Requirements\n\n### Requirement: Refunds\nThe system SHALL refund charges.\n\n",
    );
    let mut ids = Vec::new();
    for title in scenarios {
        delta.push_str(&format!(
            "#### Scenario: {title}\n- **WHEN** a refund is asked for\n- **THEN** it is handled\n\n"
        ));
        ids.push(title.split_whitespace().next().unwrap().to_string());
    }
    put(r, "openspec/changes/fix-refunds/specs/pay/spec.md", &delta);
    let kind = if bug { "Kind: bug\n\n" } else { "" };
    put(
        r,
        "openspec/changes/fix-refunds/proposal.md",
        &format!("## Why\n\nRefunds round wrong.\n\n{kind}"),
    );
    put(
        r,
        "openspec/changes/fix-refunds/tasks.md",
        &format!("# Tasks\n\n- [ ] 1.1 Refunds ({})\n", ids.join(", ")),
    );
    dir
}

#[test]
fn spec_038_a_keep_marker_is_parsed() {
    let dir = project(&["PAY-012 [keep] Refunds under $100 auto-approve"], false);
    let out = sj(dir.path(), &["spec", "show", "PAY-012", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["keep"], true, "{v}");
    assert_eq!(v["name"], "Refunds under $100 auto-approve", "{v}");
}

#[test]
fn spec_038_a_plain_scenario_is_not_keep() {
    let dir = project(&["PAY-012 Refunds under $100 auto-approve"], false);
    let out = sj(dir.path(), &["spec", "show", "PAY-012", "--json"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["keep"], false, "{v}");
}

#[test]
fn spec_039_an_unknown_marker_is_rejected() {
    let dir = project(&["PAY-013 [maybe] Partial refunds"], false);
    sj(dir.path(), &["spec", "validate", "fix-refunds"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unknown scenario marker [maybe] (openspec/changes/fix-refunds/specs/pay/spec.md:6)",
        ));
}

#[test]
fn spec_040_a_bugfix_change_is_scaffolded() {
    let dir = project(&["PAY-012 Refunds"], false);
    sj(dir.path(), &["spec", "new", "fix-refund-rounding", "--bug"])
        .assert()
        .success();
    let text = std::fs::read_to_string(
        dir.path()
            .join("openspec/changes/fix-refund-rounding/proposal.md"),
    )
    .unwrap();
    for section in [
        "## Current Behaviour",
        "## Expected Behaviour",
        "## Unchanged Behaviour",
    ] {
        assert!(text.contains(section), "{section} missing:\n{text}");
    }
    assert!(text.lines().any(|l| l == "Kind: bug"), "{text}");
}

#[test]
fn spec_041_a_bugfix_with_no_keep_scenario_is_warned_about() {
    let dir = project(&["PAY-012 Refunds round half up"], true);
    sj(dir.path(), &["spec", "check", "fix-refunds"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "warn    bug change has no [keep] scenario; add one for behaviour that must not change",
        ));
    // With a keep scenario next to the reproduction, the warning goes away.
    let dir = project(
        &[
            "PAY-012 Refunds round half up",
            "PAY-013 [keep] Full refunds",
        ],
        true,
    );
    sj(dir.path(), &["spec", "check", "fix-refunds"])
        .assert()
        .success()
        .stdout("check: ok\n");
}

#[test]
fn spec_042_a_bugfix_with_only_keep_scenarios_is_an_error() {
    let dir = project(&["PAY-013 [keep] Full refunds"], true);
    sj(dir.path(), &["spec", "check", "fix-refunds"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "bug change has no reproduction scenario",
        ));
}
