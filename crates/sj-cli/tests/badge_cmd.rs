//! CI-004..006 from specjutsu/changes/gate-action: `sj badge`.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = r#"schema = 1

[[package]]
path = "."
sources = ["src/**"]

[[package.suite]]
runner = "vitest"
tests = ["src/**/*.test.ts"]
"#;

/// A project with `total` scenarios in `specs/`, the first `tested` of them with a test, and
/// the first `skipped` of those tests marked `.skip`. A change folder holds 5 more scenarios
/// with tests, which the badge must not count.
fn project(total: usize, tested: usize, skipped: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    let mut spec =
        String::from("## Requirements\n\n### Requirement: Cart\nThe system SHALL sell.\n\n");
    let mut tests = String::new();
    for n in 1..=total {
        spec.push_str(&format!(
            "#### Scenario: CART-{n:03} Case {n}\n- **WHEN** x\n- **THEN** y\n\n"
        ));
        if n <= tested {
            let it = if n <= skipped { "it.skip" } else { "it" };
            tests.push_str(&format!("{it}(\"CART-{n:03} case {n}\", () => {{}})\n"));
        }
    }
    write(r, "openspec/specs/cart/spec.md", &spec);
    let mut change =
        String::from("## ADDED Requirements\n\n### Requirement: More\nThe system SHALL more.\n\n");
    for n in 1..=5 {
        change.push_str(&format!(
            "#### Scenario: MORE-{n:03} More {n}\n- **WHEN** x\n- **THEN** y\n\n"
        ));
        tests.push_str(&format!("it(\"MORE-{n:03} more {n}\", () => {{}})\n"));
    }
    write(r, "openspec/changes/more/specs/more/spec.md", &change);
    write(r, "openspec/changes/more/proposal.md", "## Why\n\nTest.\n");
    write(r, "src/cart.test.ts", &tests);
    dir
}

fn badge(root: &Path) -> serde_json::Value {
    Command::cargo_bin("sj")
        .unwrap()
        .arg("badge")
        .arg("--root")
        .arg(root)
        .assert()
        .success();
    let text = std::fs::read_to_string(root.join(".specjutsu/badge.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn ci_004_badge_reports_tested_scenarios() {
    let dir = project(50, 46, 1);
    let text = {
        badge(dir.path());
        std::fs::read_to_string(dir.path().join(".specjutsu/badge.json")).unwrap()
    };
    assert_eq!(
        text.trim_end(),
        r#"{"schemaVersion":1,"label":"specs","message":"45/50 tested","color":"brightgreen"}"#
    );
}

#[test]
fn ci_005_badge_colour_follows_the_thresholds() {
    let yellow = project(50, 35, 0);
    assert_eq!(badge(yellow.path())["color"], "yellow");
    let red = project(50, 34, 0);
    assert_eq!(badge(red.path())["color"], "red");
}

#[test]
fn ci_006_a_project_without_scenarios_gets_a_neutral_badge() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "specjutsu.toml", CONFIG);
    std::fs::create_dir_all(dir.path().join("openspec/specs")).unwrap();
    let out = dir.path().join("custom/badge.json");
    Command::cargo_bin("sj")
        .unwrap()
        .arg("badge")
        .arg("--root")
        .arg(dir.path())
        .arg("--out")
        .arg(&out)
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out).unwrap()).unwrap();
    assert_eq!(v["message"], "no specs");
    assert_eq!(v["color"], "lightgrey");
}
