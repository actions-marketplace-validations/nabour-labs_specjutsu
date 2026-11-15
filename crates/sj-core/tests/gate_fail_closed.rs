//! GATE scenarios from openspec/changes/gate-fail-closed.

mod support;

use std::collections::BTreeMap;

use sj_core::gate::{evaluate, is_impl_file, Input};
use sj_core::trace::Report;

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
    }
}

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
