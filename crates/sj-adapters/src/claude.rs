//! The Claude Code adapter: a CLAUDE.md block, the `sj-*` skills under `.claude/skills/`, and
//! hook entries in `.claude/settings.json`.

use std::path::{Path, PathBuf};

use crate::plan::{apply, Step};
use anyhow::{bail, Context, Result};

use crate::jsonedit::{self, indent_unit, line_indent, Node};

pub const BEGIN: &str = "<!-- specjutsu:begin -->";
pub const END: &str = "<!-- specjutsu:end -->";

/// The CLAUDE.md block body.
pub const BLOCK_BODY: &str = include_str!("../assets/claude/CLAUDE.block.md");

/// The skills, as (name, source text).
pub const SKILLS: [(&str, &str); 10] = [
    (
        "sj-explore",
        include_str!("../assets/skills/sj-explore/SKILL.md"),
    ),
    (
        "sj-propose",
        include_str!("../assets/skills/sj-propose/SKILL.md"),
    ),
    (
        "sj-clarify",
        include_str!("../assets/skills/sj-clarify/SKILL.md"),
    ),
    (
        "sj-principles",
        include_str!("../assets/skills/sj-principles/SKILL.md"),
    ),
    (
        "sj-apply",
        include_str!("../assets/skills/sj-apply/SKILL.md"),
    ),
    (
        "sj-debug",
        include_str!("../assets/skills/sj-debug/SKILL.md"),
    ),
    (
        "sj-review",
        include_str!("../assets/skills/sj-review/SKILL.md"),
    ),
    (
        "sj-verify",
        include_str!("../assets/skills/sj-verify/SKILL.md"),
    ),
    (
        "sj-finish",
        include_str!("../assets/skills/sj-finish/SKILL.md"),
    ),
    (
        "sj-onboard",
        include_str!("../assets/skills/sj-onboard/SKILL.md"),
    ),
];

const PRE_EDIT: &str = r#"{"matcher": "Edit|Write|MultiEdit|NotebookEdit", "hooks": [{"type": "command", "command": "sj hook claude pre-edit"}]}"#;
const SESSION_START: &str =
    r#"{"hooks": [{"type": "command", "command": "sj hook claude session-start"}]}"#;
const ENTRIES: [(&str, &str); 2] = [("PreToolUse", PRE_EDIT), ("SessionStart", SESSION_START)];

/// What happened to one file, relative to the repository root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Created(String),
    Updated(String),
    Removed(String),
    /// A hand-edited generated file that was left alone.
    Kept(String),
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::Created(p) => write!(f, "created {p}"),
            Change::Updated(p) => write!(f, "updated {p}"),
            Change::Removed(p) => write!(f, "removed {p}"),
            Change::Kept(p) => write!(f, "modified {p} (kept; sj update --force replaces it)"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// settings.json

fn is_ours(entry: &Node) -> bool {
    entry.get("hooks").is_some_and(|m| {
        m.value.elements().iter().any(|h| {
            h.get("command")
                .and_then(|c| c.value.as_str())
                .is_some_and(|c| c.starts_with("sj hook claude "))
        })
    })
}

fn splice(text: &str, at: usize, insert: &str) -> String {
    format!("{}{insert}{}", &text[..at], &text[at..])
}

fn cut(text: &str, from: usize, to: usize) -> String {
    format!("{}{}", &text[..from], &text[to..])
}

/// The whitespace between the separator (`,`, `[` or `{`) before `pos` and `pos`.
fn ws_before(text: &str, pos: usize) -> &str {
    let before = &text[..pos];
    let trimmed = before.trim_end_matches([' ', '\t', '\n', '\r']);
    &text[trimmed.len()..pos]
}

/// The indent of a line that starts after `ws` (the part after its last newline), or `""`.
fn ws_indent(ws: &str) -> &str {
    match ws.rfind('\n') {
        Some(i) => &ws[i + 1..],
        None => "",
    }
}

struct Style {
    multiline: bool,
    unit: String,
}

fn array_text(entry: &str, ind: &str, s: &Style) -> String {
    if s.multiline {
        format!("[\n{ind}{}{entry}\n{ind}]", s.unit)
    } else {
        format!("[{entry}]")
    }
}

fn hooks_text(event: &str, entry: &str, ind: &str, s: &Style) -> String {
    if s.multiline {
        let inner = format!("{ind}{}", s.unit);
        format!(
            "{{\n{inner}\"{event}\": {}\n{ind}}}",
            array_text(entry, &inner, s)
        )
    } else {
        format!("{{\"{event}\": {}}}", array_text(entry, "", s))
    }
}

/// Add member `key` to `obj`, after its last member; `value` gets the member's line indent.
fn insert_member(
    text: &str,
    obj: &Node,
    key: &str,
    value: impl Fn(&str) -> String,
    s: &Style,
) -> String {
    match obj.members().last() {
        Some(last) => {
            let ws = ws_before(text, last.key_start);
            let v = value(ws_indent(ws));
            splice(text, last.value.end, &format!(",{ws}\"{key}\": {v}"))
        }
        None if s.multiline => {
            let outer = line_indent(text, obj.start);
            let ind = format!("{outer}{}", s.unit);
            let v = value(&ind);
            splice(
                text,
                obj.start + 1,
                &format!("\n{ind}\"{key}\": {v}\n{outer}"),
            )
        }
        None => splice(text, obj.start + 1, &format!("\"{key}\": {}", value(""))),
    }
}

fn insert_element(text: &str, arr: &Node, entry: &str, s: &Style) -> String {
    match arr.elements().last() {
        Some(last) => {
            let ws = ws_before(text, last.start);
            splice(text, last.end, &format!(",{ws}{entry}"))
        }
        None if s.multiline => {
            let outer = line_indent(text, arr.start);
            splice(
                text,
                arr.start + 1,
                &format!("\n{outer}{}{entry}\n{outer}", s.unit),
            )
        }
        None => splice(text, arr.start + 1, entry),
    }
}

pub(crate) fn remove_member(text: &str, obj: &Node, key: &str) -> String {
    let m = obj.members();
    let Some(i) = m.iter().position(|x| x.key == key) else {
        return text.to_string();
    };
    if i > 0 {
        return cut(text, m[i - 1].value.end, m[i].value.end);
    }
    if let Some(next) = m.get(1) {
        return cut(text, m[0].key_start, next.key_start);
    }
    // The only member: also drop the line break and indent an insert added before `}`.
    let mut out = cut(text, obj.start + 1, m[0].value.end);
    let tail = format!("\n{}", line_indent(text, obj.start));
    let at = obj.start + 1;
    if out[at..].starts_with(&tail) && out[at + tail.len()..].trim_start().starts_with('}') {
        out = cut(&out, at, at + tail.len());
    }
    out
}

pub(crate) fn remove_element(text: &str, arr: &Node, i: usize) -> String {
    let e = arr.elements();
    if i > 0 {
        cut(text, e[i - 1].end, e[i].end)
    } else {
        cut(text, e[0].start, e[1].start)
    }
}

pub(crate) fn root_object(text: &str) -> Result<Node> {
    let root = jsonedit::parse(text)?;
    if !root.is_object() {
        bail!("the top level is not an object");
    }
    Ok(root)
}

/// The settings text with both specjutsu hooks added after any existing ones. `None` is a
/// missing file. Entries already present are left alone, so this is idempotent.
pub fn settings_install(text: Option<&str>) -> Result<String> {
    let Some(text) = text else {
        return Ok(format!(
            "{{\n  \"hooks\": {{\n    \"PreToolUse\": [\n      {PRE_EDIT}\n    ],\n    \"SessionStart\": [\n      {SESSION_START}\n    ]\n  }}\n}}\n"
        ));
    };
    hooks_install(text, &ENTRIES, is_ours)
}

/// `text` with each `(event, entry)` added under `hooks.<event>`, after any existing entries,
/// keeping every other byte. Events that already hold an entry `ours` recognises are left
/// alone, so this is idempotent.
pub(crate) fn hooks_install(
    text: &str,
    entries: &[(&str, &str)],
    ours: fn(&Node) -> bool,
) -> Result<String> {
    let mut t = text.to_string();
    for &(event, entry) in entries {
        let root = root_object(&t)?;
        let s = Style {
            multiline: t[root.start..root.end].contains('\n'),
            unit: indent_unit(&t).to_string(),
        };
        t = match root.get("hooks") {
            None => insert_member(
                &t,
                &root,
                "hooks",
                |ind| hooks_text(event, entry, ind, &s),
                &s,
            ),
            Some(h) if !h.value.is_object() => bail!("`hooks` is not an object"),
            Some(h) => match h.value.get(event) {
                None => insert_member(&t, &h.value, event, |ind| array_text(entry, ind, &s), &s),
                Some(ev) if !ev.value.is_array() => bail!("`hooks.{event}` is not an array"),
                Some(ev) if ev.value.elements().iter().any(ours) => t.clone(),
                Some(ev) => insert_element(&t, &ev.value, entry, &s),
            },
        };
    }
    jsonedit::parse(&t).context("internal error: the edited settings are not valid JSON")?;
    Ok(t)
}

/// The settings text with every specjutsu hook entry removed, and any event list or `hooks`
/// object that becomes empty removed with it.
pub fn settings_remove(text: &str) -> Result<String> {
    hooks_remove(text, is_ours)
}

/// `text` with every hook entry `ours` recognises removed, and any event list or `hooks`
/// object that becomes empty removed with it.
pub(crate) fn hooks_remove(text: &str, ours: fn(&Node) -> bool) -> Result<String> {
    let mut t = text.to_string();
    loop {
        let root = root_object(&t)?;
        let Some(h) = root.get("hooks").filter(|h| h.value.is_object()) else {
            break;
        };
        let found = h
            .value
            .members()
            .iter()
            .find_map(|ev| ev.value.elements().iter().position(ours).map(|i| (ev, i)));
        let Some((ev, i)) = found else {
            break;
        };
        t = if ev.value.elements().len() > 1 {
            remove_element(&t, &ev.value, i)
        } else if h.value.members().len() > 1 {
            remove_member(&t, &h.value, &ev.key)
        } else {
            remove_member(&t, &root, "hooks")
        };
    }
    jsonedit::parse(&t).context("internal error: the edited settings are not valid JSON")?;
    Ok(t)
}

fn has_our_hooks(text: &str) -> bool {
    has_hooks(text, is_ours)
}

/// Whether `text` has a hook entry `ours` recognises.
pub(crate) fn has_hooks(text: &str, ours: fn(&Node) -> bool) -> bool {
    let Ok(root) = jsonedit::parse(text) else {
        return false;
    };
    root.get("hooks").is_some_and(|h| {
        h.value
            .members()
            .iter()
            .any(|ev| ev.value.elements().iter().any(ours))
    })
}

// ---------------------------------------------------------------------------------------------
// CLAUDE.md block

pub fn block() -> String {
    block_with(BLOCK_BODY)
}

/// A marker block around `body`, as written into CLAUDE.md or AGENTS.md.
pub fn block_with(body: &str) -> String {
    format!("{BEGIN}\n{}{END}\n", lf(body))
}

/// Templates are compiled in from the checkout; a CRLF checkout must not change the output.
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
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

/// CLAUDE.md with the block appended, or replaced in place.
pub fn block_install(text: Option<&str>) -> String {
    block_install_with(text, &block())
}

/// `text` with `block` appended, or replacing the marker block already there.
pub fn block_install_with(text: Option<&str>, block: &str) -> String {
    match text {
        None | Some("") => block.to_string(),
        Some(t) => match block_range(t) {
            Some((b, e)) => format!("{}{}{}", &t[..b], block, &t[e..]),
            None => format!("{t}\n{block}"),
        },
    }
}

/// CLAUDE.md without the block (and the line break install put before it).
pub fn block_remove(text: &str) -> String {
    match block_range(text) {
        None => text.to_string(),
        Some((0, e)) => text[e..].to_string(),
        Some((b, e)) if text[..b].ends_with('\n') => format!("{}{}", &text[..b - 1], &text[e..]),
        Some((b, e)) => format!("{}{}", &text[..b], &text[e..]),
    }
}

// ---------------------------------------------------------------------------------------------
// Skills

const MARK: &str = "<!-- specjutsu:generated ";

/// FNV-1a, 64-bit.
pub fn fnv64(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A skill as installed: its source plus a last line with the source's hash.
pub fn generated(source: &str) -> String {
    format!("{source}{MARK}{:016x} -->\n", fnv64(source))
}

/// Whether an installed file is exactly what some version of specjutsu wrote.
pub fn is_unmodified(installed: &str) -> bool {
    let Some(at) = installed.rfind(MARK) else {
        return false;
    };
    let (body, mark) = installed.split_at(at);
    let hash = mark[MARK.len()..].trim_end().trim_end_matches("-->").trim();
    body.ends_with('\n') && hash == format!("{:016x}", fnv64(body))
}

// ---------------------------------------------------------------------------------------------
// Install, update, remove

pub(crate) fn rel(p: &Path, root: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn read_opt(p: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(p) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", p.display())),
    }
}

fn write(p: &Path, text: &str) -> Result<()> {
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }
    std::fs::write(p, text).with_context(|| format!("cannot write {}", p.display()))
}

/// Write `new` to `p` when it differs, recording the change.
pub(crate) fn put(
    p: &Path,
    root: &Path,
    old: Option<&str>,
    new: &str,
    out: &mut Vec<Change>,
) -> Result<()> {
    if old == Some(new) {
        return Ok(());
    }
    write(p, new)?;
    out.push(match old {
        None => Change::Created(rel(p, root)),
        Some(_) => Change::Updated(rel(p, root)),
    });
    Ok(())
}

fn paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        root.join("CLAUDE.md"),
        root.join(".claude").join("settings.json"),
        root.join(".claude").join("skills"),
    )
}

/// Whether `.claude/settings.json` has the specjutsu hooks.
pub fn has_hook(root: &Path) -> bool {
    let (_, settings, _) = paths(root);
    std::fs::read_to_string(settings).is_ok_and(|t| has_our_hooks(&t))
}

/// Whether the adapter is installed: the CLAUDE.md block or a specjutsu hook exists.
pub fn is_installed(root: &Path) -> bool {
    let (md, settings, _) = paths(root);
    std::fs::read_to_string(md).is_ok_and(|t| t.contains(BEGIN))
        || std::fs::read_to_string(settings).is_ok_and(|t| has_our_hooks(&t))
}

/// Every file `install` would write, keep or remove, computed without touching anything.
/// A generated `sj-*` skill this version no longer ships is removed; an edited one is left.
pub fn plan(root: &Path, force: bool) -> Result<Vec<Step>> {
    let (md, settings, skills) = paths(root);
    let old_settings = read_opt(&settings)?;
    let new_settings = settings_install(old_settings.as_deref())
        .with_context(|| format!("{} is not valid JSON", settings.display()))?;
    let old_md = read_opt(&md)?;
    let new_md = block_install(old_md.as_deref());
    let mut files = vec![(md, old_md, new_md), (settings, old_settings, new_settings)];
    for (name, source) in SKILLS {
        let p = skills.join(name).join("SKILL.md");
        let old = read_opt(&p)?;
        files.push((p, old, generated(&lf(source))));
    }
    let mut out = Vec::new();
    for (i, (path, old, new)) in files.into_iter().enumerate() {
        match old.as_deref() {
            Some(o) if o == new => {}
            Some(o) if i >= 2 && !is_unmodified(o) && !force => out.push(Step::keep(path, root)),
            _ => out.push(Step::write(path, root, old.is_some(), new)),
        }
    }
    out.extend(stale_skills(root, &skills)?);
    Ok(out)
}

/// A removal step for each unedited generated `sj-*` skill that is not in [`SKILLS`].
fn stale_skills(root: &Path, skills: &Path) -> Result<Vec<Step>> {
    let Ok(entries) = std::fs::read_dir(skills) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("sj-") && !SKILLS.iter().any(|(s, _)| s == n))
        .collect();
    names.sort();
    let mut out = Vec::new();
    for n in names {
        let p = skills.join(&n).join("SKILL.md");
        if read_opt(&p)?.is_some_and(|t| is_unmodified(&t)) {
            out.push(Step::delete(p, root));
        }
    }
    Ok(out)
}

/// What `install` would change, without writing anything.
pub fn plan_install(root: &Path, force: bool) -> Result<Vec<Change>> {
    Ok(plan(root, force)?.into_iter().map(|s| s.change).collect())
}

/// Install or refresh every file. A hand-edited skill is kept unless `force`. Nothing is
/// written when the settings file can't be read as JSON.
pub fn install(root: &Path, force: bool) -> Result<Vec<Change>> {
    apply(plan(root, force)?)
}

pub(crate) fn remove_if_empty(dir: &Path) {
    if std::fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none()) {
        let _ = std::fs::remove_dir(dir);
    }
}

/// Remove the block, the hooks and the unmodified skills. Files left empty are deleted.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let (md, settings, skills) = paths(root);
    let old_settings = read_opt(&settings)?;
    let new_settings = match &old_settings {
        Some(t) => Some(
            settings_remove(t)
                .with_context(|| format!("{} is not valid JSON", settings.display()))?,
        ),
        None => None,
    };
    let mut out = Vec::new();

    if let Some(old) = read_opt(&md)? {
        let new = block_remove(&old);
        if new.is_empty() {
            std::fs::remove_file(&md)?;
            out.push(Change::Removed(rel(&md, root)));
        } else if new != old {
            put(&md, root, Some(&old), &new, &mut out)?;
        }
    }
    if let (Some(old), Some(new)) = (&old_settings, &new_settings) {
        if new.trim() == "{}" && old.trim() != "{}" {
            std::fs::remove_file(&settings)?;
            out.push(Change::Removed(rel(&settings, root)));
        } else {
            put(&settings, root, Some(old), new, &mut out)?;
        }
    }
    for (name, _) in SKILLS {
        let dir = skills.join(name);
        let p = dir.join("SKILL.md");
        match read_opt(&p)? {
            Some(t) if is_unmodified(&t) => {
                std::fs::remove_file(&p)?;
                remove_if_empty(&dir);
                out.push(Change::Removed(rel(&p, root)));
            }
            Some(_) => out.push(Change::Kept(rel(&p, root))),
            None => {}
        }
    }
    remove_if_empty(&skills);
    remove_if_empty(&root.join(".claude"));
    Ok(out)
}
