//! Codex's `PreToolUse` hook protocol for `apply_patch` edits.
//!
//! Codex sends the patch text in `tool_input.command`; one patch can add, change, delete or
//! move several files. An allowed edit prints nothing and exits 0.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::claude::{put, read_opt, rel, remove_if_empty, Change};

/// Patch lines that name a file the patch writes.
const FILE_LINES: [&str; 4] = [
    "*** Add File: ",
    "*** Update File: ",
    "*** Delete File: ",
    "*** Move to: ",
];

/// Every file the patch touches, in patch order, without duplicates.
pub fn targets(input: &str) -> Result<Vec<String>> {
    let v: serde_json::Value = serde_json::from_str(input).context("the hook input is not JSON")?;
    let Some(patch) = v["tool_input"]["command"].as_str() else {
        bail!("no patch in tool_input.command");
    };
    let mut out: Vec<String> = Vec::new();
    for line in patch.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(path) = FILE_LINES.iter().find_map(|p| line.strip_prefix(p)) {
            let path = path.trim().to_string();
            if !path.is_empty() && !out.contains(&path) {
                out.push(path);
            }
        }
    }
    if out.is_empty() {
        bail!("no file path in the patch");
    }
    Ok(out)
}

/// The answer that blocks the tool call, with the reasons Codex shows the agent.
pub fn deny(reasons: &[String]) -> String {
    let v = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reasons.join("\n"),
        }
    });
    format!("{v}\n")
}

// ---------------------------------------------------------------------------------------------
// .codex/config.toml

const BEGIN: &str = "# specjutsu:begin";
const END: &str = "# specjutsu:end";

/// The hook group, between marker comments so removal can cut exactly what install added.
const BLOCK: &str = "# specjutsu:begin\n\
[[hooks.PreToolUse]]\n\
matcher = \"^(apply_patch|Edit|Write)$\"\n\
\n\
[[hooks.PreToolUse.hooks]]\n\
type = \"command\"\n\
command = \"sj hook codex pre-edit\"\n\
# specjutsu:end\n";

fn path(root: &Path) -> PathBuf {
    root.join(".codex").join("config.toml")
}

/// The byte range of the block, its trailing newline included.
fn block_range(text: &str) -> Option<(usize, usize)> {
    let b = text.find(BEGIN)?;
    let e = text[b..].find(END)? + b + END.len();
    let e = if text[e..].starts_with('\n') {
        e + 1
    } else {
        e
    };
    Some((b, e))
}

/// Whether `.codex/config.toml` has the specjutsu hook block.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| block_range(&t).is_some())
}

/// Append the hook block, or refresh it in place. Nothing is written when the file isn't
/// valid TOML or can't take the block (for example, `hooks.PreToolUse` is an inline array).
pub fn plan(root: &Path) -> Result<Vec<crate::plan::Step>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = match old.as_deref() {
        None | Some("") => BLOCK.to_string(),
        Some(t) => {
            toml::from_str::<toml::Value>(t)
                .with_context(|| format!("{} is not valid TOML", p.display()))?;
            match block_range(t) {
                Some((b, e)) => format!("{}{BLOCK}{}", &t[..b], &t[e..]),
                None => format!("{t}\n{BLOCK}"),
            }
        }
    };
    toml::from_str::<toml::Value>(&new).with_context(|| {
        format!(
            "cannot add the specjutsu hook to {}; add `sj hook codex pre-edit` as a PreToolUse hook by hand",
            p.display()
        )
    })?;
    Ok(crate::plan::put(p, root, old.as_deref(), new)
        .into_iter()
        .collect())
}

/// Carry out [`plan`].
pub fn install(root: &Path) -> Result<Vec<Change>> {
    crate::plan::apply(plan(root)?)
}

/// Remove the block and the line break install put before it. A file left empty is deleted,
/// with `.codex/` if that leaves it empty.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    let Some(old) = read_opt(&p)? else {
        return Ok(out);
    };
    let new = match block_range(&old) {
        None => return Ok(out),
        Some((0, e)) => old[e..].to_string(),
        Some((b, e)) if old[..b].ends_with('\n') => format!("{}{}", &old[..b - 1], &old[e..]),
        Some((b, e)) => format!("{}{}", &old[..b], &old[e..]),
    };
    if new.is_empty() {
        std::fs::remove_file(&p)?;
        remove_if_empty(&root.join(".codex"));
        out.push(Change::Removed(rel(&p, root)));
    } else {
        put(&p, root, Some(&old), &new, &mut out)?;
    }
    Ok(out)
}
