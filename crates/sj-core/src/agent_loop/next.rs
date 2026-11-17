//! `sj next`: the context for the next task, and which open tasks can run in parallel.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use super::default_base;
use crate::authoring::{self, blocks, Change};
use crate::git::Git;
use crate::{spec, testscan, trace};

/// Most bytes `sj next` prints besides the scenario texts (LOOP-002).
pub const BUDGET: usize = 1600;

/// Room kept for the `  … <n> more` line when the file list is cut.
const MORE_LINE: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScenarioText {
    pub id: String,
    /// The heading and its steps; `None` when no spec defines the ID.
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Next {
    pub change: String,
    pub task: String,
    pub scenarios: Vec<ScenarioText>,
    /// Changed since the merge-base, repository-relative; `None` without git or a base.
    pub files: Option<Vec<String>>,
    pub files_omitted: usize,
    /// `<spec_root>/principles.md`, repository-relative, when it exists.
    pub principles: Option<String>,
    pub commands: Vec<String>,
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The full text of scenario `id` in `text`: its heading up to the next heading, trailing blank
/// lines dropped, ending in one newline.
pub fn scenario_text(text: &str, id: &str) -> Option<String> {
    static HEAD: OnceLock<Regex> = OnceLock::new();
    let head = HEAD.get_or_init(|| {
        Regex::new(r"^####\s+Scenario:\s*([A-Z][A-Z0-9]*-[0-9]+)(?:\s|$)").unwrap()
    });
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    let start = (0..lines.len())
        .find(|&i| !mask[i] && head.captures(lines[i]).is_some_and(|c| &c[1] == id))?;
    let mut end = start + 1;
    while end < lines.len() && (mask[end] || !lines[end].starts_with('#')) {
        end += 1;
    }
    while end > start + 1 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    let mut out = lines[start..end].join("\n");
    out.push('\n');
    Some(out)
}

fn find_scenario(change: &Change, spec_root: &Path, id: &str) -> Result<Option<String>> {
    let mut files = authoring::spec_files(&change.dir.join("specs"))?;
    files.extend(authoring::spec_files(&spec_root.join("specs"))?);
    for f in files {
        if let Some(t) = scenario_text(&authoring::read(&f)?, id) {
            return Ok(Some(t));
        }
    }
    Ok(None)
}

/// Files changed since the merge-base of `base` and `HEAD`; `None` when that cannot be worked out.
fn changed_files(root: &Path, base: Option<&str>) -> Option<Vec<String>> {
    let base = default_base(root, base)?;
    let git = Git::new(root);
    let mb = git.merge_base(&base, "HEAD").ok()?;
    git.changed_files(&mb).ok()
}

/// The change to work on: the one named, or the first open change with an unchecked task.
pub fn pick(spec_root: &Path, change: Option<&str>) -> Result<Option<(Change, authoring::Task)>> {
    let candidates = match change {
        Some(name) => vec![authoring::find_change(spec_root, name)?],
        None => authoring::open_changes(spec_root)?,
    };
    for c in candidates {
        let tasks = authoring::tasks(&c.dir.join("tasks.md"))?;
        if let Some(t) = tasks.into_iter().find(|t| !t.done) {
            return Ok(Some((c, t)));
        }
    }
    Ok(None)
}

/// The next task's context, or `None` when no task is open.
pub fn next(
    root: &Path,
    spec_root: &Path,
    change: Option<&str>,
    base: Option<&str>,
) -> Result<Option<Next>> {
    let Some((c, task)) = pick(spec_root, change)? else {
        return Ok(None);
    };
    let mut ids: Vec<String> = Vec::new();
    for id in blocks::task_ids(&task.text) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    let mut scenarios = Vec::new();
    for id in &ids {
        scenarios.push(ScenarioText {
            id: id.clone(),
            text: find_scenario(&c, spec_root, id)?,
        });
    }
    let principles = spec_root.join("principles.md");
    let mut n = Next {
        change: c.name.clone(),
        task: task.text.clone(),
        scenarios,
        files: None,
        files_omitted: 0,
        principles: principles.is_file().then(|| rel(&principles, root)),
        commands: ids.iter().map(|id| format!("sj test {id}")).collect(),
    };
    if let Some(all) = changed_files(root, base) {
        n.files = Some(Vec::new());
        let fixed = render(&n).len() - scenario_bytes(&n);
        let mut room = BUDGET.saturating_sub(fixed);
        let mut kept = Vec::new();
        for (i, f) in all.iter().enumerate() {
            let line = f.len() + 3;
            let rest_needs = if i + 1 < all.len() { MORE_LINE } else { 0 };
            if line + rest_needs > room {
                break;
            }
            room -= line;
            kept.push(f.clone());
        }
        n.files_omitted = all.len() - kept.len();
        n.files = Some(kept);
    }
    Ok(Some(n))
}

fn scenario_bytes(n: &Next) -> usize {
    n.scenarios
        .iter()
        .filter_map(|s| s.text.as_ref())
        .map(String::len)
        .sum()
}

/// The compact text of [`next`].
pub fn render(n: &Next) -> String {
    let mut out = format!("change {}\ntask {}\n\n", n.change, n.task);
    for s in &n.scenarios {
        match &s.text {
            Some(t) => {
                out.push_str(t);
                out.push('\n');
            }
            None => out.push_str(&format!("missing {}\n\n", s.id)),
        }
    }
    if let Some(files) = &n.files {
        out.push_str("files\n");
        for f in files {
            out.push_str(&format!("  {f}\n"));
        }
        if n.files_omitted > 0 {
            out.push_str(&format!("  … {} more\n", n.files_omitted));
        }
    }
    if let Some(p) = &n.principles {
        out.push_str(&format!("principles {p}\n"));
    }
    if !n.commands.is_empty() {
        out.push_str("run\n");
        for c in &n.commands {
            out.push_str(&format!("  {c}\n"));
        }
    }
    out
}

/// The change's unchecked `[P]` tasks in batches that share no test file, in task order.
pub fn parallel(root: &Path, spec_root: &Path, change: &str) -> Result<Vec<Vec<String>>> {
    let c = authoring::find_change(spec_root, change)?;
    let tasks: Vec<authoring::Task> = authoring::tasks(&c.dir.join("tasks.md"))?
        .into_iter()
        .filter(|t| !t.done && t.text.contains("[P]"))
        .collect();
    if tasks.is_empty() {
        return Ok(Vec::new());
    }
    let config = crate::config::load(root)?;
    let scenarios = spec::load_scenarios(spec_root)?;
    let scan = testscan::scan_project(root, &config)?;
    let report = trace::build_report(&scenarios, &scan.tests);
    let files_of = |text: &str| -> BTreeSet<PathBuf> {
        let ids = blocks::task_ids(text);
        report
            .mapped
            .iter()
            .filter(|m| ids.contains(&m.id))
            .flat_map(|m| m.tests.iter().map(|t| t.file.clone()))
            .collect()
    };
    let mut batches: Vec<(Vec<String>, BTreeSet<PathBuf>)> = Vec::new();
    for t in tasks {
        let files = files_of(&t.text);
        match batches
            .iter_mut()
            .find(|(_, used)| used.is_disjoint(&files))
        {
            Some((texts, used)) => {
                texts.push(t.text);
                used.extend(files);
            }
            None => batches.push((vec![t.text], files)),
        }
    }
    Ok(batches.into_iter().map(|(t, _)| t).collect())
}

/// The change `sj next --parallel` looks at: the one named, else the first open change with an
/// unchecked task.
pub fn parallel_change(spec_root: &Path, change: Option<&str>) -> Result<Option<String>> {
    if let Some(name) = change {
        return Ok(Some(authoring::find_change(spec_root, name)?.name));
    }
    Ok(pick(spec_root, None)?.map(|(c, _)| c.name))
}
