//! `sj import speckit` CLI scenarios (openspec/changes/speckit-import).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

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

/// A git repository holding the Spec Kit fixture, committed.
fn repo() -> support::Repo {
    let repo = support::Repo::with_lib("");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sj-core/tests/fixtures/speckit");
    copy(&fixture, repo.path());
    repo.commit("spec kit project");
    repo
}

fn import(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(["import", "speckit"])
        .args(args)
        .arg("--root")
        .arg(root);
    c
}

fn status(repo: &support::Repo) -> String {
    repo.git(&["status", "--porcelain", "--untracked-files=all"])
}

#[test]
fn imp_001_dry_run_writes_nothing() {
    let repo = repo();
    import(repo.path(), &[])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "import: 2 features (dry run; --write converts them)\n",
        ))
        .stdout(predicate::str::contains(
            "chrome-ai-agent  finished  6 requirements, 5 scenarios, 3 gaps\n",
        ))
        .stdout(predicate::str::contains(
            "  specs/001-chrome-ai-agent/plan.md → openspec/changes/archive/2026-09-27-chrome-ai-agent/design.md\n",
        ))
        .stdout(predicate::str::contains(
            "  → openspec/specs/chrome-ai-agent/spec.md\n",
        ))
        .stdout(predicate::str::contains("history  open  2 requirements, 1 scenario, 1 gap\n"))
        .stdout(predicate::str::contains("gap FR-007 (no scenario)\n"))
        .stdout(predicate::str::contains(
            "principles .specify/memory/constitution.md → openspec/principles.md\n",
        ));
    assert_eq!(status(&repo), "");
}

#[test]
fn imp_002_write_creates_specs_and_an_archived_change() {
    let repo = repo();
    import(repo.path(), &["--write"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("import: wrote 2 features\n"));
    let root = repo.path();
    assert!(root
        .join("openspec/specs/chrome-ai-agent/spec.md")
        .is_file());
    let archive = root.join("openspec/changes/archive/2026-09-27-chrome-ai-agent");
    let src = root.join("specs/001-chrome-ai-agent");
    for (from, to) in [
        ("plan.md", "design.md"),
        ("research.md", "research.md"),
        ("data-model.md", "data-model.md"),
        ("quickstart.md", "quickstart.md"),
        ("contracts/api.md", "contracts/api.md"),
        ("checklists/requirements.md", "checklists/requirements.md"),
        ("tasks.md", "tasks.md"),
    ] {
        assert_eq!(
            std::fs::read(src.join(from)).unwrap(),
            std::fs::read(archive.join(to)).unwrap(),
            "{from}"
        );
    }
    assert!(archive.join("proposal.md").is_file());
}

#[test]
fn imp_003_an_open_feature_becomes_an_open_change() {
    let repo = repo();
    import(repo.path(), &["--write"]).assert().success();
    let change = repo.path().join("openspec/changes/history");
    assert_eq!(
        std::fs::read(change.join("tasks.md")).unwrap(),
        std::fs::read(repo.path().join("specs/002-history/tasks.md")).unwrap()
    );
    // The FR gap is an open question that `sj spec check` reports.
    Command::cargo_bin("sj")
        .unwrap()
        .args(["spec", "check", "history", "--root"])
        .arg(repo.path())
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "clarify add a scenario for FR-001 (specs/history/spec.md:",
        ));
    // Open changes are listed with their task progress.
    Command::cargo_bin("sj")
        .unwrap()
        .args(["spec", "list", "--root"])
        .arg(repo.path())
        .assert()
        .success()
        .stdout("history  1/4 tasks  1 scenario\n");
}

#[test]
fn imp_006_sources_are_untouched_and_principles_kept() {
    let repo = repo();
    import(repo.path(), &["--write"]).assert().success();
    let root = repo.path();
    assert_eq!(
        std::fs::read(root.join("openspec/principles.md")).unwrap(),
        std::fs::read(root.join(".specify/memory/constitution.md")).unwrap()
    );
    for line in status(&repo).lines() {
        assert!(line.starts_with("?? openspec/"), "{line}");
    }
}

#[test]
fn imp_001_existing_targets_need_force() {
    let repo = repo();
    import(repo.path(), &["--write"]).assert().success();
    let spec = repo.path().join("openspec/specs/chrome-ai-agent/spec.md");
    std::fs::write(&spec, "edited\n").unwrap();
    import(repo.path(), &["--write"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--force"));
    assert_eq!(std::fs::read_to_string(&spec).unwrap(), "edited\n");
    import(repo.path(), &["--write", "--force"])
        .assert()
        .success();
    assert_ne!(std::fs::read_to_string(&spec).unwrap(), "edited\n");
}

#[test]
fn imp_001_json_lists_the_plan() {
    let repo = repo();
    let out = import(repo.path(), &["--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["features"][0]["name"], "chrome-ai-agent");
    assert_eq!(v["features"][0]["gaps"][2], "FR-007");
    assert_eq!(v["written"], false);
    assert_eq!(status(&repo), "");
}
