//! `sj spec archive`: apply a change's deltas to `specs/` and move it to `changes/archive/`.
//!
//! The merge reimplements OpenSpec 1.14.0's `buildUpdatedSpec` (specs-apply.js, MIT) so both
//! tools write the same files: RENAMED, then REMOVED, MODIFIED and ADDED, then a rebuild with
//! one blank line between blocks and blank runs collapsed outside fences.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use super::blocks::{self, Block};
use super::validate::{validate, Problem};
use super::{find_change, read, spec_files, tasks};

pub struct Options {
    /// Archive despite validation errors and open tasks.
    pub force: bool,
    /// `YYYY-MM-DD`, the archive folder's prefix.
    pub date: String,
}

/// One spec file the archive writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecUpdate {
    pub file: PathBuf,
    pub created: bool,
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub renamed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Validation errors or open tasks; nothing was written.
    Refused(Vec<Problem>),
    Archived {
        specs: Vec<SpecUpdate>,
        dest: PathBuf,
    },
}

/// A merged spec and the operations applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    pub text: String,
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub renamed: usize,
}

const PLACEHOLDER: (&str, &str) = (
    "TBD - created by archiving change ",
    ". Update Purpose after archive.",
);

fn skeleton(capability: &str, change: &str, purpose: Option<&str>) -> String {
    let body = match purpose.map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) => p.to_string(),
        None => format!("{}{change}{}", PLACEHOLDER.0, PLACEHOLDER.1),
    };
    format!("# {capability} Specification\n\n## Purpose\n{body}\n\n## Requirements\n")
}

/// The target spec split around its `## Requirements` section.
struct Parts {
    before: String,
    header: String,
    preamble: String,
    blocks: Vec<Block>,
    after: String,
}

fn split_requirements(content: &str) -> Parts {
    static HEADER: OnceLock<Regex> = OnceLock::new();
    let header_re = HEADER.get_or_init(|| Regex::new(r"(?i)^##\s+Requirements\s*$").unwrap());
    let lines: Vec<&str> = content.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    let Some(h) = (0..lines.len()).find(|&i| !mask[i] && header_re.is_match(lines[i])) else {
        let before = content.trim_end();
        return Parts {
            before: if before.is_empty() {
                String::new()
            } else {
                format!("{before}\n\n")
            },
            header: "## Requirements".into(),
            preamble: String::new(),
            blocks: Vec::new(),
            after: "\n".into(),
        };
    };
    let end = (h + 1..lines.len())
        .find(|&i| !mask[i] && blocks::section_title(lines[i]).is_some())
        .unwrap_or(lines.len());
    let blocks = blocks::blocks_in(&lines, &mask, h + 1..end);
    let first = blocks.first().map_or(end, |b| b.header);
    Parts {
        before: lines[..h].join("\n"),
        header: lines[h].to_string(),
        preamble: lines[h + 1..first].join("\n").trim_end().to_string(),
        blocks,
        after: lines[end..].join("\n"),
    }
}

fn collapse_blank_runs(content: &str) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    let mut kept = Vec::with_capacity(lines.len());
    let mut blank = 0;
    for (i, line) in lines.iter().enumerate() {
        if mask[i] {
            blank = 0;
            kept.push(*line);
            continue;
        }
        if line.is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        kept.push(*line);
    }
    kept.join("\n")
}

fn fold(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Apply one delta file to its target spec (`None` for a new capability).
pub fn merge(target: Option<&str>, delta: &str, capability: &str, change: &str) -> Result<Merged> {
    let delta = blocks::normalize(delta);
    let plan = blocks::parse_delta(&delta);
    let cap = capability;
    if let Some((side, name, line)) = plan.unpaired.first() {
        let missing = if *side == "FROM" { "TO" } else { "FROM" };
        bail!(
            "{cap}: RENAMED entry on line {line} has no matching {missing}: \
             for header \"### Requirement: {name}\""
        );
    }
    let mut sets: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    let names = [
        (
            "ADDED",
            plan.added
                .iter()
                .map(|b| b.name.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "MODIFIED",
            plan.modified.iter().map(|b| b.name.clone()).collect(),
        ),
        (
            "REMOVED",
            plan.removed.iter().map(|r| r.name.clone()).collect(),
        ),
    ];
    for (kind, list) in &names {
        let set = sets.entry(kind).or_default();
        for n in list {
            if !set.insert(n.clone()) {
                bail!("{cap}: duplicate requirement in {kind} for header \"### Requirement: {n}\"");
            }
        }
    }
    let mut froms = BTreeSet::new();
    let mut tos = BTreeSet::new();
    for r in &plan.renamed {
        if !froms.insert(r.from.clone()) {
            bail!(
                "{cap}: duplicate FROM in RENAMED for header \"### Requirement: {}\"",
                r.from
            );
        }
        if !tos.insert(r.to.clone()) {
            bail!(
                "{cap}: duplicate TO in RENAMED for header \"### Requirement: {}\"",
                r.to
            );
        }
    }
    for (a, b) in [
        ("MODIFIED", "REMOVED"),
        ("MODIFIED", "ADDED"),
        ("ADDED", "REMOVED"),
    ] {
        if let Some(n) = sets[a].intersection(&sets[b]).next() {
            bail!(
                "{cap}: requirement present in multiple sections ({a} and {b}) for header \
                 \"### Requirement: {n}\""
            );
        }
    }
    for r in &plan.renamed {
        if sets["REMOVED"].iter().any(|n| fold(n) == fold(&r.from)) {
            bail!(
                "{cap}: requirement present in multiple sections (RENAMED and REMOVED) for \
                 header \"### Requirement: {}\"",
                r.from
            );
        }
        if sets["MODIFIED"].contains(&r.from) {
            bail!(
                "{cap}: when a rename exists, MODIFIED must reference the NEW header \
                 \"### Requirement: {}\"",
                r.to
            );
        }
        if sets["ADDED"].contains(&r.to) {
            bail!(
                "{cap}: RENAMED TO header collides with ADDED for \"### Requirement: {}\"",
                r.to
            );
        }
    }
    if !plan.has_operations() {
        bail!("{cap}: no ADDED, MODIFIED, REMOVED or RENAMED requirements");
    }

    let is_new = target.is_none();
    let content = match target {
        Some(t) => blocks::normalize(t),
        None => {
            if !plan.modified.is_empty() || !plan.renamed.is_empty() {
                bail!(
                    "{cap}: target spec does not exist; only ADDED requirements are allowed for \
                     a new capability"
                );
            }
            skeleton(cap, change, blocks::purpose(&delta).as_deref())
        }
    };
    let parts = split_requirements(&content);

    // Current blocks by name, and the original order of their keys.
    let mut current: BTreeMap<String, Block> = BTreeMap::new();
    for b in &parts.blocks {
        current.insert(b.name.clone(), b.clone());
    }
    let mut order: Vec<String> = parts.blocks.iter().map(|b| b.name.clone()).collect();
    let mut m = Merged {
        text: String::new(),
        added: 0,
        modified: 0,
        removed: 0,
        renamed: 0,
    };

    for r in &plan.renamed {
        if !current.contains_key(&r.from) {
            if current.contains_key(&r.to) {
                continue;
            }
            bail!(
                "{cap}: RENAMED failed for header \"### Requirement: {}\" - source not found",
                r.from
            );
        }
        if current.contains_key(&r.to) {
            bail!(
                "{cap}: RENAMED failed for header \"### Requirement: {}\" - target already exists",
                r.to
            );
        }
        let mut b = current.remove(&r.from).unwrap();
        let mut lines: Vec<&str> = b.raw.split('\n').collect();
        let header = format!("### Requirement: {}", r.to);
        lines[0] = &header;
        b.raw = lines.join("\n");
        b.name = r.to.clone();
        current.insert(r.to.clone(), b);
        if let Some(k) = order.iter_mut().find(|k| **k == r.from) {
            *k = r.to.clone();
        }
        m.renamed += 1;
    }
    for r in &plan.removed {
        if current.remove(&r.name).is_some() {
            m.removed += 1;
        } else if !is_new {
            if let Some(near) = current.keys().find(|k| fold(k) == fold(&r.name)) {
                bail!(
                    "{cap}: REMOVED failed for header \"### Requirement: {}\" - not found, but \
                     \"### Requirement: {near}\" exists",
                    r.name
                );
            }
        }
    }
    for b in &plan.modified {
        let Some(cur) = current.get(&b.name) else {
            bail!(
                "{cap}: MODIFIED failed for header \"### Requirement: {}\" - not found",
                b.name
            );
        };
        let incoming = blocks::scenario_names(&b.raw);
        let missing: Vec<String> = blocks::scenario_names(&cur.raw)
            .into_iter()
            .filter(|n| !incoming.contains(n))
            .map(|n| format!("\"{n}\""))
            .collect();
        if !missing.is_empty() {
            bail!(
                "{cap}: MODIFIED failed for header \"### Requirement: {}\" - current spec \
                 contains scenario(s) not present in the modified block: {}",
                b.name,
                missing.join(", ")
            );
        }
        if cur.raw.trim() != b.raw.trim() {
            m.modified += 1;
        }
        current.insert(b.name.clone(), b.clone());
    }
    let mut appended: Vec<Block> = Vec::new();
    for b in &plan.added {
        if let Some(existing) = current.get(&b.name) {
            if existing.raw.trim() == b.raw.trim() {
                continue;
            }
            bail!(
                "{cap}: ADDED failed for header \"### Requirement: {}\" - already exists",
                b.name
            );
        }
        if let Some(near) = current.keys().find(|k| fold(k) == fold(&b.name)) {
            bail!(
                "{cap}: ADDED failed for header \"### Requirement: {}\" - \"### Requirement: \
                 {near}\" already exists and differs only in case or spacing",
                b.name
            );
        }
        current.insert(b.name.clone(), b.clone());
        appended.push(b.clone());
        m.added += 1;
    }

    let mut kept: Vec<String> = Vec::new();
    if !parts.preamble.trim().is_empty() {
        kept.push(parts.preamble.trim_end().to_string());
    }
    for key in &order {
        if let Some(b) = current.get(key) {
            kept.push(b.raw.clone());
        }
    }
    kept.extend(appended.into_iter().map(|b| b.raw));
    let body = kept.join("\n\n").trim_end().to_string();
    let after = if parts.after.starts_with('\n') {
        parts.after.clone()
    } else {
        format!("\n{}", parts.after)
    };
    let pieces: Vec<&str> = [
        parts.before.trim_end(),
        parts.header.as_str(),
        body.as_str(),
        after.trim(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect();
    m.text = format!("{}\n", collapse_blank_runs(&pieces.join("\n\n")).trim_end());
    Ok(m)
}

fn dated(name: &str, date: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}-").unwrap());
    if re.is_match(name) {
        name.to_string()
    } else {
        format!("{date}-{name}")
    }
}

/// One change's part of a [`Plan`]: where it moves and the spec files it writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedChange {
    pub name: String,
    dir: PathBuf,
    pub dest: PathBuf,
    /// Each spec file this change touches, with the text after its merge.
    writes: Vec<(PathBuf, String, SpecUpdate)>,
}

/// Every write and move of an archive run, computed before anything is touched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub changes: Vec<PlannedChange>,
}

/// What [`plan`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Planned {
    /// Validation errors or open tasks in at least one change; nothing will be written.
    Refused(Vec<Problem>),
    Ready(Plan),
}

/// One change as [`apply`] archived it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archived {
    pub change: String,
    pub specs: Vec<SpecUpdate>,
    pub dest: PathBuf,
}

fn refusals(spec_root: &Path, change: &str, dir: &Path) -> Result<Vec<Problem>> {
    let mut reasons = validate(spec_root, Some(change))?.errors;
    let tasks_file = dir.join("tasks.md");
    for t in tasks(&tasks_file)? {
        if !t.done {
            reasons.push(Problem {
                message: format!("open task {}", t.text),
                file: tasks_file.clone(),
                line: t.line,
            });
        }
    }
    Ok(reasons)
}

/// The requirements a delta edits in place: MODIFIED, REMOVED and RENAMED (by its old name),
/// each with its 1-based line.
fn edited(delta: &str) -> Vec<(String, usize)> {
    let d = blocks::parse_delta(&blocks::normalize(delta));
    let mut out: Vec<(String, usize)> = d
        .modified
        .iter()
        .map(|b| (b.name.clone(), b.header + 1))
        .collect();
    out.extend(d.removed.iter().map(|r| (r.name.clone(), r.line)));
    out.extend(d.renamed.iter().map(|r| (r.from.clone(), r.line)));
    out
}

/// One problem for each requirement that two changes in one run both edit.
fn conflicts(found: &[(&str, super::Change)]) -> Result<Vec<Problem>> {
    // (capability, folded name) to the first change that edits it.
    let mut owner: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut out = Vec::new();
    for (_, c) in found {
        let delta_dir = c.dir.join("specs");
        for file in spec_files(&delta_dir)? {
            let capability = file
                .parent()
                .and_then(|p| p.strip_prefix(&delta_dir).ok())
                .context("delta spec outside the change's specs folder")?
                .to_string_lossy()
                .replace('\\', "/");
            for (name, line) in edited(&read(&file)?) {
                let key = (capability.clone(), fold(&name));
                match owner.get(&key) {
                    Some(first) if first != &c.name => out.push(Problem {
                        message: format!(
                            "conflict: {first} and {} both edit {capability}: {name}; \
                             archive one, then rebase the other's delta",
                            c.name
                        ),
                        file: file.clone(),
                        line,
                    }),
                    Some(_) => {}
                    None => {
                        owner.insert(key, c.name.clone());
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Check every change in `changes` and merge all their deltas in memory, in the order given:
/// a later change's merge builds on an earlier one's result. Nothing is written.
pub fn plan(spec_root: &Path, changes: &[&str], opts: &Options) -> Result<Planned> {
    let mut found = Vec::with_capacity(changes.len());
    for name in changes {
        let c = find_change(spec_root, name)?;
        if found
            .iter()
            .any(|(_, f): &(&str, super::Change)| f.name == c.name)
        {
            bail!("change `{}` is named more than once", c.name);
        }
        found.push((*name, c));
    }
    let mut reasons = Vec::new();
    if !opts.force {
        for (name, c) in &found {
            reasons.extend(refusals(spec_root, name, &c.dir)?);
        }
    }
    // Conflicts are refused even with --force: merging both would drop one change's edit.
    reasons.extend(conflicts(&found)?);
    if !reasons.is_empty() {
        return Ok(Planned::Refused(reasons));
    }

    // The merged text of every spec touched so far, so changes to one capability chain.
    let mut texts: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut planned = Vec::with_capacity(found.len());
    for (_, c) in found {
        let dest = spec_root
            .join("changes/archive")
            .join(dated(&c.name, &opts.date));
        if dest.exists() {
            bail!("archive destination already exists: {}", dest.display());
        }
        let delta_dir = c.dir.join("specs");
        let mut writes = Vec::new();
        for file in spec_files(&delta_dir)? {
            let rel = file
                .parent()
                .and_then(|p| p.strip_prefix(&delta_dir).ok())
                .context("delta spec outside the change's specs folder")?;
            let capability = rel.to_string_lossy().replace('\\', "/");
            let target = spec_root.join("specs").join(rel).join("spec.md");
            let current = match texts.get(&target) {
                Some(t) => Some(t.clone()),
                None if target.is_file() => Some(read(&target)?),
                None => None,
            };
            let delta = read(&file)?;
            let merged = merge(current.as_deref(), &delta, &capability, &c.name)?;
            let update = SpecUpdate {
                file: target.clone(),
                created: current.is_none(),
                added: merged.added,
                modified: merged.modified,
                removed: merged.removed,
                renamed: merged.renamed,
            };
            texts.insert(target.clone(), merged.text.clone());
            writes.push((target, merged.text, update));
        }
        planned.push(PlannedChange {
            name: c.name,
            dir: c.dir,
            dest,
            writes,
        });
    }
    Ok(Planned::Ready(Plan { changes: planned }))
}

/// Write every merged spec in `plan` and move each change to its archive folder.
pub fn apply(plan: Plan) -> Result<Vec<Archived>> {
    let mut out = Vec::with_capacity(plan.changes.len());
    for c in plan.changes {
        let mut specs = Vec::with_capacity(c.writes.len());
        for (target, text, update) in c.writes {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create {}", parent.display()))?;
            }
            std::fs::write(&target, text)
                .with_context(|| format!("cannot write {}", target.display()))?;
            specs.push(update);
        }
        if let Some(parent) = c.dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        std::fs::rename(&c.dir, &c.dest)
            .with_context(|| format!("cannot move {} to {}", c.dir.display(), c.dest.display()))?;
        out.push(Archived {
            change: c.name,
            specs,
            dest: c.dest,
        });
    }
    Ok(out)
}

/// Refuse, or merge every delta of `change` into `specs/` and move the change to the archive.
pub fn archive(spec_root: &Path, change: &str, opts: &Options) -> Result<Outcome> {
    match plan(spec_root, &[change], opts)? {
        Planned::Refused(reasons) => Ok(Outcome::Refused(reasons)),
        Planned::Ready(p) => {
            let a = apply(p)?.pop().context("archive plan held no change")?;
            Ok(Outcome::Archived {
                specs: a.specs,
                dest: a.dest,
            })
        }
    }
}
