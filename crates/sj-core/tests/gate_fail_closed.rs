//! GATE scenarios from openspec/changes/gate-fail-closed.

mod support;

use std::collections::BTreeMap;

use std::path::PathBuf;

use sj_core::gate::{evaluate, is_impl_file, run, Exemption, Input, Options};
use sj_core::trace::Report;
use sj_runners::cargo::{Outcome, Run, TestResult};
use support::Repo;

fn input() -> Input {
    Input {
        changes: BTreeMap::new(),
        trace: Report::default(),
        changed_files: vec![],
        exemptions: vec![],
        base_branch: "main".into(),
        head_run: None,
        base: BTreeMap::new(),
        allow_unverifiable: false,
        spec_root: "openspec".into(),
        unidentified: vec![],
        unidentified_known: 0,
    }
}

fn opts(repo: &Repo) -> Options {
    Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    }
}

const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_TEST: &str = "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n";

/// A spec file holding scenarios without IDs: (name, THEN text).
fn no_id_spec(scenarios: &[(&str, &str)]) -> String {
    let mut s =
        String::from("## Requirements\n\n### Requirement: Auth\nThe system SHALL auth.\n\n");
    for (name, then) in scenarios {
        s.push_str(&format!(
            "#### Scenario: {name}\n- **WHEN** called\n- **THEN** {then}\n\n"
        ));
    }
    s
}

const THREE: [(&str, &str); 3] = [
    ("Valid login", "a session starts"),
    ("Wrong password", "it is rejected"),
    ("Locked account", "it is refused"),
];

fn has(list: &[String], needle: &str) -> bool {
    list.iter().any(|l| l.contains(needle))
}

// ---------- file classification ----------

#[test]
fn gate_027_unrecognised_file_types_count_as_implementation() {
    let mut i = input();
    i.changed_files = vec!["scripts/build.sh".into(), "web/app.ts".into()];
    let r = evaluate(&i);
    assert!(!r.pass);
    assert!(
        has(
            &r.failures,
            "unspecified change: scripts/build.sh, web/app.ts"
        ),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_027_non_source_list() {
    for f in [
        "README.md",
        "docs/guide.mdx",
        "docs/diagram.png",
        "docs/shot.jpeg",
        "notes.txt",
        ".github/install-sj.sh",
        ".gitignore",
        "LICENSE",
        "openspec/config.yaml",
        "openspec/changes/x/.openspec.yaml",
        "crates/a/tests/support/mod.rs",
        "fixtures/trace-basic/src/lib.rs",
    ] {
        assert!(!is_impl_file(f, "openspec"), "{f} should be non-source");
    }
    for f in [
        "src/lib.rs",
        "web/app.ts",
        "scripts/build.sh",
        "Makefile",
        "backend/src/diff.py",
    ] {
        assert!(is_impl_file(f, "openspec"), "{f} should be implementation");
    }
}

#[test]
fn gate_028_manifest_lockfile_and_ci_changes_pass() {
    let mut i = input();
    i.changed_files = vec![
        ".github/workflows/ci.yml".into(),
        "Cargo.lock".into(),
        "Cargo.toml".into(),
        "crates/sj-core/Cargo.toml".into(),
        "apps/api/package.json".into(),
        "pnpm-lock.yaml".into(),
        "backend/pyproject.toml".into(),
        "backend/uv.lock".into(),
    ];
    let r = evaluate(&i);
    assert!(r.pass, "{:?}", r.failures);
}

// ---------- unidentified scenarios ----------

#[test]
fn gate_024_unidentified_scenarios_fail_and_known_ones_warn() {
    let mut i = input();
    i.unidentified = vec!["Logout clears session (openspec/specs/auth/spec.md:3)".into()];
    i.unidentified_known = 3;
    let r = evaluate(&i);
    assert!(!r.pass);
    assert!(
        has(
            &r.failures,
            "unidentified Logout clears session (openspec/specs/auth/spec.md:3)"
        ),
        "{:?}",
        r.failures
    );
    assert_eq!(
        r.warnings
            .iter()
            .filter(|w| w.contains("without ID"))
            .count(),
        1
    );
    assert!(
        has(&r.warnings, "3 scenarios without IDs"),
        "{:?}",
        r.warnings
    );
}

#[test]
fn gate_024_new_scenario_without_an_id_fails_the_gate() {
    let repo = Repo::with_lib(ADD_OK);
    repo.branch("feature");
    repo.write(
        "openspec/changes/logout/specs/auth/spec.md",
        &no_id_spec(&[("Logout clears session", "the session is gone")]),
    );
    repo.commit("docs(spec): logout");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert!(
        has(
            &r.failures,
            "unidentified Logout clears session (openspec/changes/logout/specs/auth/spec.md:6)"
        ),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_025_existing_scenarios_without_ids_only_warn_once() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.write("openspec/specs/auth/spec.md", &no_id_spec(&THREE));
    repo.commit("docs(spec): auth");
    repo.branch("feature");
    repo.add_change("math", &[("MATH-001", "Add numbers")]);
    repo.write("tests/math.rs", ADD_TEST);
    repo.write("src/lib.rs", ADD_OK);
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    let no_id: Vec<&String> = r
        .warnings
        .iter()
        .filter(|w| w.contains("without ID"))
        .collect();
    assert_eq!(no_id, ["3 scenarios without IDs"]);
}

#[test]
fn gate_025_rewording_a_scenario_without_an_id_fails() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", &no_id_spec(&THREE));
    repo.commit("docs(spec): auth");
    repo.branch("feature");
    let mut edited = THREE;
    edited[1].1 = "it is rejected with code 401";
    repo.write("openspec/specs/auth/spec.md", &no_id_spec(&edited));
    repo.commit("docs(spec): reword");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert!(
        has(&r.failures, "unidentified Wrong password"),
        "{:?}",
        r.failures
    );
    assert!(
        has(&r.warnings, "2 scenarios without IDs"),
        "{:?}",
        r.warnings
    );
}

#[test]
fn gate_025_moving_or_copying_a_scenario_without_an_id_is_not_a_change() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write(
        "openspec/changes/auth/specs/auth/spec.md",
        &no_id_spec(&THREE[..1]),
    );
    repo.write("openspec/specs/auth/spec.md", &no_id_spec(&THREE[1..]));
    repo.commit("docs(spec): auth");
    repo.branch("feature");
    // Archive: the delta's scenario moves into specs/, and a MODIFIED copy of another
    // appears in a new change.
    repo.git(&["rm", "-q", "-r", "openspec/changes/auth"]);
    repo.write("openspec/specs/auth/spec.md", &no_id_spec(&THREE));
    repo.write(
        "openspec/changes/lockout/specs/auth/spec.md",
        &no_id_spec(&THREE[2..]),
    );
    repo.commit("docs(openspec): archive auth");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
}

// ---------- exempted source changes ----------

#[test]
fn gate_029_chore_exemption_does_not_let_a_failing_test_through() {
    let mut i = input();
    i.changed_files = vec!["src/auth.rs".into()];
    i.exemptions = vec![Exemption {
        kind: "chore".into(),
        reason: "bump token lifetime".into(),
    }];
    i.head_run = Some(Run {
        success: false,
        results: vec![
            TestResult {
                path: "tests::a".into(),
                outcome: Outcome::Passed,
            },
            TestResult {
                path: "tests::b".into(),
                outcome: Outcome::Failed,
            },
        ],
    });
    let r = evaluate(&i);
    assert!(!r.pass);
    assert!(has(&r.failures, "failing tests::b"), "{:?}", r.failures);
}

#[test]
fn gate_029_exempted_source_change_runs_the_suite_at_head() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("tests/math.rs", ADD_TEST);
    repo.commit("test: math");
    repo.branch("feature");
    repo.write("src/lib.rs", ADD_BUG);
    repo.commit("chore: tweak\n\nSpecjutsu-Exempt: chore — bump token lifetime");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert!(
        has(&r.failures, "failing math_001_adds"),
        "{:?}",
        r.failures
    );
}
