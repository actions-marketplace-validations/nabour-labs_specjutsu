//! Antigravity's `PreToolUse` hook protocol and its `.agents/hooks.json` entry.
//!
//! Antigravity sends the tool call as `toolCall.name` and `toolCall.args`, and reads one JSON
//! decision from stdout. Every answer is JSON, even when the edit is allowed.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{
    has_hooks, hooks_install, hooks_remove, put, read_opt, rel, remove_if_empty, root_object,
    Change,
};
use crate::jsonedit::Node;

/// Keys that may hold the edited file's path in `toolCall.args`, tried in order. The docs
/// name `TargetFile` for `write_to_file`; the others are fallbacks (checked by `more-adapters-3` task 2.1).
const PATH_KEYS: [&str; 3] = ["TargetFile", "file_path", "path"];

/// The file the tool call edits.
pub fn target(input: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    let args = &v["toolCall"]["args"];
    for key in PATH_KEYS {
        if let Some(p) = args[key].as_str() {
            return Ok(p.to_string());
        }
    }
    bail!(
        "no file path in toolCall.args (looked for {})",
        PATH_KEYS.join(", ")
    )
}

/// The answer that lets the edit go ahead.
pub fn allow() -> String {
    "{\"decision\":\"allow\"}\n".to_string()
}

/// The answer that blocks the edit, with the reasons shown to the agent.
pub fn deny(reasons: &[String]) -> String {
    let v = serde_json::json!({"decision": "deny", "reason": reasons.join("\n")});
    format!("{v}\n")
}

// ---------------------------------------------------------------------------------------------
// .agents/hooks.json

const ENTRY: &str = r#"{"matcher": "write_to_file|replace_file_content|multi_replace_file_content", "hooks": [{"type": "command", "command": "sj hook antigravity pre-edit"}]}"#;
const ENTRIES: [(&str, &str); 1] = [("PreToolUse", ENTRY)];

fn is_ours(entry: &Node) -> bool {
    entry.get("hooks").is_some_and(|m| {
        m.value.elements().iter().any(|h| {
            h.get("command")
                .and_then(|c| c.value.as_str())
                .is_some_and(|c| c.starts_with("sj hook antigravity "))
        })
    })
}

fn path(root: &Path) -> PathBuf {
    root.join(".agents").join("hooks.json")
}

/// Whether `.agents/hooks.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| has_hooks(&t, is_ours))
}

/// Whether `.agents/hooks.json` exists, with or without the specjutsu hook.
pub fn has_hooks_file(root: &Path) -> bool {
    path(root).exists()
}

/// Add the `PreToolUse` entry, keeping every other byte. Nothing is written when the file
/// isn't valid JSON.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = match old.as_deref() {
        None => {
            format!("{{\n  \"hooks\": {{\n    \"PreToolUse\": [\n      {ENTRY}\n    ]\n  }}\n}}\n")
        }
        Some(t) => hooks_install(t, &ENTRIES, is_ours)
            .with_context(|| format!("{} is not valid JSON", p.display()))?,
    };
    Ok(crate::plan::put(p, root, old.as_deref(), new)
        .into_iter()
        .collect())
}

/// Carry out [`plan`].
pub fn install(root: &Path) -> Result<Vec<Change>> {
    crate::plan::apply(plan(root)?)
}

/// Remove the entry. A file left as an empty object is deleted, with `.agents/` if that
/// leaves it empty.
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
    if root_object(&new)?.members().is_empty() {
        std::fs::remove_file(&p)?;
        remove_if_empty(&root.join(".agents"));
        out.push(Change::Removed(rel(&p, root)));
    } else {
        put(&p, root, Some(&old), &new, &mut out)?;
    }
    Ok(out)
}
