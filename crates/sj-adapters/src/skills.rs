//! The static check for `skills/`: format, size, foreign commands, required commands, and
//! coverage of the workflows the skills replace.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;

/// The skills specjutsu ships, in working order.
pub const SKILLS: [&str; 10] = [
    "sj-explore",
    "sj-propose",
    "sj-clarify",
    "sj-principles",
    "sj-apply",
    "sj-debug",
    "sj-review",
    "sj-verify",
    "sj-finish",
    "sj-onboard",
];

/// Most bytes in a skill's body, after the frontmatter.
pub const BODY_LIMIT: usize = 6000;

/// Superpowers skills the set replaces (`writing-skills` and `using-superpowers` are left out).
pub const SUPERPOWERS: [&str; 12] = [
    "brainstorming",
    "writing-plans",
    "executing-plans",
    "test-driven-development",
    "subagent-driven-development",
    "dispatching-parallel-agents",
    "systematic-debugging",
    "requesting-code-review",
    "receiving-code-review",
    "verification-before-completion",
    "finishing-a-development-branch",
    "using-git-worktrees",
];

/// OpenSpec workflow commands the set replaces.
pub const OPENSPEC: [&str; 7] = [
    "explore", "propose", "apply", "archive", "sync", "update", "onboard",
];

/// Spec Kit commands the set replaces.
pub const SPEC_KIT: [&str; 8] = [
    "constitution",
    "specify",
    "clarify",
    "plan",
    "tasks",
    "analyze",
    "checklist",
    "implement",
];

/// Commands a skill must name for its workflow (SKILL-003).
const REQUIRED: [(&str, &[&str]); 6] = [
    ("sj-apply", &["sj next", "sj test", "sj task done"]),
    ("sj-debug", &["sj test"]),
    ("sj-verify", &["sj verify"]),
    ("sj-finish", &["sj verify", "sj spec archive"]),
    (
        "sj-propose",
        &["sj spec new", "sj spec validate", "sj spec check"],
    ),
    ("sj-clarify", &["sj spec check"]),
];

/// The scratch change `sj-onboard` creates and removes (SKILL-005..007).
const DEMO: &str = "sj-onboard-demo";

/// Commands `sj-onboard` must name, in this order (SKILL-005).
const ONBOARD_LOOP: [&str; 4] = ["sj spec new", "sj test", "sj task done", "sj verify"];

/// The onboarding walk-through: the full loop, cleanup without archiving, and a clean start.
fn check_onboard(text: &str, out: &mut Vec<Problem>) {
    let name = "sj-onboard";
    let mut found: Vec<(&str, usize)> = Vec::new();
    for c in ONBOARD_LOOP {
        let Some(at) = text.find(c) else {
            out.push(problem(name, format!("does not name `{c}`")));
            continue;
        };
        // The first earlier step this one is named ahead of.
        if let Some((prev, _)) = found.iter().find(|(_, p)| at < *p) {
            out.push(problem(name, format!("names `{c}` before `{prev}`")));
        }
        found.push((c, at));
    }
    if !text.contains(DEMO) {
        out.push(problem(
            name,
            format!("does not name the scratch change {DEMO}"),
        ));
    }
    let lower = text.to_ascii_lowercase();
    let folder = format!("changes/{DEMO}/");
    if !(lower.contains("delete")
        && text.contains(&folder)
        && lower.contains("demo test")
        && lower.contains("demo code"))
    {
        out.push(problem(
            name,
            format!("does not delete {folder}, the demo test and the demo code"),
        ));
    }
    if text.contains("sj spec archive") {
        out.push(problem(name, "runs `sj spec archive`"));
    }
    if !(text.contains("git status")
        && lower.contains("uncommitted")
        && lower.contains("stop")
        && text.contains(&folder))
    {
        out.push(problem(
            name,
            format!("does not stop on uncommitted changes or an existing {DEMO}"),
        ));
    }
}

/// Skills that must apply the principles file (SKILL-004).
const PRINCIPLED: [&str; 2] = ["sj-propose", "sj-review"];

/// One broken rule. `skill` is `coverage` for problems in `COVERAGE.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub skill: String,
    pub rule: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.skill, self.rule)
    }
}

fn problem(skill: &str, rule: impl Into<String>) -> Problem {
    Problem {
        skill: skill.to_string(),
        rule: rule.into(),
    }
}

/// Frontmatter fields and the body after them (leading blank lines dropped), or `None` without
/// frontmatter.
fn split(text: &str) -> Option<(BTreeMap<String, String>, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let mut fields = BTreeMap::new();
    let mut offset = 0;
    let mut key: Option<String> = None;
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        let l = line.trim_end_matches(['\n', '\r']);
        if l == "---" {
            return Some((fields, rest[offset..].trim_start_matches(['\n', '\r'])));
        }
        if let Some(k) = &key {
            if l.starts_with([' ', '\t']) {
                let v: &mut String = fields.get_mut(k).unwrap();
                if !v.is_empty() {
                    v.push(' ');
                }
                v.push_str(l.trim());
                continue;
            }
        }
        if let Some((k, v)) = l.split_once(':') {
            let v = v.trim();
            let v = if matches!(v, ">" | "|" | ">-" | "|-") {
                ""
            } else {
                v
            };
            fields.insert(k.trim().to_string(), v.trim_matches('"').to_string());
            key = Some(k.trim().to_string());
        } else {
            key = None;
        }
    }
    None
}

fn foreign(text: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = R.get_or_init(|| {
        Regex::new(
            r"openspec\s+[a-z][a-z-]*|specify\s+(?:init|check)\b|/speckit|/opsx|superpowers:",
        )
        .unwrap()
    });
    let mut out: Vec<String> = Vec::new();
    for m in r.find_iter(text) {
        let s = m.as_str().split_whitespace().collect::<Vec<_>>().join(" ");
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

fn check_skill(dir: &Path, name: &str, out: &mut Vec<Problem>) -> Result<()> {
    let file = dir.join(name).join("SKILL.md");
    if !file.is_file() {
        out.push(problem(name, "missing SKILL.md"));
        return Ok(());
    }
    let text = std::fs::read_to_string(&file)
        .with_context(|| format!("cannot read {}", file.display()))?;
    match split(&text) {
        None => out.push(problem(name, "no frontmatter")),
        Some((fields, body)) => {
            match fields.get("name").map(String::as_str) {
                Some(n) if n == name => {}
                Some(n) => out.push(problem(name, format!("name is {n}, expected {name}"))),
                None => out.push(problem(name, format!("name is missing, expected {name}"))),
            }
            let says_when = fields
                .get("description")
                .is_some_and(|d| d.to_ascii_lowercase().contains("use when"));
            if !says_when {
                out.push(problem(name, "description does not say when to use it"));
            }
            if body.len() > BODY_LIMIT {
                out.push(problem(
                    name,
                    format!("body is {} bytes (limit {BODY_LIMIT})", body.len()),
                ));
            }
        }
    }
    for f in foreign(&text) {
        out.push(problem(name, format!("names a foreign command ({f})")));
    }
    if let Some((_, cmds)) = REQUIRED.iter().find(|(s, _)| *s == name) {
        for c in *cmds {
            if !text.contains(c) {
                out.push(problem(name, format!("does not name `{c}`")));
            }
        }
    }
    if name == "sj-onboard" {
        check_onboard(&text, out);
    }
    if PRINCIPLED.contains(&name) {
        if !text.contains("principles.md") {
            out.push(problem(name, "does not name principles.md"));
        }
        if !(text.contains("design.md") && text.to_ascii_lowercase().contains("deviation")) {
            out.push(problem(name, "does not record deviations in design.md"));
        }
    }
    Ok(())
}

fn check_coverage(dir: &Path, out: &mut Vec<Problem>) -> Result<()> {
    let file = dir.join("COVERAGE.md");
    if !file.is_file() {
        out.push(problem("coverage", "missing COVERAGE.md"));
        return Ok(());
    }
    let text = std::fs::read_to_string(&file)
        .with_context(|| format!("cannot read {}", file.display()))?;
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for line in text.lines() {
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().trim_matches('`'))
            .collect();
        if !line.trim_start().starts_with('|') || cells.len() < 3 {
            continue;
        }
        let (tool, item, skill) = (cells[0], cells[1], cells[2]);
        if tool == "Tool" || tool.starts_with("---") {
            continue;
        }
        *counts
            .entry((tool.to_string(), item.to_string()))
            .or_default() += 1;
        if !SKILLS.contains(&skill) || !dir.join(skill).join("SKILL.md").is_file() {
            out.push(problem(
                "coverage",
                format!("{tool} {item} maps to unknown skill {skill}"),
            ));
        }
    }
    let lists: [(&str, &[&str]); 3] = [
        ("Superpowers", &SUPERPOWERS),
        ("OpenSpec", &OPENSPEC),
        ("Spec Kit", &SPEC_KIT),
    ];
    for (tool, items) in lists {
        for item in items {
            match counts.get(&(tool.to_string(), item.to_string())) {
                None => out.push(problem("coverage", format!("{tool} {item} is not mapped"))),
                Some(1) => {}
                Some(_) => out.push(problem(
                    "coverage",
                    format!("{tool} {item} is mapped twice"),
                )),
            }
        }
    }
    Ok(())
}

/// Every problem in `skills_dir`, skills in [`SKILLS`] order, then coverage.
pub fn check(skills_dir: &Path) -> Result<Vec<Problem>> {
    let mut out = Vec::new();
    for name in SKILLS {
        check_skill(skills_dir, name, &mut out)?;
    }
    check_coverage(skills_dir, &mut out)?;
    Ok(out)
}
