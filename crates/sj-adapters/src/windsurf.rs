//! Windsurf's `pre_write_code` hook in `.windsurf/hooks.json`. The hook answers by exit code:
//! 0 to allow, 2 to block with the reason on stderr.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::claude::Change;
use crate::hook_file::{command_is, HookFile};
use crate::jsonedit::Node;

/// The file the tool call edits, from `tool_info.file_path`.
pub fn target(input: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    match v["tool_info"]["file_path"].as_str() {
        Some(p) => Ok(p.to_string()),
        None => bail!("no file path in tool_info.file_path"),
    }
}

fn is_ours(entry: &Node) -> bool {
    command_is(entry, "command", "sj hook windsurf ")
}

const FILE: HookFile = HookFile {
    path: &[".windsurf", "hooks.json"],
    event: "pre_write_code",
    entry: r#"{"command": "sj hook windsurf pre-edit", "show_output": true}"#,
    ours: is_ours,
    versioned: false,
};

/// Whether `.windsurf/hooks.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    FILE.is_installed(root)
}

/// Add the `pre_write_code` entry.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    FILE.plan(root)
}

/// Remove the entry; a file left empty is deleted, with `.windsurf/` if that leaves it empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    FILE.remove(root)
}
