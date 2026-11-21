//! Kiro's `PreToolUse` hook in `.kiro/hooks/specjutsu.json`, matching the `write` category
//! (every built-in file-write tool). The hook answers by exit code: 0 to allow, 2 to block
//! with the reason on stderr.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::claude::Change;
use crate::hook_file::{command_is, HookFile};
use crate::jsonedit::Node;

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

fn is_ours(entry: &Node) -> bool {
    command_is(entry, "command", "sj hook kiro ")
}

const FILE: HookFile = HookFile {
    path: &[".kiro", "hooks", "specjutsu.json"],
    event: "PreToolUse",
    entry: r#"{"matcher": "write", "command": "sj hook kiro pre-edit"}"#,
    ours: is_ours,
    versioned: true,
};

/// Whether `.kiro/hooks/specjutsu.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    FILE.is_installed(root)
}

/// Add the `PreToolUse` entry.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    FILE.plan(root)
}

/// Remove the entry; the file goes when nothing else is in it, with `.kiro/hooks/` and
/// `.kiro/` if that leaves them empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    FILE.remove(root)
}
