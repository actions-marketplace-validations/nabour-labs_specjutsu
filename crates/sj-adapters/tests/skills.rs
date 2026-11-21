//! The static skill check (specjutsu/changes/workflow-skills).

use std::path::Path;

use sj_adapters::skills::{self, Problem};

const SKILLS: [&str; 10] = [
    "sj-explore",
    "sj-propose",
    "sj-clarify",
    "sj-principles",
    "sj-apply",
    "sj-debug",
    "sj-review",
    "sj-verify",
    "sj-finish",
    "sj-onboard",
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
| OpenSpec | onboard | sj-onboard |
| Spec Kit | constitution | sj-principles |
| Spec Kit | specify | sj-propose |
| Spec Kit | clarify | sj-clarify |
| Spec Kit | plan | sj-propose |
| Spec Kit | tasks | sj-propose |
| Spec Kit | analyze | sj-propose |
| Spec Kit | checklist | sj-review |
| Spec Kit | implement | sj-apply |
";

/// An onboarding body that satisfies SKILL-005..007.
const ONBOARD: &str = "Stop if `git status` shows uncommitted changes or \
`changes/sj-onboard-demo/` exists. Create the change `sj-onboard-demo` with `sj spec new`, \
run `sj test DEMO-001` red, then green, then `sj task done`, then `sj verify`. At the end, \
delete `changes/sj-onboard-demo/`, the demo test and the demo code. Don't archive it.";

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
        "sj-onboard" => ONBOARD,
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
        .replace("| OpenSpec | onboard | sj-onboard |\n", "")
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
        "coverage: OpenSpec onboard is not mapped",
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
    std::fs::remove_dir_all(dir.path().join("sj-onboard")).unwrap();
    std::fs::remove_file(dir.path().join("COVERAGE.md")).unwrap();
    let got = messages(dir.path());
    assert!(
        got.iter().any(|m| m == "sj-onboard: missing SKILL.md"),
        "{got:?}"
    );
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

#[test]
fn skill_001_this_repos_skills_check_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    for s in SKILLS {
        assert!(dir.join(s).join("SKILL.md").is_file(), "{s} is not shipped");
    }
    assert_eq!(messages(&dir), Vec::<String>::new());
}

#[test]
fn skill_002_this_repos_skill_bodies_are_within_the_limit() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    for s in SKILLS {
        let text = std::fs::read_to_string(dir.join(s).join("SKILL.md")).unwrap();
        assert!(text.len() <= 6000 + 400, "{s}: {} bytes", text.len());
    }
}

#[test]
fn skill_001_openspec_onboard_maps_to_sj_onboard() {
    let dir = valid();
    let coverage = COVERAGE.replace("| OpenSpec | onboard | sj-onboard |\n", "");
    put(dir.path(), "COVERAGE.md", &coverage);
    std::fs::remove_dir_all(dir.path().join("sj-onboard")).unwrap();
    let got = messages(dir.path());
    for want in [
        "coverage: OpenSpec onboard is not mapped",
        "sj-onboard: missing SKILL.md",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

fn onboard(body: &str) -> Vec<String> {
    let dir = valid();
    put(
        dir.path(),
        "sj-onboard/SKILL.md",
        &skill_md("sj-onboard", body),
    );
    messages(dir.path())
}

// spec: SKILL-005
#[test]
fn skill_005_onboard_names_the_full_loop() {
    let got = onboard("Run `sj verify`, then `sj spec new`, then `sj test`.\n");
    for want in [
        "sj-onboard: does not name `sj task done`",
        "sj-onboard: names `sj verify` before `sj spec new`",
        "sj-onboard: does not name the scratch change sj-onboard-demo",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    assert!(!onboard(ONBOARD).iter().any(|m| m.starts_with("sj-onboard")));
}

// spec: SKILL-006
#[test]
fn skill_006_onboard_cleans_up_after_itself() {
    let got = onboard(&ONBOARD.replace(
        "At the end, delete `changes/sj-onboard-demo/`, the demo test and the demo code. Don't archive it.",
        "At the end, run `sj spec archive sj-onboard-demo`.",
    ));
    for want in [
        "sj-onboard: does not delete changes/sj-onboard-demo/, the demo test and the demo code",
        "sj-onboard: runs `sj spec archive`",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

// spec: SKILL-007
#[test]
fn skill_007_onboard_refuses_a_dirty_start() {
    let got = onboard(&ONBOARD.replace(
        "Stop if `git status` shows uncommitted changes or `changes/sj-onboard-demo/` exists. ",
        "",
    ));
    assert!(
        got.iter().any(|m| m
            == "sj-onboard: does not stop on uncommitted changes or an existing sj-onboard-demo"),
        "{got:?}"
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let text = std::fs::read_to_string(dir.join("sj-onboard/SKILL.md")).unwrap();
    assert!(text.len() <= 6000 + 400, "{} bytes", text.len());
    assert_eq!(messages(&dir), Vec::<String>::new());
}

/// A setup skill body that satisfies SKILL-008 and SKILL-009.
const SETUP: &str = "Run `sj --version`. If it is missing, ask the user, then run `install.sh`, \
or `cargo install specjutsu` on Windows. Keep an existing `specjutsu.toml`. Run `sj adopt` on \
an existing project, then `sj init --agents auto`, then `sj doctor` and `sj status`.";

fn setup(body: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specjutsu/SKILL.md",
        &skill_md("specjutsu", body),
    );
    skills::check_setup(dir.path())
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect()
}

#[test]
fn skill_008_the_setup_skill_names_the_setup_steps_in_order() {
    assert_eq!(setup(SETUP), Vec::<String>::new());
    let got = setup(&SETUP.replace("`sj doctor` and ", ""));
    assert!(
        got.contains(&"specjutsu: does not name `sj doctor`".to_string()),
        "{got:?}"
    );
    let swapped = "Ask the user. Keep `specjutsu.toml`. Run `sj --version`, `install.sh`, \
`cargo install specjutsu`, `sj init --agents auto`, then `sj adopt`, `sj doctor`, `sj status`.";
    let got = setup(swapped);
    assert!(
        got.contains(&"specjutsu: names `sj init --agents` before `sj adopt`".to_string()),
        "{got:?}"
    );
    let got = setup(&SETUP.replace("`cargo install specjutsu`", "a build from source"));
    assert!(
        got.contains(&"specjutsu: does not name `cargo install specjutsu`".to_string()),
        "{got:?}"
    );
    let dir = tempfile::tempdir().unwrap();
    let got: Vec<String> = skills::check_setup(dir.path())
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect();
    assert_eq!(got, vec!["specjutsu: missing SKILL.md".to_string()]);
}

#[test]
fn skill_009_the_setup_skill_never_forces() {
    let got = setup(&format!("{SETUP} If that fails, run `sj init --force`."));
    assert!(
        got.contains(&"specjutsu: passes `--force`".to_string()),
        "{got:?}"
    );
    let got = setup(
        &SETUP
            .replace("ask the user, then ", "")
            .replace("Keep an existing `specjutsu.toml`. ", ""),
    );
    assert!(
        got.contains(&"specjutsu: does not ask before installing".to_string()),
        "{got:?}"
    );
    assert!(
        got.contains(&"specjutsu: does not keep an existing specjutsu.toml".to_string()),
        "{got:?}"
    );
}

#[test]
fn skill_008_this_repos_setup_skill_checks_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got: Vec<String> = skills::check_setup(&dir)
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect();
    assert_eq!(got, Vec::<String>::new());
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills/specjutsu");
    assert!(!assets.exists(), "the setup skill must not be embedded");
}
