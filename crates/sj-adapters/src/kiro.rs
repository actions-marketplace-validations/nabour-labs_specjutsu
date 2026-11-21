//! Kiro's `PreToolUse` hook in `.kiro/hooks/specjutsu.json`, in Kiro's hook file schema,
//! matching the `write` category
//! (every built-in file-write tool). The hook answers by exit code: 0 to allow, 2 to block
//! with the reason on stderr.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{read_opt, rel, remove_if_empty, Change};

/// Keys that may hold the edited file's path in `tool_input`, tried in order.
const PATH_KEYS: [&str; 2] = ["path", "file_path"];

/// The file the tool call edits.
pub fn target(input: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    for key in PATH_KEYS {
        if let Some(p) = v["tool_input"][key].as_str() {
            return Ok(p.to_string());
        }
    }
    bail!(
        "no file path in tool_input (looked for {})",
        PATH_KEYS.join(", ")
    )
}

/// Kiro's hook file schema: `version` `"v1"` and a list of named hooks.
const FILE: &str = r#"{
  "version": "v1",
  "hooks": [
    {
      "name": "specjutsu-pre-edit",
      "trigger": "PreToolUse",
      "matcher": "write",
      "action": {"type": "command", "command": "sj hook kiro pre-edit"}
    }
  ]
}
"#;
const COMMAND: &str = "sj hook kiro ";

fn path(root: &Path) -> PathBuf {
    root.join(".kiro").join("hooks").join("specjutsu.json")
}

/// Whether `.kiro/hooks/specjutsu.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| t.contains(COMMAND))
}

/// Write `.kiro/hooks/specjutsu.json`, which specjutsu owns.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    let p = path(root);
    let old = read_opt(&p)?;
    Ok(crate::plan::put(p, root, old.as_deref(), FILE.to_string())
        .into_iter()
        .collect())
}

/// Delete `.kiro/hooks/specjutsu.json`, with `.kiro/hooks/` and `.kiro/` if that leaves
/// them empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    match read_opt(&p)? {
        Some(t) if t.contains(COMMAND) => {
            std::fs::remove_file(&p)?;
            remove_if_empty(&root.join(".kiro").join("hooks"));
            remove_if_empty(&root.join(".kiro"));
            out.push(Change::Removed(rel(&p, root)));
        }
        _ => {}
    }
    Ok(out)
}
