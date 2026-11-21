//! Antigravity's `PreToolUse` hook protocol and its `.agents/hooks.json` entry.
//!
//! Antigravity sends the tool call as `toolCall.name` and `toolCall.args`, and reads one JSON
//! decision from stdout. Every answer is JSON, even when the edit is allowed.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{
    has_hooks, hooks_remove, hooks_text, insert_member, put, read_opt, rel, remove_if_empty,
    remove_member, root_object, style, Change,
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
//
// The file's top-level keys are hook names, each holding event keys:
// `{"specjutsu": {"PreToolUse": [{"matcher": ..., "hooks": [...]}]}}`.

/// The hook name specjutsu's entry goes under.
const NAME: &str = "specjutsu";
const EVENT: &str = "PreToolUse";
const ENTRY: &str = r#"{"matcher": "write_to_file|replace_file_content|multi_replace_file_content", "hooks": [{"type": "command", "command": "sj hook antigravity pre-edit"}]}"#;
const COMMAND: &str = "sj hook antigravity ";

fn path(root: &Path) -> PathBuf {
    root.join(".agents").join("hooks.json")
}

/// Whether `text`'s `specjutsu` hook runs `sj hook antigravity`.
fn has_ours(text: &str) -> bool {
    root_object(text).is_ok_and(|r| {
        r.get(NAME)
            .is_some_and(|m| text[m.value.start..m.value.end].contains(COMMAND))
    })
}

/// Whether `.agents/hooks.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| has_ours(&t))
}

/// Whether `.agents/hooks.json` exists, with or without the specjutsu hook.
pub fn has_hooks_file(root: &Path) -> bool {
    path(root).exists()
}

/// `text` without the `hooks.PreToolUse` entry earlier versions wrote, a shape Antigravity
/// doesn't read.
fn without_legacy(text: &str) -> Result<String> {
    if !has_hooks(text, legacy) {
        return Ok(text.to_string());
    }
    hooks_remove(text, legacy)
}

fn legacy(entry: &Node) -> bool {
    entry.get("hooks").is_some_and(|m| {
        m.value.elements().iter().any(|h| {
            h.get("command")
                .and_then(|c| c.value.as_str())
                .is_some_and(|c| c.starts_with(COMMAND))
        })
    })
}

/// Add the `specjutsu` hook, keeping every other byte. Nothing is written when the file
/// isn't valid JSON.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = match old.as_deref() {
        None => {
            format!("{{\n  \"{NAME}\": {{\n    \"{EVENT}\": [\n      {ENTRY}\n    ]\n  }}\n}}\n")
        }
        Some(t) if has_ours(t) => t.to_string(),
        Some(t) => {
            let t =
                without_legacy(t).with_context(|| format!("{} is not valid JSON", p.display()))?;
            let r =
                root_object(&t).with_context(|| format!("{} is not valid JSON", p.display()))?;
            let s = style(&t, &r);
            insert_member(&t, &r, NAME, |ind| hooks_text(EVENT, ENTRY, ind, &s), &s)
        }
    };
    Ok(crate::plan::put(p, root, old.as_deref(), new)
        .into_iter()
        .collect())
}

/// Carry out [`plan`].
pub fn install(root: &Path) -> Result<Vec<Change>> {
    crate::plan::apply(plan(root)?)
}

/// Remove the `specjutsu` hook (and any entry an earlier version wrote). A file left as an
/// empty object is deleted, with `.agents/` if that leaves it empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    let Some(old) = read_opt(&p)? else {
        return Ok(out);
    };
    let mut new =
        without_legacy(&old).with_context(|| format!("{} is not valid JSON", p.display()))?;
    if has_ours(&new) {
        let r = root_object(&new)?;
        new = remove_member(&new, &r, NAME);
    }
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
