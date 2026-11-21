//! `sj instructions`, `sj templates`, the `language` key and project context
//! (specjutsu/changes/agent-instructions).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn sj(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(root);
    c
}

fn stdout(root: &Path, args: &[&str]) -> String {
    let out = sj(root, args).output().unwrap();
    assert!(
        out.status.success(),
        "sj {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn json(root: &Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&stdout(root, args)).unwrap()
}

/// A spec with AUTH-001..AUTH-004, one requirement each.
fn auth_spec() -> String {
    let mut s =
        String::from("# auth Specification\n\n## Purpose\nSign users in.\n\n## Requirements\n");
    for (i, name) in ["Login", "Logout", "Refresh", "Revoke"].iter().enumerate() {
        s.push_str(&format!(
            "\n### Requirement: {name}\nThe system SHALL {name}.\n\n#### Scenario: AUTH-00{} {name} works\n- **WHEN** asked\n- **THEN** it works\n",
            i + 1
        ));
    }
    s
}

const SSO_DELTA: &str = "## ADDED Requirements\n\n### Requirement: SSO\nThe system SHALL sign in with SSO.\n\n#### Scenario: AUTH-005 SSO works\n- **WHEN** asked\n- **THEN** it works\n";

/// `openspec/specs/auth` plus the changes `add-oauth-login` and `add-sso`, both scaffolded by
/// `sj spec new` so their files are still the templates.
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", "schema = 1\n");
    put(root, "openspec/specs/auth/spec.md", &auth_spec());
    stdout(root, &["spec", "new", "add-oauth-login"]);
    stdout(root, &["spec", "new", "add-sso"]);
    put(
        root,
        "openspec/changes/add-sso/specs/auth/spec.md",
        SSO_DELTA,
    );
    dir
}

const PRINCIPLES: &str = "# Principles\n\nRust CLI\n\n## Rules\n\n### tasks\n\n- Name the scenario IDs\n\n### design\n\n- Record every rejected alternative\n";

// --- config: language -------------------------------------------------------------------

#[test]
fn conf_012_a_language_tag_is_accepted() {
    let dir = project();
    let root = dir.path();
    put(root, "specjutsu.toml", "schema = 1\nlanguage = \"pt-BR\"\n");
    let out = sj(root, &["doctor"]).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("specjutsu.toml"), "{text}");
    assert!(!text.contains("language"), "{text}");

    let v = json(root, &["instructions", "add-oauth-login", "--json"]);
    assert_eq!(v["language"], "pt-BR");
}

#[test]
fn conf_013_an_invalid_language_is_an_environment_error() {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "specjutsu.toml",
        "schema = 1\nlanguage = \"french!\"\n",
    );
    let commands: [&[&str]; 5] = [
        &["trace"],
        &["test"],
        &["ledger"],
        &["instructions", "add-oauth-login"],
        &["templates"],
    ];
    for args in commands {
        let out = sj(root, args).output().unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "sj {args:?}: {err}");
        assert!(err.contains("language"), "sj {args:?}: {err}");
        assert!(err.contains("french!"), "sj {args:?}: {err}");
    }
}

#[test]
fn conf_029_language_falls_back_to_openspec_s_config_yaml() {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "openspec/config.yaml",
        "schema: spec-driven\nlanguage: fr\n",
    );
    let v = json(root, &["instructions", "add-oauth-login", "--json"]);
    assert_eq!(v["language"], "fr");

    put(root, "specjutsu.toml", "schema = 1\nlanguage = \"de\"\n");
    let v = json(root, &["instructions", "add-oauth-login", "--json"]);
    assert_eq!(v["language"], "de");
}

#[test]
fn conf_017_a_translated_keyword_is_flagged() {
    let dir = project();
    let root = dir.path();
    put(root, "specjutsu.toml", "schema = 1\nlanguage = \"fr\"\n");
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Jetons\nLe système SHALL refuser un jeton expiré.\n\n\
         #### Scenario: AUTH-006 Jeton valide\n- **WHEN** le jeton est valide\n- **THEN** la requête passe\n\n\
         #### Scénario: AUTH-007 Jeton expiré\n- **WHEN** le jeton a expiré\n- **THEN** la réponse est 401\n",
    );
    let out = sj(root, &["spec", "validate", "add-oauth-login"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.lines().any(|l| l.starts_with("warn")
            && l.contains("openspec/changes/add-oauth-login/specs/auth/spec.md:10")
            && l.contains("#### Scenario:")),
        "{text}"
    );

    sj(root, &["spec", "validate", "add-oauth-login", "--strict"])
        .assert()
        .code(1);
}

// --- config: project context ------------------------------------------------------------

#[test]
fn conf_014_an_edit_to_principles_md_shows_on_the_next_run() {
    let dir = project();
    let root = dir.path();
    put(root, "openspec/principles.md", PRINCIPLES);
    let before = stdout(root, &["instructions", "add-oauth-login", "proposal"]);
    assert!(before.contains("Rust CLI"), "{before}");
    assert!(!before.contains("Use British spelling."), "{before}");

    put(
        root,
        "openspec/principles.md",
        &PRINCIPLES.replace("Rust CLI\n", "Rust CLI\nUse British spelling.\n"),
    );
    let after = stdout(root, &["instructions", "add-oauth-login", "proposal"]);
    assert!(after.contains("Use British spelling."), "{after}");
}

#[test]
fn conf_015_an_unmigrated_openspec_config_is_read_in_place() {
    let dir = project();
    let root = dir.path();
    let yaml = "schema: spec-driven\ncontext: \"Tech stack: Django\"\nrules:\n  proposal:\n    - Name the ticket\n";
    put(root, "openspec/config.yaml", yaml);
    let out = stdout(root, &["instructions", "add-oauth-login", "proposal"]);
    assert!(out.contains("Tech stack: Django"), "{out}");
    assert!(out.contains("Name the ticket"), "{out}");
    assert!(out.contains("openspec/config.yaml"), "{out}");
    assert_eq!(read(root, "openspec/config.yaml"), yaml);

    let v = json(
        root,
        &["instructions", "add-oauth-login", "proposal", "--json"],
    );
    assert_eq!(v["context"][0]["source"], "openspec/config.yaml");
    assert_eq!(v["context"][0]["text"], "Tech stack: Django");
    assert_eq!(v["rules"][0]["source"], "openspec/config.yaml");
    assert_eq!(v["rules"][0]["rules"][0], "Name the ticket");
}

#[test]
fn conf_016_both_sources_are_combined_in_a_fixed_order() {
    let dir = project();
    let root = dir.path();
    put(root, "openspec/principles.md", PRINCIPLES);
    put(
        root,
        "openspec/config.yaml",
        "context: |\n  Tech stack: Django\n",
    );
    let v = json(
        root,
        &["instructions", "add-oauth-login", "proposal", "--json"],
    );
    let ctx = v["context"].as_array().unwrap();
    assert_eq!(ctx.len(), 2, "{v}");
    assert_eq!(ctx[0]["source"], "openspec/principles.md");
    assert!(ctx[0]["text"].as_str().unwrap().contains("Rust CLI"));
    assert_eq!(ctx[1]["source"], "openspec/config.yaml");
    assert_eq!(ctx[1]["text"], "Tech stack: Django");

    let out = stdout(root, &["instructions", "add-oauth-login", "proposal"]);
    let a = out.find("Rust CLI").unwrap();
    let b = out.find("Tech stack: Django").unwrap();
    assert!(a < b, "{out}");
}

// --- authoring: templates ---------------------------------------------------------------

#[test]
fn spec_034_templates_list_built_ins_by_default() {
    let dir = project();
    let out = stdout(dir.path(), &["templates"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4, "{out}");
    for name in ["proposal.md", "spec.md", "design.md", "tasks.md"] {
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with(name) && l.ends_with(&format!("built-in:{name}"))),
            "{name}: {out}"
        );
    }
}

#[test]
fn spec_035_a_project_template_wins_and_is_used_by_sj_spec_new() {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "openspec/templates/proposal.md",
        "## Problem\n\n<!-- What hurts. -->\n",
    );
    let out = stdout(root, &["templates"]);
    assert!(
        out.lines()
            .any(|l| l == "proposal.md  openspec/templates/proposal.md"),
        "{out}"
    );
    stdout(root, &["spec", "new", "add-x"]);
    assert!(read(root, "openspec/changes/add-x/proposal.md").starts_with("## Problem"));
}

#[test]
fn spec_036_json_templates_name_their_source() {
    let dir = project();
    let root = dir.path();
    put(root, "openspec/templates/tasks.md", "# Tasks\n");
    let v = json(root, &["templates", "--json"]);
    let all = v.as_array().unwrap();
    assert_eq!(all.len(), 4, "{v}");
    for t in all {
        let want = if t["name"] == "tasks.md" {
            "project"
        } else {
            "built-in"
        };
        assert_eq!(t["source"], want, "{t}");
    }
    let tasks = all.iter().find(|t| t["name"] == "tasks.md").unwrap();
    assert_eq!(tasks["path"], "openspec/templates/tasks.md");
}

// --- loop: sj instructions --------------------------------------------------------------

#[test]
fn loop_012_instructions_pick_the_first_unwritten_artifact() {
    let dir = project();
    let root = dir.path();
    put(
        root,
        "openspec/changes/add-oauth-login/proposal.md",
        "## Why\n\nUsers want OAuth.\n",
    );
    let out = stdout(root, &["instructions", "add-oauth-login"]);
    assert!(out.contains("artifact specs"), "{out}");
    assert!(
        out.contains("changes/add-oauth-login/specs/<capability>/spec.md"),
        "{out}"
    );
}

#[test]
fn loop_013_json_instructions_carry_every_field() {
    let dir = project();
    let v = json(
        dir.path(),
        &["instructions", "add-oauth-login", "design", "--json"],
    );
    assert_eq!(v["change"], "add-oauth-login");
    assert_eq!(v["artifact"], "design");
    for key in [
        "template", "context", "rules", "next_ids", "language", "check",
    ] {
        assert!(v.get(key).is_some(), "missing {key}: {v}");
    }
    assert!(v["language"].is_null(), "{v}");
}

#[test]
fn loop_014_instructions_name_the_next_free_ids() {
    let dir = project();
    let out = stdout(dir.path(), &["instructions", "add-oauth-login", "specs"]);
    assert!(out.lines().any(|l| l == "next AUTH-006"), "{out}");
    let v = json(
        dir.path(),
        &["instructions", "add-oauth-login", "specs", "--json"],
    );
    assert_eq!(v["next_ids"][0], "AUTH-006");
}

#[test]
fn loop_015_instructions_include_only_the_artifact_s_own_rules() {
    let dir = project();
    let root = dir.path();
    put(root, "openspec/principles.md", PRINCIPLES);
    let out = stdout(root, &["instructions", "add-oauth-login", "tasks"]);
    assert!(out.contains("Name the scenario IDs"), "{out}");
    assert!(!out.contains("Record every rejected alternative"), "{out}");
}

#[test]
fn loop_016_each_artifact_names_its_check() {
    let dir = project();
    let root = dir.path();
    let specs = json(
        root,
        &["instructions", "add-oauth-login", "specs", "--json"],
    );
    assert_eq!(specs["check"], "sj spec validate add-oauth-login");
    let tasks = json(
        root,
        &["instructions", "add-oauth-login", "tasks", "--json"],
    );
    assert_eq!(tasks["check"], "sj spec check add-oauth-login");
    let text = stdout(root, &["instructions", "add-oauth-login", "tasks"]);
    assert!(
        text.contains("check sj spec check add-oauth-login"),
        "{text}"
    );
}

#[test]
fn loop_017_unknown_change_or_artifact_is_an_error() {
    let dir = project();
    let root = dir.path();
    sj(root, &["instructions", "no-such-change"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("no-such-change"));
    let out = sj(root, &["instructions", "add-oauth-login", "readme"])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(out.stdout.is_empty());
    for a in ["readme", "proposal", "specs", "design", "tasks"] {
        assert!(err.contains(a), "{a}: {err}");
    }
}

#[test]
fn loop_018_a_fully_written_change_points_to_the_loop() {
    let dir = project();
    let root = dir.path();
    let c = "openspec/changes/add-oauth-login";
    put(root, &format!("{c}/proposal.md"), "## Why\n\nOAuth.\n");
    put(root, &format!("{c}/specs/auth/spec.md"), SSO_DELTA);
    put(root, &format!("{c}/design.md"), "## Context\n\nNone.\n");
    put(
        root,
        &format!("{c}/tasks.md"),
        "# Tasks\n\n- [ ] 1.1 SSO (AUTH-005)\n",
    );
    sj(root, &["instructions", "add-oauth-login"])
        .assert()
        .success()
        .stdout("all artifacts written; run sj next add-oauth-login\n");
}

#[test]
fn loop_019_instructions_carry_the_project_language() {
    let dir = project();
    let root = dir.path();
    put(root, "specjutsu.toml", "schema = 1\nlanguage = \"fr\"\n");
    let out = stdout(root, &["instructions", "add-oauth-login", "proposal"]);
    let line = out
        .lines()
        .find(|l| l.starts_with("language fr"))
        .unwrap_or_else(|| panic!("no language line: {out}"));
    for kw in [
        "### Requirement:",
        "#### Scenario:",
        "WHEN",
        "THEN",
        "SHALL",
        "MUST",
        "## ADDED Requirements",
    ] {
        assert!(line.contains(kw), "{kw}: {line}");
    }
    assert!(line.contains("English"), "{line}");
}
