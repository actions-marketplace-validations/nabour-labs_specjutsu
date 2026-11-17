//! The static skill check (openspec/changes/workflow-skills).

use std::path::Path;

use sj_adapters::skills::{self, Problem};

const SKILLS: [&str; 9] = [
    "sj-explore",
    "sj-propose",
    "sj-clarify",
    "sj-principles",
    "sj-apply",
    "sj-debug",
    "sj-review",
    "sj-verify",
    "sj-finish",
];

const COVERAGE: &str = "| Tool | Item | Skill |
| --- | --- | --- |
| Superpowers | brainstorming | sj-explore |
| Superpowers | writing-plans | sj-propose |
| Superpowers | executing-plans | sj-apply |
| Superpowers | test-driven-development | sj-apply |
| Superpowers | subagent-driven-development | sj-apply |
| Superpowers | dispatching-parallel-agents | sj-apply |
| Superpowers | systematic-debugging | sj-debug |
| Superpowers | requesting-code-review | sj-review |
| Superpowers | receiving-code-review | sj-review |
| Superpowers | verification-before-completion | sj-verify |
| Superpowers | finishing-a-development-branch | sj-finish |
| Superpowers | using-git-worktrees | sj-finish |
| OpenSpec | explore | sj-explore |
| OpenSpec | propose | sj-propose |
| OpenSpec | apply | sj-apply |
| OpenSpec | archive | sj-finish |
| OpenSpec | sync | sj-finish |
| OpenSpec | update | sj-propose |
| Spec Kit | constitution | sj-principles |
| Spec Kit | specify | sj-propose |
| Spec Kit | clarify | sj-clarify |
| Spec Kit | plan | sj-propose |
| Spec Kit | tasks | sj-propose |
| Spec Kit | analyze | sj-propose |
| Spec Kit | checklist | sj-review |
| Spec Kit | implement | sj-apply |
";

/// A body that satisfies every per-skill rule.
fn body(name: &str) -> String {
    let needs = match name {
        "sj-apply" => "Run `sj next`, then `sj test AUTH-001`, then `sj task done`.",
        "sj-debug" => "Reproduce it with `sj test AUTH-001`.",
        "sj-verify" => "Run `sj verify`.",
        "sj-finish" => "Run `sj verify`, then `sj spec archive`.",
        "sj-propose" => "Run `sj spec new`, then `sj spec validate` and `sj spec check`. Read principles.md and record each deviation in design.md.",
        "sj-clarify" => "Run `sj spec check`.",
        "sj-review" => "Read principles.md and record each deviation in design.md.",
        _ => "Run `sj status`.",
    };
    format!("# {name}\n\n{needs}\n")
}

fn skill_md(name: &str, body: &str) -> String {
    format!("---\nname: {name}\ndescription: Use when testing the check.\n---\n\n{body}")
}

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A skills folder that checks clean.
fn valid() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for s in SKILLS {
        put(dir.path(), &format!("{s}/SKILL.md"), &skill_md(s, &body(s)));
    }
    put(dir.path(), "COVERAGE.md", COVERAGE);
    dir
}

fn messages(dir: &Path) -> Vec<String> {
    skills::check(dir)
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect()
}

#[test]
fn skill_001_a_complete_fixture_checks_clean() {
    let dir = valid();
    assert_eq!(messages(dir.path()), Vec::<String>::new());
}

#[test]
fn skill_001_coverage_is_complete() {
    let dir = valid();
    let coverage = COVERAGE
        .replace("| Superpowers | brainstorming | sj-explore |\n", "")
        .replace(
            "| OpenSpec | sync | sj-finish |",
            "| OpenSpec | sync | sj-finish |\n| OpenSpec | sync | sj-apply |",
        )
        .replace(
            "| Spec Kit | plan | sj-propose |",
            "| Spec Kit | plan | sj-plan |",
        );
    put(dir.path(), "COVERAGE.md", &coverage);
    let got = messages(dir.path());
    for want in [
        "coverage: Superpowers brainstorming is not mapped",
        "coverage: OpenSpec sync is mapped twice",
        "coverage: Spec Kit plan maps to unknown skill sj-plan",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

#[test]
fn skill_001_a_missing_skill_or_coverage_file_fails() {
    let dir = valid();
    std::fs::remove_dir_all(dir.path().join("sj-clarify")).unwrap();
    std::fs::remove_file(dir.path().join("COVERAGE.md")).unwrap();
    let got = messages(dir.path());
    assert!(
        got.iter().any(|m| m == "sj-clarify: missing SKILL.md"),
        "{got:?}"
    );
    assert!(
        got.iter().any(|m| m == "coverage: missing COVERAGE.md"),
        "{got:?}"
    );
}

#[test]
fn skill_002_an_oversized_or_foreign_skill_fails_the_check() {
    let dir = valid();
    let big = format!("{}{}", body("sj-propose"), "x".repeat(7000));
    put(
        dir.path(),
        "sj-propose/SKILL.md",
        &skill_md("sj-propose", &big),
    );
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md(
            "sj-apply",
            &format!("{}\nThen run `openspec status`.\n", body("sj-apply")),
        ),
    );
    let got = messages(dir.path());
    let n = big.len();
    assert!(
        got.iter()
            .any(|m| m == &format!("sj-propose: body is {n} bytes (limit 6000)")),
        "{got:?}"
    );
    assert!(
        got.iter()
            .any(|m| m == "sj-apply: names a foreign command (openspec status)"),
        "{got:?}"
    );
}

#[test]
fn skill_002_other_foreign_commands_and_bad_frontmatter() {
    let dir = valid();
    put(
        dir.path(),
        "sj-explore/SKILL.md",
        &skill_md(
            "sj-explore",
            "Use /speckit.plan, then superpowers:brainstorming. Specs use OpenSpec's format.\n",
        ),
    );
    put(
        dir.path(),
        "sj-debug/SKILL.md",
        "---\nname: debug\ndescription: Debugging.\n---\n\nRun `sj test`.\n",
    );
    put(dir.path(), "sj-verify/SKILL.md", "Run `sj verify`.\n");
    let got = messages(dir.path());
    for want in [
        "sj-explore: names a foreign command (/speckit)",
        "sj-explore: names a foreign command (superpowers:)",
        "sj-debug: name is debug, expected sj-debug",
        "sj-debug: description does not say when to use it",
        "sj-verify: no frontmatter",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    // Naming the format is fine; only its commands are foreign.
    assert!(!got.iter().any(|m| m.contains("(OpenSpec")), "{got:?}");
}

#[test]
fn skill_003_each_skill_names_its_required_commands() {
    let dir = valid();
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md("sj-apply", "Run `sj next` and `sj test AUTH-001`.\n"),
    );
    put(
        dir.path(),
        "sj-finish/SKILL.md",
        &skill_md("sj-finish", "Open the PR.\n"),
    );
    let got = messages(dir.path());
    for want in [
        "sj-apply: does not name `sj task done`",
        "sj-finish: does not name `sj verify`",
        "sj-finish: does not name `sj spec archive`",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    assert!(
        !got.iter().any(|m| m == "sj-apply: does not name `sj next`"),
        "{got:?}"
    );
}

#[test]
fn skill_004_planning_and_review_reference_the_principles_file() {
    let dir = valid();
    put(
        dir.path(),
        "sj-review/SKILL.md",
        &skill_md("sj-review", "Review the diff.\n"),
    );
    put(
        dir.path(),
        "sj-propose/SKILL.md",
        &skill_md(
            "sj-propose",
            "Run `sj spec new`, `sj spec validate` and `sj spec check`. Read principles.md.\n",
        ),
    );
    let got = messages(dir.path());
    for want in [
        "sj-review: does not name principles.md",
        "sj-review: does not record deviations in design.md",
        "sj-propose: does not record deviations in design.md",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}
