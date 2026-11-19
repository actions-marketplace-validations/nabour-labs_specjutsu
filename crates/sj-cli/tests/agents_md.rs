//! `sj init --agents <list>` parsing and the `agents-md` adapter
//! (openspec/changes/more-adapters).

use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

const BEGIN: &str = "<!-- specjutsu:begin -->";
const END: &str = "<!-- specjutsu:end -->";

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root).env_remove("CLAUDE_PROJECT_DIR");
    c
}

/// Every file under `root`, by relative path, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, root, out);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn adapt_012_unknown_agent_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("CLAUDE.md"), "# Project rules\n").unwrap();
    std::fs::write(root.join("AGENTS.md"), "# Shared rules\n").unwrap();
    let before = snapshot(root);

    sj(root)
        .args(["init", "--agents", "claude,windsurf"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "unknown agent windsurf (supported: claude, codex, cursor, cline, agents-md, auto)",
        ));

    assert_eq!(snapshot(root), before);
}

#[test]
fn adapt_015_agents_md_block_is_small_and_stable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    sj(root)
        .args(["init", "--agents", "agents-md"])
        .assert()
        .success();

    let text = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    let b = text.find(BEGIN).expect("begin marker");
    let e = text.find(END).expect("end marker") + END.len();
    let block = &text[b..e];
    assert!(block.len() <= 1200, "block is {} bytes", block.len());
    assert!(block.contains("sj next"), "{block}");
    assert!(block.contains("sj verify"), "{block}");
    assert!(!block.contains("sj-apply"), "names a skill: {block}");
    assert!(
        !root.join(".claude").exists(),
        "agents-md must not install Claude"
    );
    assert!(
        !root.join("CLAUDE.md").exists(),
        "agents-md must not install Claude"
    );

    sj(root).arg("update").assert().success();
    let after_first = snapshot(root);
    sj(root)
        .arg("update")
        .assert()
        .success()
        .stdout("adapters up to date\n");
    assert_eq!(snapshot(root), after_first);
}
