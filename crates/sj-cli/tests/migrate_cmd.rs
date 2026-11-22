//! `sj migrate` scenarios (specjutsu/changes/tool-migration).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

const SPEC: &str = "# math Specification

## Purpose
Math.

## Requirements

### Requirement: Add
The system SHALL add.

#### Scenario: Adds two numbers
- **WHEN** 1 and 2 are added
- **THEN** the result is 3

#### Scenario: Adds zero
- **WHEN** 0 is added
- **THEN** the number is unchanged
";

const CONFIG_YAML: &str = "schema: spec-driven

# Project context.
context: |
  Demo is a small crate.
  Tests never call the network.

rules:
  tasks:
    - Each task names the scenario IDs it covers
    - \"Commit the failing test first\"

operations:
  apply:
    guidance:
      - Run the tests before marking a task done
";

fn openspec_skill(name: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: Generated.\nlicense: MIT\nmetadata:\n  author: openspec\n  version: \"1.0\"\n  generatedBy: \"1.14.0\"\n---\n\nBody.\n"
    )
}

const SETTINGS: &str = r#"{
  "extraKnownMarketplaces": {
    "superpowers-marketplace": {
      "source": {
        "source": "github",
        "repo": "obra/superpowers-marketplace"
      }
    }
  },
  "enabledPlugins": {
    "superpowers@superpowers-marketplace": true
  },
  "permissions": {
    "deny": [
      "Skill(superpowers:brainstorming)",
      "Skill(superpowers:writing-plans)",
      "Skill(superpowers:executing-plans)",
      "Edit(/docs/superpowers/**)"
    ]
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "node scripts/hooks/require-worktree.mjs"
          }
        ]
      }
    ]
  }
}
"#;

/// A Cargo project using OpenSpec (generated skills and commands, a project skill) and
/// Superpowers, committed.
fn openspec_repo() -> support::Repo {
    let repo = support::Repo::with_lib("");
    repo.write("openspec/specs/math/spec.md", SPEC);
    repo.write("openspec/changes/.gitkeep", "");
    repo.write("openspec/config.yaml", CONFIG_YAML);
    for agent in [".claude", ".agents"] {
        for skill in ["openspec-propose", "openspec-apply-change"] {
            repo.write(
                &format!("{agent}/skills/{skill}/SKILL.md"),
                &openspec_skill(skill),
            );
        }
    }
    repo.write(
        ".claude/commands/opsx/propose.md",
        "---\nname: \"OPSX: Propose\"\ndescription: \"Propose\"\n---\n\nBody.\n",
    );
    repo.write(
        ".claude/skills/deploy/SKILL.md",
        "---\nname: deploy\ndescription: Deploy the app.\n---\n\nRun the deploy.\n",
    );
    repo.write(
        ".claude/skills/openspec-notes/SKILL.md",
        "---\nname: openspec-notes\ndescription: Our own notes skill.\n---\n\nNotes.\n",
    );
    repo.write(".claude/settings.json", SETTINGS);
    repo.write("docs/superpowers/plans/2026-01-01-plan.md", "# Plan\n");
    repo.commit("openspec project");
    repo
}

fn copy(from: &Path, to: &Path) {
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            std::fs::create_dir_all(&dest).unwrap();
            copy(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).unwrap();
        }
    }
}

/// A Cargo project using Spec Kit: the import fixture, a manifest and the skill it lists.
fn speckit_repo() -> support::Repo {
    let repo = support::Repo::with_lib("");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sj-core/tests/fixtures/speckit");
    copy(&fixture, repo.path());
    repo.write(
        ".specify/integrations/claude.manifest.json",
        r#"{
  "integration": "claude",
  "version": "1.0.7",
  "files": {
    ".claude/skills/speckit-plan/SKILL.md": "0000"
  }
}
"#,
    );
    repo.write(
        ".claude/skills/speckit-plan/SKILL.md",
        "---\nname: \"speckit-plan\"\ndescription: \"Plan.\"\n---\n\nBody.\n",
    );
    repo.commit("spec kit project");
    repo
}

fn migrate(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("migrate").args(args).arg("--root").arg(root);
    c
}

fn status(repo: &support::Repo) -> String {
    repo.git(&["status", "--porcelain", "--untracked-files=all"])
}

#[test]
fn mig_001_preview_changes_nothing() {
    let repo = openspec_repo();
    migrate(
        repo.path(),
        &["--from", "openspec", "--from", "superpowers"],
    )
    .assert()
    .success()
    .stdout(predicate::str::starts_with(
        "migrate: openspec, superpowers (dry run; --write applies it)\n",
    ))
    .stdout(predicate::str::contains("create specjutsu.toml (adopt)\n"))
    .stdout(predicate::str::contains(
        "change openspec/specs/math/spec.md (2 IDs)\n",
    ))
    .stdout(predicate::str::contains(
        "create openspec/principles.md (from openspec/config.yaml)\n",
    ))
    .stdout(predicate::str::contains("remove openspec/config.yaml\n"))
    .stdout(predicate::str::contains(
        "remove .claude/skills/openspec-propose/SKILL.md (OpenSpec skill)\n",
    ))
    .stdout(predicate::str::contains(
        "remove .agents/skills/openspec-apply-change/SKILL.md (OpenSpec skill)\n",
    ))
    .stdout(predicate::str::contains(
        "remove .claude/commands/opsx/propose.md (OpenSpec command)\n",
    ))
    .stdout(predicate::str::contains(
        "change .claude/settings.json (remove Superpowers)\n",
    ))
    .stdout(predicate::str::contains(
        "review docs/superpowers/plans/2026-01-01-plan.md (kept)\n",
    ))
    .stdout(predicate::str::contains("create CLAUDE.md (adapter)\n"))
    .stdout(predicate::str::contains("deploy").not())
    .stdout(predicate::str::contains("openspec-notes").not());
    assert_eq!(status(&repo), "");
}

#[test]
fn mig_002_a_dirty_tree_blocks_the_write() {
    let repo = openspec_repo();
    repo.write("README.md", "# Demo\n");
    repo.commit("readme");
    repo.write("README.md", "# Demo, edited\n");
    migrate(repo.path(), &["--from", "openspec", "--write"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("commit or stash"))
        .stderr(predicate::str::contains("README.md"));
    assert_eq!(status(&repo), " M README.md\n");
}

#[test]
fn mig_003_openspec_agent_files_are_removed_spec_content_kept() {
    let repo = openspec_repo();
    migrate(repo.path(), &["--from", "openspec", "--write"])
        .assert()
        .stdout(predicate::str::starts_with("migrate: wrote openspec\n"));
    let root = repo.path();
    let spec = std::fs::read_to_string(root.join("openspec/specs/math/spec.md")).unwrap();
    assert_eq!(
        spec,
        SPEC.replace("Scenario: Adds two", "Scenario: MATH-001 Adds two")
            .replace("Scenario: Adds zero", "Scenario: MATH-002 Adds zero")
    );
    for gone in [
        ".claude/skills/openspec-propose",
        ".claude/skills/openspec-apply-change",
        ".claude/commands",
        ".agents",
        "openspec/config.yaml",
    ] {
        assert!(!root.join(gone).exists(), "{gone}");
    }
    assert!(root.join(".claude/skills/deploy/SKILL.md").is_file());
    assert!(root
        .join(".claude/skills/openspec-notes/SKILL.md")
        .is_file());
    let principles = std::fs::read_to_string(root.join("openspec/principles.md")).unwrap();
    assert_eq!(
        principles,
        "# Principles

Demo is a small crate.
Tests never call the network.

## Rules

### tasks

- Each task names the scenario IDs it covers
- Commit the failing test first

## Operations

### apply

#### guidance

- Run the tests before marking a task done
"
    );
}

#[test]
fn mig_004_spec_kit_files_go_only_after_a_good_import() {
    let repo = speckit_repo();
    let out = migrate(repo.path(), &["--from", "speckit", "--write"])
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with("migrate: wrote speckit\n"))
        .stdout(predicate::str::contains(
            "import specs/001-chrome-ai-agent → chrome-ai-agent (finished, 6 requirements, 5 scenarios, 3 gaps)\n",
        ))
        .get_output()
        .stdout
        .clone();
    let root = repo.path();
    assert!(root
        .join("openspec/specs/chrome-ai-agent/spec.md")
        .is_file());
    assert!(root
        .join("openspec/changes/archive/2026-09-27-chrome-ai-agent/proposal.md")
        .is_file());
    assert!(root.join("openspec/changes/history/tasks.md").is_file());
    assert_eq!(
        std::fs::read_to_string(root.join("openspec/principles.md")).unwrap(),
        repo.git(&["show", "HEAD:.specify/memory/constitution.md"])
    );
    for gone in [".specify", "specs", ".claude/skills/speckit-plan"] {
        assert!(!root.join(gone).exists(), "{gone}");
    }
    // The only validate errors are the import's open questions.
    let out = String::from_utf8(out).unwrap();
    let fails: Vec<&str> = out.lines().filter(|l| l.starts_with("fail")).collect();
    assert!(!fails.is_empty());
    for l in fails {
        assert!(
            l.contains("requirement FR-") || l.contains("task names no scenario: T0"),
            "{l}"
        );
    }
}

#[test]
fn mig_005_a_failed_import_removes_nothing() {
    let repo = speckit_repo();
    repo.write("openspec/specs/chrome-ai-agent/spec.md", "ours\n");
    repo.commit("existing spec");
    migrate(repo.path(), &["--from", "speckit", "--write"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "openspec/specs/chrome-ai-agent/spec.md",
        ));
    assert_eq!(status(&repo), "");
}

#[test]
fn mig_006_project_hooks_survive_superpowers_removal() {
    let repo = openspec_repo();
    migrate(repo.path(), &["--from", "superpowers", "--write"])
        .assert()
        .stdout(predicate::str::contains(
            "review docs/superpowers/plans/2026-01-01-plan.md (kept)\n",
        ));
    let root = repo.path();
    let settings = std::fs::read_to_string(root.join(".claude/settings.json")).unwrap();
    assert!(!settings.contains("superpowers"), "{settings}");
    assert!(settings.contains("node scripts/hooks/require-worktree.mjs"));
    assert!(settings.contains("sj hook claude pre-edit"));
    assert!(settings.contains("sj hook claude session-start"));
    assert!(root
        .join("docs/superpowers/plans/2026-01-01-plan.md")
        .is_file());
}

#[test]
fn mig_007_a_migrated_project_is_ready_to_work() {
    let repo = openspec_repo();
    migrate(
        repo.path(),
        &["--from", "openspec", "--from", "superpowers", "--write"],
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("doctor  ok\n"))
    .stdout(predicate::str::contains("validate ok\n"))
    .stdout(predicate::str::is_match(r"\ngit: \d+ created, \d+ changed, \d+ removed\n").unwrap())
    .stdout(predicate::str::ends_with(
        "rollback: git restore . && git clean -fd\n",
    ));
    let root = repo.path();
    let toml = std::fs::read_to_string(root.join("specjutsu.toml")).unwrap();
    assert!(toml.contains("runner = \"cargo\""), "{toml}");
    let md = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(md.contains("<!-- specjutsu:begin -->"));
    assert!(root.join(".claude/skills/sj-apply/SKILL.md").is_file());

    let base = repo.git(&["rev-parse", "HEAD"]);
    repo.commit("chore: migrate to specjutsu");
    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", base.trim(), "--root"])
        .arg(root)
        .assert()
        .success();
}

fn read(repo: &support::Repo, path: &str) -> String {
    std::fs::read_to_string(repo.path().join(path)).unwrap()
}

/// An Antigravity workflow as OpenSpec generates it: only a description, and a body that
/// runs the OpenSpec CLI.
fn opsx_workflow(what: &str) -> String {
    format!(
        "---\ndescription: \"{what}\"\n---\n\n{what}.\n\n1. Run `openspec status --change \"<name>\" --json`.\n"
    )
}

#[test]
fn mig_008_openspecs_antigravity_workflows_are_removed() {
    let repo = support::Repo::with_lib("");
    repo.write("openspec/specs/math/spec.md", SPEC);
    repo.write("openspec/changes/.gitkeep", "");
    repo.write(
        ".agents/workflows/opsx-propose.md",
        &opsx_workflow("Propose a change"),
    );
    repo.write(
        ".agents/workflows/opsx-apply.md",
        &opsx_workflow("Apply a change"),
    );
    let deploy = "---\ndescription: \"Deploy the app\"\n---\n\nRun `pnpm deploy`.\n";
    repo.write(".agents/workflows/deploy.md", deploy);
    let notes = "---\ndescription: \"Our notes on the OpenSpec workflow\"\n---\n\nRead `docs/notes.md` first.\n";
    repo.write(".agents/workflows/opsx-notes.md", notes);
    repo.commit("antigravity project");

    migrate(repo.path(), &["--from", "openspec"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "remove .agents/workflows/opsx-propose.md (OpenSpec workflow)\n",
        ))
        .stdout(predicate::str::contains(
            "remove .agents/workflows/opsx-apply.md (OpenSpec workflow)\n",
        ))
        .stdout(predicate::str::contains("deploy.md").not())
        .stdout(predicate::str::contains("opsx-notes.md").not());
    migrate(repo.path(), &["--from", "openspec", "--write"])
        .assert()
        .success();

    assert!(!repo
        .path()
        .join(".agents/workflows/opsx-propose.md")
        .exists());
    assert!(!repo.path().join(".agents/workflows/opsx-apply.md").exists());
    assert_eq!(read(&repo, ".agents/workflows/deploy.md"), deploy);
    assert_eq!(read(&repo, ".agents/workflows/opsx-notes.md"), notes);
}

#[test]
fn mig_010_detected_agents_get_their_adapters() {
    let repo = support::Repo::with_lib("");
    repo.write("openspec/specs/math/spec.md", SPEC);
    repo.write("openspec/changes/.gitkeep", "");
    repo.write("AGENTS.md", "# Agents\n\nProject rules.\n");
    repo.write(".cursor/rules/style.mdc", "Use tabs.\n");
    repo.commit("cursor project");

    migrate(repo.path(), &["--from", "openspec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("change AGENTS.md (adapter)\n"))
        .stdout(predicate::str::contains(
            "create .cursor/hooks.json (adapter)\n",
        ));
    migrate(repo.path(), &["--from", "openspec", "--write"])
        .assert()
        .success();

    let agents = read(&repo, "AGENTS.md");
    assert!(
        agents.starts_with("# Agents\n\nProject rules.\n"),
        "{agents}"
    );
    assert!(agents.contains("sj next"), "{agents}");
    assert!(read(&repo, ".cursor/hooks.json").contains("sj hook cursor pre-edit"));
    assert!(repo
        .path()
        .join(".claude/skills/sj-apply/SKILL.md")
        .is_file());
    assert!(read(&repo, "CLAUDE.md").contains("sj next"));
}

/// MIG-011 (specjutsu/changes/migrate-ready): a pnpm + vitest OpenSpec project with Claude
/// settings, whose migration commit must pass the gate.
#[test]
fn mig_011_the_migration_commit_passes_the_gate() {
    let repo = support::Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    repo.write("package.json", r#"{"name":"root","private":true}"#);
    repo.write("pnpm-workspace.yaml", "packages:\n  - apps/*\n");
    repo.write("pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
    repo.write(
        "apps/api/package.json",
        r#"{"name":"api","devDependencies":{"vitest":"^3.0.0"}}"#,
    );
    repo.write("apps/api/src/math.ts", "export const add = (a, b) => a + b\n");
    repo.write(".gitignore", "node_modules/\n");
    repo.write(".claude/settings.json", "{\"permissions\":{\"allow\":[]}}\n");
    repo.write("openspec/specs/math/spec.md", SPEC);
    repo.write("openspec/changes/.gitkeep", "");
    repo.write("openspec/config.yaml", CONFIG_YAML);
    repo.commit("openspec project");

    migrate(repo.path(), &["--from", "openspec"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "change .gitignore (ignore the ledger)\n",
        ));
    migrate(repo.path(), &["--from", "openspec", "--write"])
        .assert()
        .success();
    let ignore = read(&repo, ".gitignore");
    assert!(ignore.lines().any(|l| l == ".specjutsu/ledger/"), "{ignore}");
    let toml = read(&repo, "specjutsu.toml");
    assert!(
        toml.contains("setup = \"pnpm install --frozen-lockfile\""),
        "{toml}"
    );

    let base = repo.git(&["rev-parse", "HEAD"]);
    repo.commit("chore: migrate to specjutsu");
    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", base.trim(), "--root"])
        .arg(repo.path())
        .assert()
        .success();
}
