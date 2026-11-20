//! `sj update --dry-run` (openspec/changes/cli-conveniences): the update's own plan, printed.

use std::collections::BTreeMap;
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

/// A project with the Claude adapter installed by this build.
fn installed() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/.gitkeep", "");
    sj(dir.path(), &["init", "--agents", "claude"])
        .assert()
        .success();
    dir
}

/// `CLAUDE.md` and every file under `.claude/`, relative path to contents.
fn snapshot(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert("CLAUDE.md".into(), read(root, "CLAUDE.md"));
    let mut stack = vec![root.join(".claude")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read_to_string(&p).unwrap());
            }
        }
    }
    out
}

fn dry_run(root: &Path, extra: &[&str]) -> String {
    let mut args = vec!["update", "--dry-run"];
    args.extend_from_slice(extra);
    let out = sj(root, &args).assert().success().get_output().stdout.clone();
    String::from_utf8(out).unwrap()
}

// spec: ADAPT-036
#[test]
fn adapt_036_dry_run_lists_planned_writes_and_writes_nothing() {
    let dir = installed();
    // What an older specjutsu wrote: generated, unedited, different text.
    put(
        dir.path(),
        ".claude/skills/sj-apply/SKILL.md",
        &sj_adapters::claude::generated("# Old apply skill\n"),
    );
    let before = snapshot(dir.path());
    let out = dry_run(dir.path(), &[]);
    assert!(
        out.lines()
            .any(|l| l == "write .claude/skills/sj-apply/SKILL.md"),
        "{out}"
    );
    assert_eq!(snapshot(dir.path()), before);
}

// spec: ADAPT-037
#[test]
fn adapt_037_dry_run_marks_hand_edited_files_as_kept() {
    let dir = installed();
    let rel = ".claude/skills/sj-apply/SKILL.md";
    let edited = read(dir.path(), rel).replace("test first", "test first, always");
    put(dir.path(), rel, &edited);

    let out = dry_run(dir.path(), &[]);
    assert!(out.lines().any(|l| l == format!("keep {rel}")), "{out}");
    assert_eq!(read(dir.path(), rel), edited);

    let out = dry_run(dir.path(), &["--force"]);
    assert!(out.lines().any(|l| l == format!("write {rel}")), "{out}");
    assert_eq!(read(dir.path(), rel), edited);
}

// spec: ADAPT-038
#[test]
fn adapt_038_dry_run_lists_files_an_update_would_remove() {
    let dir = installed();
    let rel = ".claude/skills/sj-old/SKILL.md";
    put(
        dir.path(),
        rel,
        &sj_adapters::claude::generated("# A skill this version no longer ships\n"),
    );
    let out = dry_run(dir.path(), &[]);
    assert!(out.lines().any(|l| l == format!("remove {rel}")), "{out}");
    assert!(dir.path().join(rel).is_file());

    // A clean install has nothing to do.
    std::fs::remove_dir_all(dir.path().join(".claude/skills/sj-old")).unwrap();
    sj(dir.path(), &["update", "--dry-run"])
        .assert()
        .success()
        .stdout("adapters up to date\n");
}
