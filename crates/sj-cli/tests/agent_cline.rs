//! `sj init --agents cline`: instructions only (specjutsu/changes/more-adapters).

use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root).env_remove("CLAUDE_PROJECT_DIR");
    c
}

/// Every file under `dir`, by path relative to `dir`, with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(d: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

#[test]
fn adapt_022_cline_installs_instructions_only() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".clinerules")).unwrap();
    std::fs::write(root.join(".clinerules/style.md"), "Use tabs.\n").unwrap();
    let rules_before = snapshot(&root.join(".clinerules"));

    sj(root)
        .args(["init", "--agents", "cline"])
        .assert()
        .success();

    let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("<!-- specjutsu:begin -->"), "{agents}");
    assert_eq!(snapshot(&root.join(".clinerules")), rules_before);
    assert!(!root.join(".cline").exists());

    sj(root)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("cline: instructions\n"));
}
