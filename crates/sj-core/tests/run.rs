//! RUN scenarios covered by sj-core (specjutsu/changes/test-ledger).

mod support;

use std::path::Path;

use sj_core::run::{excerpt, resolve, run_tests, Scope};
use sj_runners::cargo::Outcome;
use support::{spec_md, Repo};

/// FOO-001 and FOO-002 specified; tests `foo_001_adds`, `xfoo_001_adds` (unmapped, but the
/// `foo_001_adds` libtest filter also matches it) and `foo_002_subtracts`.
fn fixture() -> Repo {
    let repo = Repo::with_lib("");
    repo.write(
        "openspec/specs/math/spec.md",
        &spec_md(&[("FOO-001", "Add"), ("FOO-002", "Subtract")]),
    );
    repo.write(
        "tests/math.rs",
        "#[test]\nfn foo_001_adds() {}\n\n#[test]\nfn xfoo_001_adds() {}\n\n#[test]\nfn foo_002_subtracts() {}\n",
    );
    repo
}

fn spec_root() -> &'static Path {
    Path::new("openspec")
}

fn names(run: &sj_core::run::TestRun) -> Vec<&str> {
    let mut v: Vec<&str> = run.tests.iter().map(|t| t.path.as_str()).collect();
    v.sort();
    v
}

#[test]
fn run_001_no_scope_runs_the_whole_workspace() {
    let repo = fixture();
    let run = run_tests(repo.path(), spec_root(), &Scope::All).unwrap();
    assert!(run.pass && !run.build_failed);
    assert_eq!(
        names(&run),
        ["foo_001_adds", "foo_002_subtracts", "xfoo_001_adds"]
    );
    assert_eq!((run.passed, run.failed, run.ignored), (3, 0, 0));
    let foo1 = run.tests.iter().find(|t| t.path == "foo_001_adds").unwrap();
    assert_eq!(foo1.ids, ["FOO-001"]);
    assert_eq!(foo1.result, Outcome::Passed);
}

#[test]
fn run_002_scenario_scope_runs_only_mapped_tests() {
    let repo = fixture();
    let run = run_tests(repo.path(), spec_root(), &Scope::Scenario("FOO-001".into())).unwrap();
    assert_eq!(names(&run), ["foo_001_adds"]);
    assert_eq!(run.passed, 1);
}

#[test]
fn run_002_scope_argument_shaped_like_an_id_is_a_scenario() {
    assert_eq!(Scope::parse(None), Scope::All);
    assert_eq!(
        Scope::parse(Some("GATE-004")),
        Scope::Scenario("GATE-004".into())
    );
    assert_eq!(
        Scope::parse(Some("sj-core")),
        Scope::Package("sj-core".into())
    );
}

#[test]
fn run_003_scenario_without_tests_is_an_error() {
    let repo = fixture();
    let err = resolve(repo.path(), spec_root(), &Scope::Scenario("FOO-009".into()))
        .expect_err("unmapped scenario must be an error");
    assert!(
        err.to_string().contains("FOO-009 has no mapped test"),
        "{err}"
    );
}

#[test]
fn run_004_package_scope_selects_one_package() {
    let repo = Repo::with_lib("");
    repo.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n",
    );
    for p in ["a", "b"] {
        repo.write(
            &format!("{p}/Cargo.toml"),
            &format!("[package]\nname = \"{p}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        );
        repo.write(&format!("{p}/src/lib.rs"), "");
        repo.write(
            &format!("{p}/tests/t.rs"),
            &format!("#[test]\nfn in_{p}() {{}}\n"),
        );
    }
    let run = run_tests(repo.path(), spec_root(), &Scope::Package("a".into())).unwrap();
    assert_eq!(names(&run), ["in_a"]);
}

#[test]
fn run_006_failures_carry_ids_and_an_excerpt() {
    let repo = fixture();
    repo.write(
        "tests/math.rs",
        "#[test]\nfn foo_001_adds() { panic!(\"boom 001\"); }\n\n#[test]\nfn foo_002_subtracts() {}\n",
    );
    let run = run_tests(repo.path(), spec_root(), &Scope::All).unwrap();
    assert!(!run.pass);
    assert_eq!((run.passed, run.failed), (1, 1));
    let f = run
        .tests
        .iter()
        .find(|t| t.result == Outcome::Failed)
        .unwrap();
    assert_eq!(f.path, "foo_001_adds");
    assert_eq!(f.ids, ["FOO-001"]);
    let ex = f.excerpt.as_ref().unwrap();
    assert!(ex.lines.iter().any(|l| l.contains("boom 001")), "{ex:?}");
    let ok = run
        .tests
        .iter()
        .find(|t| t.result == Outcome::Passed)
        .unwrap();
    assert!(ok.excerpt.is_none());
}

#[test]
fn run_007_long_failure_output_is_truncated() {
    let text: Vec<String> = (1..=50).map(|i| format!("line {i}")).collect();
    let ex = excerpt(&text.join("\n"), 20);
    assert_eq!(ex.lines.len(), 20);
    assert_eq!(ex.lines[19], "line 20");
    assert_eq!(ex.omitted, 30);
    let short = excerpt("a\nb", 20);
    assert_eq!((short.lines.len(), short.omitted), (2, 0));
}

#[test]
fn run_008_build_failure_is_reported_with_errors() {
    let repo = fixture();
    repo.write("tests/math.rs", "#[test]\nfn foo_001_adds() { nope(); }\n");
    let run = run_tests(repo.path(), spec_root(), &Scope::Scenario("FOO-001".into())).unwrap();
    assert!(run.build_failed && !run.pass);
    assert!(run.tests.is_empty());
    assert!(run.build_errors.iter().any(|l| l.starts_with("error")));
    assert_eq!(
        run.selected,
        [("foo_001_adds".to_string(), vec!["FOO-001".to_string()])]
    );
}

#[test]
fn run_009_not_a_cargo_project_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = run_tests(dir.path(), spec_root(), &Scope::All)
        .err()
        .unwrap();
    assert!(err.to_string().contains("no Cargo.toml"), "{err}");
}
