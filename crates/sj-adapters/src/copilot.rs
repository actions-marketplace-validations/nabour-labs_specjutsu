//! GitHub Copilot's `PreToolUse` hook (CLI and cloud agent) in `.github/hooks/specjutsu.json`.
//!
//! The PascalCase event gives Claude-shaped input. Copilot denies the tool call on any
//! non-zero exit, so the hook always exits 0: it prints a deny to block and nothing to allow.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::claude::Change;
use crate::hook_file::{command_is, HookFile};
use crate::jsonedit::Node;

/// Keys that may hold the edited file's path in `tool_input`, tried in order.
const PATH_KEYS: [&str; 2] = ["file_path", "path"];

/// Every file the tool call edits: one path, or each file of an `apply_patch` patch.
pub fn targets(input: &str) -> Result<Vec<String>> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    let tool_input = &v["tool_input"];
    for key in PATH_KEYS {
        if let Some(p) = tool_input[key].as_str() {
            return Ok(vec![p.to_string()]);
        }
    }
    if tool_input["command"].is_string() {
        return crate::codex::targets(input);
    }
    bail!(
        "no file path in tool_input (looked for {}, or an apply_patch command)",
        PATH_KEYS.join(", ")
    )
}

/// The answer that blocks the tool call, with the reasons Copilot shows the agent.
pub fn deny(reasons: &[String]) -> String {
    let v = serde_json::json!({
        "permissionDecision": "deny",
        "permissionDecisionReason": reasons.join("\n"),
    });
    format!("{v}\n")
}

/// What `sj init --agents copilot` prints after installing.
pub const NOTICE: &str =
    "copilot: hooks take effect for the cloud agent once .github/hooks/ is on the default branch";

fn is_ours(entry: &Node) -> bool {
    command_is(entry, "bash", "sj hook copilot ")
}

const FILE: HookFile = HookFile {
    path: &[".github", "hooks", "specjutsu.json"],
    event: "PreToolUse",
    entry: r#"{"type": "command", "bash": "sj hook copilot pre-edit", "matcher": "^(Write|Edit|create|edit)$"}"#,
    ours: is_ours,
    versioned: true,
};

/// Whether `.github/hooks/specjutsu.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    FILE.is_installed(root)
}

/// Whether the repository has Copilot files.
pub fn is_present(root: &Path) -> bool {
    root.join(".github/copilot-instructions.md").exists() || root.join(".github/hooks").exists()
}

/// Add the `PreToolUse` entry.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    FILE.plan(root)
}

/// Remove the entry; the file goes when nothing else is in it, with `.github/hooks/` and
/// `.github/` if that leaves them empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    FILE.remove(root)
}
