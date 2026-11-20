//! Cursor's `preToolUse` hook protocol: the input it sends and the answers it accepts.
//!
//! Cursor treats hook output that isn't valid JSON as a deny, so every answer here is one
//! JSON object, even when the edit is allowed.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{
    has_hooks, hooks_install, hooks_remove, put, read_opt, rel, remove_if_empty, root_object,
    Change,
};
use crate::jsonedit::Node;

/// Keys that may hold the edited file's path in `tool_input`, tried in order. Cursor's docs
/// don't name the key for `Write` and `Delete`.
const PATH_KEYS: [&str; 3] = ["file_path", "path", "target_file"];

/// The file the tool call edits.
pub fn target(input: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    let tool_input = &v["tool_input"];
    for key in PATH_KEYS {
        if let Some(p) = tool_input[key].as_str() {
            return Ok(p.to_string());
        }
    }
    bail!(
        "no file path in tool_input (looked for {})",
        PATH_KEYS.join(", ")
    )
}

/// The answer that lets the edit go ahead.
pub fn allow() -> String {
    "{\"permission\":\"allow\"}\n".to_string()
}

/// The answer that blocks the edit, with the reasons shown to the user and the agent.
pub fn deny(reasons: &[String]) -> String {
    let msg = reasons.join("\n");
    let v = serde_json::json!({
        "permission": "deny",
        "user_message": msg,
        "agent_message": msg,
    });
    format!("{v}\n")
}

// ---------------------------------------------------------------------------------------------
// .cursor/hooks.json

const ENTRY: &str = r#"{"command": "sj hook cursor pre-edit", "matcher": "Write|Delete"}"#;
const ENTRIES: [(&str, &str); 1] = [("preToolUse", ENTRY)];

fn is_ours(entry: &Node) -> bool {
    entry
        .get("command")
        .and_then(|c| c.value.as_str())
        .is_some_and(|c| c.starts_with("sj hook cursor "))
}

fn path(root: &Path) -> PathBuf {
    root.join(".cursor").join("hooks.json")
}

/// Whether `.cursor/hooks.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| has_hooks(&t, is_ours))
}

/// Add the `preToolUse` entry, keeping every other byte. Nothing is written when the file
/// isn't valid JSON.
pub fn install(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = match old.as_deref() {
        None => format!(
            "{{\n  \"version\": 1,\n  \"hooks\": {{\n    \"preToolUse\": [\n      {ENTRY}\n    ]\n  }}\n}}\n"
        ),
        Some(t) => hooks_install(t, &ENTRIES, is_ours)
            .with_context(|| format!("{} is not valid JSON", p.display()))?,
    };
    let mut out = Vec::new();
    put(&p, root, old.as_deref(), &new, &mut out)?;
    Ok(out)
}

/// Remove the entry. A file left with nothing but `version` is deleted, with `.cursor/` if
/// that leaves it empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    let Some(old) = read_opt(&p)? else {
        return Ok(out);
    };
    let new = hooks_remove(&old, is_ours)
        .with_context(|| format!("{} is not valid JSON", p.display()))?;
    if new == old {
        return Ok(out);
    }
    let only_version = root_object(&new)?
        .members()
        .iter()
        .all(|m| m.key == "version");
    if only_version {
        std::fs::remove_file(&p)?;
        remove_if_empty(&root.join(".cursor"));
        out.push(Change::Removed(rel(&p, root)));
    } else {
        put(&p, root, Some(&old), &new, &mut out)?;
    }
    Ok(out)
}
