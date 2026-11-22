//! What the agent hooks decide: the session-start bootstrap and the pre-edit check.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::agent_loop::{next, status};
use crate::authoring::blocks;
use crate::config::{self, Mode};
use crate::gate::Classifier;
use crate::ledger;

/// Most bytes the session-start hook prints.
pub const BOOTSTRAP_LIMIT: usize = 1200;

/// Longest line kept when the bootstrap must shrink.
const LINE_LIMIT: usize = 200;

fn cut_line(line: &str, max: usize) -> String {
    if line.len() <= max {
        return line.to_string();
    }
    let mut end = max - '…'.len_utf8();
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &line[..end])
}

fn join(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

/// The session-start text: every open change's status and the active task, within
/// [`BOOTSTRAP_LIMIT`] bytes.
pub fn session_start(root: &Path) -> Result<String> {
    let config = config::load(root)?;
    let spec_root = root.join(&config.spec_root);
    let rows = status::status(root, &spec_root, None)?;
    if rows.is_empty() {
        return Ok("specjutsu: no open changes\n".to_string());
    }
    // (line, droppable): the per-state ID lines go first when space runs out.
    let mut lines: Vec<(String, bool)> = Vec::new();
    for r in &rows {
        for (i, l) in status::render(r).into_iter().enumerate() {
            let droppable = i > 0 && !l.starts_with("  next ") && l != "  all tasks done";
            lines.push((l, droppable));
        }
    }
    if let Some((c, t)) = next::pick(&spec_root, None)? {
        lines.push((format!("active {}: next {}", c.name, t.text), false));
        lines.push((
            "run `sj next` for its scenarios, files and test commands".to_string(),
            true,
        ));
    }
    let all: Vec<String> = lines.iter().map(|(l, _)| l.clone()).collect();
    if join(&all).len() <= BOOTSTRAP_LIMIT {
        return Ok(join(&all));
    }
    let kept: Vec<String> = lines
        .iter()
        .filter(|(_, d)| !d)
        .map(|(l, _)| cut_line(l, LINE_LIMIT))
        .collect();
    let mut text = join(&kept);
    if text.len() > BOOTSTRAP_LIMIT {
        let mut end = BOOTSTRAP_LIMIT - "…\n".len();
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text = format!("{}…\n", &text[..end]);
    }
    Ok(text)
}

/// What the pre-edit hook does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// Advisory mode: allowed, with these reasons.
    Warn(Vec<String>),
    /// Guided and strict modes: blocked, with these reasons.
    Block(Vec<String>),
}

/// `path` made absolute and resolved through symlinks, as far as it exists.
fn resolve(path: &Path) -> PathBuf {
    let mut existing = path.to_path_buf();
    let mut rest: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        match (existing.file_name(), existing.parent()) {
            (Some(name), Some(parent)) => {
                rest.push(name.to_os_string());
                existing = parent.to_path_buf();
            }
            _ => return path.to_path_buf(),
        }
    }
    let mut out = existing.canonicalize().unwrap_or(existing);
    for name in rest.into_iter().rev() {
        out.push(name);
    }
    out
}

/// The nearest folder at or above `path`'s folder with a `.git` entry: a folder for a main
/// checkout, a file for a linked worktree.
fn work_tree_of(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Decide on a Claude Code `PreToolUse` payload (the JSON on the hook's stdin).
pub fn pre_edit(root: &Path, payload: &str) -> Result<Decision> {
    let v: serde_json::Value =
        serde_json::from_str(payload).context("the hook input is not JSON")?;
    let input = &v["tool_input"];
    let Some(target) = input["file_path"]
        .as_str()
        .or_else(|| input["notebook_path"].as_str())
    else {
        return Ok(Decision::Allow);
    };
    decide(root, target)
}

/// The pre-edit decision for one target path, absolute or relative to `root`, with no
/// agent-specific input or output. Every agent's hook protocol ends here.
///
/// The decision uses the git work tree that holds the target, so an edit in a linked
/// worktree is judged by that worktree's config, active task and ledger, wherever it sits
/// relative to `root`. A target in no work tree, or in the work tree that holds `root`, is
/// judged by `root`.
pub fn decide(root: &Path, target: &str) -> Result<Decision> {
    let given = resolve(&std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf()));
    let target = Path::new(target);
    let target = resolve(&if target.is_absolute() {
        target.to_path_buf()
    } else {
        given.join(target)
    });
    // A work tree at or above the given root is the given project itself (it may sit in a
    // subfolder of its repository); any other one is a separate checkout, such as a worktree.
    let root_abs = match work_tree_of(&target) {
        Some(tree) if !given.starts_with(&tree) => tree,
        _ => given,
    };
    let root = root_abs.as_path();
    let config = config::load(root)?;
    let Ok(rel) = target.strip_prefix(&root_abs).map(Path::to_path_buf) else {
        return Ok(Decision::Allow);
    };
    let rel = rel.to_string_lossy().replace('\\', "/");
    let classifier = if root.join(config::FILE).is_file() {
        Classifier::new(&config, &config.spec_root)?
    } else {
        Classifier::builtin(&config.spec_root)
    };
    if !classifier.is_impl(&rel) {
        return Ok(Decision::Allow);
    }

    let spec_root = root.join(&config.spec_root);
    let Some((_, task)) = next::pick(&spec_root, None)? else {
        return Ok(Decision::Allow);
    };
    let mut ids: Vec<String> = Vec::new();
    for id in blocks::task_ids(&task.text) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        return Ok(Decision::Allow);
    }
    let red: BTreeSet<String> = ledger::load(root)?
        .records
        .into_iter()
        .filter(|r| r.result.is_red())
        .flat_map(|r| r.ids)
        .collect();
    // A `[keep]` scenario passes on base, so it never has a red run (ADAPT-043).
    let keep = crate::spec::keep_ids(&spec_root)?;
    let reasons: Vec<String> = ids
        .iter()
        .filter(|id| !red.contains(*id) && !keep.contains(*id))
        .map(|id| format!("write a failing test for {id} first (sj test {id})"))
        .collect();
    Ok(match (reasons.is_empty(), config.mode) {
        (true, _) => Decision::Allow,
        (false, Mode::Advisory) => Decision::Warn(reasons),
        (false, _) => Decision::Block(reasons),
    })
}
