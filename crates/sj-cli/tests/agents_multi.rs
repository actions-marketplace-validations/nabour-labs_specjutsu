//! Installing several agents at once, `auto` detection, and removal
//! (specjutsu/changes/more-adapters).

use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

const BEGIN: &str = "<!-- specjutsu:begin -->";

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.current_dir(root).env_remove("CLAUDE_PROJECT_DIR");
    c
}

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Every file under `root`, by relative path, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(d: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

const CONFIG: &str = "schema = 1\nmode = \"guided\"\n";

fn thirty_lines() -> String {
    (1..=30).map(|i| format!("- project rule {i}\n")).collect()
}

/// Whether `.cursor/hooks.json` has our `preToolUse` entry for edits.
fn has_cursor_hook(root: &Path) -> bool {
    let v: serde_json::Value = serde_json::from_str(&read(root, ".cursor/hooks.json")).unwrap();
    v["hooks"]["preToolUse"].as_array().is_some_and(|a| {
        a.iter()
            .any(|e| e["command"] == "sj hook cursor pre-edit" && e["matcher"] == "Write|Delete")
    })
}

/// Whether `.codex/config.toml` has our `PreToolUse` group for `apply_patch` edits.
fn has_codex_hook(root: &Path) -> bool {
    let v: toml::Value = toml::from_str(&read(root, ".codex/config.toml")).unwrap();
    v["hooks"]["PreToolUse"].as_array().is_some_and(|groups| {
        groups.iter().any(|g| {
            g["matcher"].as_str() == Some("^(apply_patch|Edit|Write)$")
                && g["hooks"].as_array().is_some_and(|hs| {
                    hs.iter().any(|h| {
                        h["type"].as_str() == Some("command")
                            && h["command"].as_str() == Some("sj hook codex pre-edit")
                    })
                })
        })
    })
}

#[test]
fn adapt_011_installing_two_agents_writes_agents_md_once() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    let rules = thirty_lines();
    put(root, "AGENTS.md", &rules);

    sj(root)
        .args(["init", "--agents", "codex,cursor"])
        .assert()
        .success();

    let agents = read(root, "AGENTS.md");
    assert!(agents.starts_with(&rules), "{agents}");
    assert_eq!(agents.matches(BEGIN).count(), 1, "{agents}");
    assert!(agents[rules.len()..].contains(BEGIN));
    assert!(has_codex_hook(root));
    assert!(has_cursor_hook(root));

    // Installing again changes nothing.
    let after = snapshot(root);
    sj(root)
        .args(["init", "--agents", "codex,cursor"])
        .assert()
        .success()
        .stdout("adapters up to date\n");
    assert_eq!(snapshot(root), after);
}

#[test]
fn adapt_013_install_then_remove_restores_agents_md_and_the_hook_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    put(root, "AGENTS.md", "# Shared rules\n\nBe kind.\n");
    put(
        root,
        ".codex/config.toml",
        "# team defaults\nmodel = \"gpt-5\"\n\n[features]\nweb_search = true\n",
    );
    put(
        root,
        ".cursor/hooks.json",
        "{\n  \"version\": 1,\n  \"hooks\": {\n    \"afterFileEdit\": [\n      { \"command\": \".cursor/hooks/format.sh\" }\n    ]\n  }\n}\n",
    );
    let before = snapshot(root);

    sj(root)
        .args(["init", "--agents", "codex,cursor,cline"])
        .assert()
        .success();
    assert!(has_codex_hook(root));
    assert!(has_cursor_hook(root));
    assert!(read(root, "AGENTS.md").contains(BEGIN));
    // The existing settings survive the install.
    assert!(read(root, ".codex/config.toml").starts_with("# team defaults\nmodel = \"gpt-5\"\n"));
    assert!(read(root, ".cursor/hooks.json").contains(".cursor/hooks/format.sh"));

    sj(root)
        .args(["init", "--remove-agents", "codex,cursor,cline"])
        .assert()
        .success();
    assert_eq!(snapshot(root), before);
}

#[test]
fn adapt_013_removal_deletes_hook_files_it_created() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    let before = snapshot(root);

    sj(root)
        .args(["init", "--agents", "codex,cursor"])
        .assert()
        .success();
    assert!(has_codex_hook(root));
    assert!(has_cursor_hook(root));

    sj(root)
        .args(["init", "--remove-agents", "codex,cursor"])
        .assert()
        .success();
    assert_eq!(snapshot(root), before);
    assert!(!root.join(".codex").exists());
    assert!(!root.join(".cursor").exists());
}

#[test]
fn adapt_014_auto_picks_the_configured_agents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    put(root, ".cursor/rules/style.mdc", "Use tabs.\n");
    put(root, ".clinerules/style.md", "Use tabs.\n");

    sj(root)
        .args(["init", "--agents", "auto"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "agents: cursor, cline, agents-md\n",
        ));

    assert!(has_cursor_hook(root));
    assert!(read(root, "AGENTS.md").contains(BEGIN));
    assert!(!root.join(".claude").exists());
    assert!(!root.join("CLAUDE.md").exists());
    assert!(!root.join(".codex").exists());
}

#[test]
fn adapt_014_auto_with_no_agent_files_installs_agents_md() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);

    sj(root)
        .args(["init", "--agents", "auto"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "agents: agents-md (no agent files found)\n",
        ));
    assert!(read(root, "AGENTS.md").contains(BEGIN));
}

#[test]
fn adapt_042_auto_detects_gemini_and_antigravity() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    put(
        root,
        ".gemini/settings.json",
        "{\n  \"theme\": \"GitHub\"\n}\n",
    );
    put(root, ".agents/rules/style.md", "Use tabs.\n");

    sj(root)
        .args(["init", "--agents", "auto"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "agents: gemini, antigravity, agents-md\n",
        ));

    assert!(read(root, ".gemini/settings.json").contains("sj hook gemini pre-edit"));
    assert!(read(root, ".agents/hooks.json").contains("sj hook antigravity pre-edit"));
    assert!(read(root, "AGENTS.md").contains(BEGIN));
    assert!(!root.join(".claude").exists());
}

#[test]
fn adapt_023_auto_detects_the_new_agents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    put(
        root,
        ".gemini/settings.json",
        "{\n  \"theme\": \"GitHub\"\n}\n",
    );
    std::fs::create_dir_all(root.join(".windsurf")).unwrap();
    put(root, ".kiro/steering/product.md", "# Product\n");

    sj(root)
        .args(["init", "--agents", "auto"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "agents: gemini, windsurf, kiro, agents-md\n",
        ));

    assert!(read(root, ".gemini/settings.json").contains("sj hook gemini pre-edit"));
    assert!(read(root, ".windsurf/hooks.json").contains("sj hook windsurf pre-edit"));
    assert!(read(root, ".kiro/hooks/specjutsu.json").contains("sj hook kiro pre-edit"));
    assert!(read(root, "AGENTS.md").contains(BEGIN));
    assert!(!root.join(".github").exists());
}

#[test]
fn adapt_023_auto_detects_copilot() {
    for marker in [
        ".github/copilot-instructions.md",
        ".github/hooks/audit.json",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        put(root, "specjutsu.toml", CONFIG);
        put(root, marker, "{}\n");

        sj(root)
            .args(["init", "--agents", "auto"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("agents: copilot, agents-md\n"));
        assert!(
            read(root, ".github/hooks/specjutsu.json").contains("sj hook copilot pre-edit"),
            "{marker}"
        );
    }
}

#[test]
fn adapt_024_install_then_remove_restores_the_new_agents_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    put(root, "specjutsu.toml", CONFIG);
    put(
        root,
        ".gemini/settings.json",
        "{\n  \"theme\": \"GitHub\"\n}\n",
    );
    put(
        root,
        ".windsurf/hooks.json",
        "{\n  \"hooks\": {\n    \"post_write_code\": [\n      {\"command\": \"./format.sh\"}\n    ]\n  }\n}\n",
    );
    let before = snapshot(root);
    let all = "copilot,gemini,windsurf,kiro,antigravity";

    sj(root).args(["init", "--agents", all]).assert().success();
    for (file, command) in [
        (".github/hooks/specjutsu.json", "sj hook copilot pre-edit"),
        (".gemini/settings.json", "sj hook gemini pre-edit"),
        (".windsurf/hooks.json", "sj hook windsurf pre-edit"),
        (".kiro/hooks/specjutsu.json", "sj hook kiro pre-edit"),
        (".agents/hooks.json", "sj hook antigravity pre-edit"),
    ] {
        assert!(read(root, file).contains(command), "{file}");
    }
    assert!(read(root, ".windsurf/hooks.json").contains("./format.sh"));
    for line in ["copilot: hook\n", "windsurf: hook\n", "kiro: hook\n"] {
        sj(root)
            .arg("doctor")
            .assert()
            .stdout(predicate::str::contains(line));
    }

    sj(root)
        .args(["init", "--remove-agents", all])
        .assert()
        .success();
    assert_eq!(snapshot(root), before);
    for d in [".github/hooks", ".kiro/hooks", ".agents"] {
        assert!(!root.join(d).exists(), "{d}");
    }
}
