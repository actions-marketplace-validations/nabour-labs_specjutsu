//! GATE scenarios for the trace baseline on base (specjutsu/changes/brownfield-adoption).

mod support;

use std::path::PathBuf;

use sj_core::gate::{run, Options};
use support::{spec_md, Repo};

fn opts(repo: &Repo) -> Options {
    Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    }
}

/// A change that touches no scenario and no implementation file.
fn docs_only_change(repo: &Repo) {
    repo.branch("feature");
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");
}

#[test]
fn gate_050_baselined_gaps_collapse_into_one_warning() {
    let repo = Repo::with_lib("pub fn lib() {}\n");
    let ids: Vec<String> = (1..=2981).map(|n| format!("BIG-{n:04}")).collect();
    let scenarios: Vec<(&str, &str)> = ids.iter().map(|id| (id.as_str(), "Untested")).collect();
    repo.write("openspec/specs/big/spec.md", &spec_md(&scenarios));
    let baseline: String = ids.iter().map(|id| format!("gap {id}\n")).collect();
    repo.write("openspec/trace-baseline.txt", &baseline);
    repo.commit("base with baseline");
    docs_only_change(&repo);

    let r = run(&opts(&repo)).unwrap();
    assert!(
        r.warnings
            .iter()
            .any(|w| w == "known gaps: 2981 (trace-baseline.txt)"),
        "{:?}",
        r.warnings.iter().take(5).collect::<Vec<_>>()
    );
    for id in &ids {
        let line = format!("gap {id} ");
        assert!(
            !r.warnings.iter().any(|w| w.starts_with(&line)),
            "unexpected warning for {id}"
        );
    }
}

#[test]
fn gate_051_baselined_orphan_does_not_fail_the_gate() {
    let repo = Repo::with_lib("pub fn lib() {}\n");
    // AUTH is a known prefix, so a test naming the missing AUTH-009 is an orphan.
    repo.write(
        "openspec/specs/auth/spec.md",
        &spec_md(&[("AUTH-001", "Logs in")]),
    );
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_009_old() { assert!(true); }\n",
    );
    repo.write(
        "openspec/trace-baseline.txt",
        "orphan AUTH-009 tests/auth.rs auth_009_old\n",
    );
    repo.commit("base with baselined orphan");
    docs_only_change(&repo);

    let r = run(&opts(&repo)).unwrap();
    assert!(
        !r.failures
            .iter()
            .any(|f| f.contains("orphan AUTH-009 auth_009_old")),
        "{:?}",
        r.failures
    );
}
