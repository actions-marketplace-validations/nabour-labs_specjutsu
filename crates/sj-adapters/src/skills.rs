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

/// Most bytes in one reference file next to a skill's `SKILL.md` (SKILL-010).
pub const REF_LIMIT: usize = 8000;

/// Project files a skill names that are not references in its folder.
const PROJECT_FILES: [&str; 5] = [
    "proposal.md",
    "design.md",
    "tasks.md",
    "spec.md",
    "principles.md",
];

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

/// Each command in `cmds` is named, and first named after the ones before it.
fn in_order(name: &str, text: &str, cmds: &[&str], out: &mut Vec<Problem>) {
    let mut found: Vec<(&str, usize)> = Vec::new();
    for &c in cmds {
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
}

/// The setup skill, for agents that may not have `sj` yet (SKILL-008, SKILL-009). It lives
/// in `skills/` beside the `sj-*` set but is not embedded in the adapters.
pub const SETUP: &str = "specjutsu";

/// Steps the setup skill must name, in this order (SKILL-008).
const SETUP_STEPS: [&str; 6] = [
    "sj --version",
    "install.sh",
    "sj adopt",
    "sj init --agents",
    "sj doctor",
    "sj status",
];

fn check_setup_text(text: &str, out: &mut Vec<Problem>) {
    in_order(SETUP, text, &SETUP_STEPS, out);
    if !text.contains("cargo install specjutsu") {
        out.push(problem(SETUP, "does not name `cargo install specjutsu`"));
    }
    if text.contains("--force") {
        out.push(problem(SETUP, "passes `--force`"));
    }
    if !text.to_ascii_lowercase().contains("ask") {
        out.push(problem(SETUP, "does not ask before installing"));
    }
    if !text.contains("specjutsu.toml") {
        out.push(problem(SETUP, "does not keep an existing specjutsu.toml"));
    }
}

/// The onboarding walk-through: the full loop, cleanup without archiving, and a clean start.
fn check_onboard(text: &str, out: &mut Vec<Problem>) {
    let name = "sj-onboard";
    in_order(name, text, &ONBOARD_LOOP, out);
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

/// The bare lowercase `*.md` file names `text` mentions that are not project files: the
/// reference files a skill expects in its folder.
pub fn named_references(text: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = R
        .get_or_init(|| Regex::new(r"(?:^|[^A-Za-z0-9_./<>-])([a-z0-9][a-z0-9-]*\.md)\b").unwrap());
    let mut out: Vec<String> = Vec::new();
    for c in r.captures_iter(text) {
        let f = c[1].to_string();
        if !PROJECT_FILES.contains(&f.as_str()) && !out.contains(&f) {
            out.push(f);
        }
    }
    out
}

/// The reference files in a skill folder: every `*.md` other than `SKILL.md`, sorted.
pub fn reference_files(skill_dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if !skill_dir.is_dir() {
        return Ok(out);
    }
    for e in std::fs::read_dir(skill_dir)
        .with_context(|| format!("cannot read {}", skill_dir.display()))?
    {
        let e = e?;
        let n = e.file_name().to_string_lossy().into_owned();
        if n != "SKILL.md" && n.ends_with(".md") && e.file_type()?.is_file() {
            out.push(n);
        }
    }
    out.sort();
    Ok(out)
}

/// Reference rules (SKILL-010) and foreign commands in references (SKILL-002).
fn check_references(dir: &Path, name: &str, text: &str, out: &mut Vec<Problem>) -> Result<()> {
    let folder = dir.join(name);
    let present = reference_files(&folder)?;
    let named = named_references(text);
    for f in &named {
        if !present.contains(f) {
            out.push(problem(name, format!("names missing reference {f}")));
        }
    }
    for f in &present {
        if !named.contains(f) {
            out.push(problem(
                name,
                format!("reference {f} is not named in SKILL.md"),
            ));
        }
        let path = folder.join(f);
        let t = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        if t.len() > REF_LIMIT {
            out.push(problem(
                name,
                format!("reference {f} is {} bytes (limit {REF_LIMIT})", t.len()),
            ));
        }
        for c in foreign(&t) {
            out.push(problem(name, format!("{f} names a foreign command ({c})")));
        }
    }
    Ok(())
}

/// Lower case with every run of whitespace collapsed to one space, so a phrase matches
/// across line breaks.
fn norm(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The sj-apply review loop and its prompts (SKILL-011..013).
fn check_apply(dir: &Path, text: &str, out: &mut Vec<Problem>) -> Result<()> {
    const NAME: &str = "sj-apply";
    let t = norm(text);
    for f in [
        "implementer-prompt.md",
        "task-reviewer-prompt.md",
        "re-review-prompt.md",
    ] {
        if !t.contains(f) {
            out.push(problem(NAME, format!("does not name {f}")));
        }
    }
    if !t.contains("fresh reviewer subagent") {
        out.push(problem(
            NAME,
            "does not send each task to a fresh reviewer subagent",
        ));
    }
    if !t.contains("3 rounds") {
        out.push(problem(NAME, "does not limit the fix loop to 3 rounds"));
    }
    let read = |f: &str| -> Result<Option<String>> {
        let p = dir.join(NAME).join(f);
        if !p.is_file() {
            return Ok(None);
        }
        let raw =
            std::fs::read_to_string(&p).with_context(|| format!("cannot read {}", p.display()))?;
        Ok(Some(norm(&raw)))
    };
    if let Some(r) = read("task-reviewer-prompt.md")? {
        const F: &str = "task-reviewer-prompt.md";
        if !r.contains("sj verify") {
            out.push(problem(NAME, format!("{F} does not name sj verify")));
        }
        if !r.contains("don't trust the implementer's report") {
            out.push(problem(
                NAME,
                format!("{F} does not say to ignore the implementer's report"),
            ));
        }
        for p in [
            "git diff",
            "approved",
            "changes_requested",
            "fail without the change",
        ] {
            if !r.contains(p) {
                out.push(problem(NAME, format!("{F} does not name {p}")));
            }
        }
    }
    if let Some(i) = read("implementer-prompt.md")? {
        const F: &str = "implementer-prompt.md";
        if i.replace("never run `sj task done`", "")
            .contains("sj task done")
        {
            out.push(problem(NAME, format!("{F} runs sj task done")));
        }
        let statuses = i.replace("done_with_concerns", "");
        let wants: [(&str, bool); 5] = [
            ("sj test", i.contains("sj test")),
            ("DONE", statuses.contains("done")),
            ("DONE_WITH_CONCERNS", i.contains("done_with_concerns")),
            ("BLOCKED", i.contains("blocked")),
            ("NEEDS_CONTEXT", i.contains("needs_context")),
        ];
        for (label, ok) in wants {
            if !ok {
                out.push(problem(NAME, format!("{F} does not name {label}")));
            }
        }
    }
    Ok(())
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
    check_references(dir, name, &text, out)?;
    if let Some((_, cmds)) = REQUIRED.iter().find(|(s, _)| *s == name) {
        for c in *cmds {
            if !text.contains(c) {
                out.push(problem(name, format!("does not name `{c}`")));
            }
        }
    }
    if name == "sj-apply" {
        check_apply(dir, &text, out)?;
    }
    if name == "sj-onboard" {
        check_onboard(&text, out);
    }
    if name == SETUP {
        check_setup_text(&text, out);
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

/// Every problem in the setup skill `skills_dir/specjutsu/SKILL.md` (SKILL-008, SKILL-009).
pub fn check_setup(skills_dir: &Path) -> Result<Vec<Problem>> {
    let mut out = Vec::new();
    check_skill(skills_dir, SETUP, &mut out)?;
    Ok(out)
}
