//! The pre-edit decision in git worktrees (specjutsu/changes/hook-worktree-root): the hook
//! judges a file by the work tree that holds it, not by the root it was given.

use std::path::Path;
use std::process::Command as Std;

use assert_cmd::Command;
use predicates::prelude::*;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Std::new("git")
        .args([
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.com",
        ])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const CONFIG: &str = "schema = 1\nmode = \"guided\"\n\n[[package]]\npath = \".\"\n\
                      sources = [\"src/**/*.rs\"]\n\n[[package.suite]]\nrunner = \"cargo\"\n\
                      tests = [\"tests/**/*.rs\"]\n";

/// An open change whose only task names `id`.
fn change(root: &Path, id: &str) {
    put(
        root,
        "openspec/changes/add-oauth-login/specs/auth/spec.md",
        &format!(
            "## ADDED Requirements\n\n### Requirement: Tokens\nThe system SHALL refresh tokens.\n\n\
             #### Scenario: {id} Refresh\n- **WHEN** a token expires\n- **THEN** it is refreshed\n"
        ),
    );
    put(
        root,
        "openspec/changes/add-oauth-login/tasks.md",
        &format!("- [ ] 1.1 Refresh tokens ({id})\n"),
    );
}

fn ledger(root: &Path, id: &str, result: &str) {
    put(
        root,
        ".specjutsu/ledger/runs.jsonl",
        &format!(
            "{{\"v\":1,\"run\":\"r\",\"ts\":1,\"tree\":\"t\",\"test\":\"auth::t\",\"ids\":[\"{id}\"],\"result\":\"{result}\"}}\n"
        ),
    );
}

/// A git repo at `<tmp>/demo` whose active task names AUTH-005 with no red record.
fn main_checkout(tmp: &Path) -> std::path::PathBuf {
    let main = tmp.join("demo");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q", "-b", "main"]);
    put(&main, "specjutsu.toml", CONFIG);
    put(&main, ".gitignore", ".specjutsu/\n.claude/worktrees/\n");
    put(&main, "src/auth.rs", "pub fn auth() {}\n");
    put(&main, "openspec/specs/.gitkeep", "");
    change(&main, "AUTH-005");
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "base"]);
    main
}

/// A linked worktree at `path` whose active task names AUTH-006.
fn worktree(main: &Path, path: &Path) {
    git(
        main,
        &["worktree", "add", "-q", path.to_str().unwrap(), "-b", "fix"],
    );
    change(path, "AUTH-006");
}

fn claude_edit(project_dir: &Path, file: &Path) -> Command {
    let path = file.to_string_lossy().replace('\\', "/");
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(["hook", "claude", "pre-edit"])
        .env("CLAUDE_PROJECT_DIR", project_dir)
        .current_dir(project_dir)
        .write_stdin(format!(
            r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{path}","old_string":"a","new_string":"b"}}}}"#
        ));
    c
}

// spec: ADAPT-040
#[test]
fn adapt_040_a_worktree_inside_the_project_is_judged_by_its_own_change() {
    let tmp = tempfile::tempdir().unwrap();
    let main = main_checkout(tmp.path());
    let wt = main.join(".claude/worktrees/fix");
    worktree(&main, &wt);
    let file = wt.join("src/auth.rs");

    // The worktree's own task has a red run: allowed, though the main checkout's has none.
    ledger(&wt, "AUTH-006", "failed");
    claude_edit(&main, &file).assert().code(0);

    // Without it, the worktree's task is what blocks.
    std::fs::remove_file(wt.join(".specjutsu/ledger/runs.jsonl")).unwrap();
    claude_edit(&main, &file)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AUTH-006"))
        .stderr(predicate::str::contains("AUTH-005").not());

    // The main checkout's own files are still judged by the main checkout.
    claude_edit(&main, &main.join("src/auth.rs"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AUTH-005"));
}

// spec: ADAPT-041
#[test]
fn adapt_041_a_worktree_outside_the_project_root_is_still_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let main = main_checkout(tmp.path());
    worktree(&main, &tmp.path().join("demo-fix"));

    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(["hook", "check-edit", "../demo-fix/src/auth.rs", "--root"])
        .arg(&main)
        .current_dir(&main)
        .env_remove("CLAUDE_PROJECT_DIR")
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "write a failing test for AUTH-006 first (sj test AUTH-006)",
        ));
}
