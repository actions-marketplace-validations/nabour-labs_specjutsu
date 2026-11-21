//! The Gemini CLI adapter: a `BeforeTool` hook in `.gemini/settings.json`, and `AGENTS.md`
//! added to `context.fileName` so Gemini reads the specjutsu block.
//!
//! Gemini sends Claude-shaped input (`tool_name`, `tool_input.file_path`) and reads the same
//! JSON decision as Antigravity, so [`crate::antigravity::allow`] and
//! [`crate::antigravity::deny`] answer it.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{
    has_hooks, hooks_install, hooks_remove, insert_element, insert_member, put, read_opt, rel,
    remove_if_empty, remove_member, root_object, style, Change,
};
use crate::jsonedit::{self, Node};

/// The file the tool call edits.
pub fn target(input: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    match v["tool_input"]["file_path"].as_str() {
        Some(p) => Ok(p.to_string()),
        None => bail!("no file path in tool_input.file_path"),
    }
}

// ---------------------------------------------------------------------------------------------
// .gemini/settings.json

const ENTRY: &str = r#"{"matcher": "write_file|replace", "hooks": [{"type": "command", "command": "sj hook gemini pre-edit"}]}"#;
const ENTRIES: [(&str, &str); 1] = [("BeforeTool", ENTRY)];

/// Gemini's own instructions file, which it stops reading once `context.fileName` is set.
const DEFAULT_NAME: &str = "GEMINI.md";
const NAME: &str = "AGENTS.md";
/// The list install writes when the setting is missing, which removal takes out again.
const NEW_LIST: &str = r#"["GEMINI.md", "AGENTS.md"]"#;

fn is_ours(entry: &Node) -> bool {
    entry.get("hooks").is_some_and(|m| {
        m.value.elements().iter().any(|h| {
            h.get("command")
                .and_then(|c| c.value.as_str())
                .is_some_and(|c| c.starts_with("sj hook gemini "))
        })
    })
}

fn path(root: &Path) -> PathBuf {
    root.join(".gemini").join("settings.json")
}

/// Whether `.gemini/settings.json` has the specjutsu hook.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| has_hooks(&t, is_ours))
}

/// `text` with `AGENTS.md` in `context.fileName`, keeping the names already there. A missing
/// setting becomes `["GEMINI.md", "AGENTS.md"]`, and a single name becomes a list.
fn context_install(text: &str) -> Result<String> {
    let root = root_object(text)?;
    let s = style(text, &root);
    let t = match root.get("context") {
        None => insert_member(
            text,
            &root,
            "context",
            |ind| {
                if s.multiline {
                    format!("{{\n{ind}{}\"fileName\": {NEW_LIST}\n{ind}}}", s.unit)
                } else {
                    format!("{{\"fileName\": {NEW_LIST}}}")
                }
            },
            &s,
        ),
        Some(c) if !c.value.is_object() => bail!("`context` is not an object"),
        Some(c) => match c.value.get("fileName") {
            None => insert_member(text, &c.value, "fileName", |_| NEW_LIST.to_string(), &s),
            Some(f) => match f.value.as_str() {
                Some(NAME) => text.to_string(),
                Some(name) => {
                    let list = format!("[{}, \"{NAME}\"]", serde_json::to_string(name)?);
                    format!("{}{list}{}", &text[..f.value.start], &text[f.value.end..])
                }
                None if !f.value.is_array() => {
                    bail!("`context.fileName` is not a string or a list")
                }
                None if f.value.elements().iter().any(|e| e.as_str() == Some(NAME)) => {
                    text.to_string()
                }
                None => insert_element(text, &f.value, &format!("\"{NAME}\""), &s),
            },
        },
    };
    jsonedit::parse(&t).context("internal error: the edited settings are not valid JSON")?;
    Ok(t)
}

/// `text` without the `context.fileName` list install wrote. A list the user had, or one
/// they changed since, is left alone: naming `AGENTS.md` there does no harm.
fn context_remove(text: &str) -> Result<String> {
    let root = root_object(text)?;
    let Some(c) = root.get("context").filter(|c| c.value.is_object()) else {
        return Ok(text.to_string());
    };
    let ours = c.value.get("fileName").is_some_and(|f| {
        let names: Vec<_> = f.value.elements().iter().map(Node::as_str).collect();
        names == [Some(DEFAULT_NAME), Some(NAME)]
    });
    if !ours {
        return Ok(text.to_string());
    }
    let t = if c.value.members().len() == 1 {
        remove_member(text, &root, "context")
    } else {
        remove_member(text, &c.value, "fileName")
    };
    jsonedit::parse(&t).context("internal error: the edited settings are not valid JSON")?;
    Ok(t)
}

/// Add the hook and the `context.fileName` entry, keeping every other byte. Nothing is
/// written when the file isn't valid JSON.
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = match old.as_deref() {
        None => format!(
            "{{\n  \"context\": {{\n    \"fileName\": {NEW_LIST}\n  }},\n  \"hooks\": {{\n    \"BeforeTool\": [\n      {ENTRY}\n    ]\n  }}\n}}\n"
        ),
        Some(t) => hooks_install(t, &ENTRIES, is_ours)
            .and_then(|t| context_install(&t))
            .with_context(|| format!("cannot edit {}", p.display()))?,
    };
    Ok(crate::plan::put(p, root, old.as_deref(), new)
        .into_iter()
        .collect())
}

/// Carry out [`plan`].
pub fn install(root: &Path) -> Result<Vec<Change>> {
    crate::plan::apply(plan(root)?)
}

/// Remove the hook and the `context.fileName` list install wrote. A file left as an empty
/// object is deleted, with `.gemini/` if that leaves it empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    let Some(old) = read_opt(&p)? else {
        return Ok(out);
    };
    if !has_hooks(&old, is_ours) {
        return Ok(out);
    }
    let new = hooks_remove(&old, is_ours)
        .and_then(|t| context_remove(&t))
        .with_context(|| format!("cannot edit {}", p.display()))?;
    if root_object(&new)?.members().is_empty() {
        std::fs::remove_file(&p)?;
        remove_if_empty(&root.join(".gemini"));
        out.push(Change::Removed(rel(&p, root)));
    } else {
        put(&p, root, Some(&old), &new, &mut out)?;
    }
    Ok(out)
}
