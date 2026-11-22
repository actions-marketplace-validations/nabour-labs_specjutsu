//! Listing and assigning scenario IDs (`sj ids`).
//!
//! Prefixes come from the capability folder or `[ids.prefixes]`; numbers continue after the
//! highest one ever used under the spec root, archive included; a delta copy of a `specs/`
//! scenario shares its ID. Writing inserts the ID before each name and changes no other byte.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::spec;

/// One ID `sj ids --write` would insert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    title_offset: usize,
}

/// Whether `s` is a valid ID prefix: an uppercase letter, then uppercase letters or digits.
pub fn is_prefix(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// The prefix derived from a capability folder name: its ASCII letters, uppercased.
pub fn derive_prefix(capability: &str) -> String {
    capability
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// The longest derived prefix a suggestion keeps whole; longer ones are cut to [`SHORT`].
const WHOLE: usize = 6;
/// The length a long derived prefix is cut to before clashes lengthen it.
const SHORT: usize = 4;

/// Suggested prefixes for `open` capabilities, beside the `kept` ones (capability to prefix).
/// A capability's letters, uppercased, are kept whole up to six letters and cut to four
/// beyond that; suggestions equal to each other or to a kept prefix lengthen together, one
/// letter at a time, until distinct or out of letters. Returns `kept` plus the suggestions,
/// or an error naming two capabilities and their prefix when a clash is left.
pub fn suggest_prefixes(
    kept: &BTreeMap<String, String>,
    open: &[String],
) -> Result<BTreeMap<String, String>> {
    let letters: Vec<(&String, String)> = open
        .iter()
        .filter(|c| !kept.contains_key(*c))
        .map(|c| (c, derive_prefix(c)))
        .filter(|(_, l)| !l.is_empty())
        .collect();
    let mut len: Vec<usize> = letters
        .iter()
        .map(|(_, l)| if l.len() <= WHOLE { l.len() } else { SHORT })
        .collect();
    loop {
        let current: Vec<&str> = letters
            .iter()
            .zip(&len)
            .map(|((_, l), n)| &l[..*n])
            .collect();
        let clashing: Vec<usize> = (0..letters.len())
            .filter(|&i| {
                len[i] < letters[i].1.len()
                    && (kept.values().any(|p| p == current[i])
                        || current
                            .iter()
                            .enumerate()
                            .any(|(j, s)| j != i && *s == current[i]))
            })
            .collect();
        if clashing.is_empty() {
            break;
        }
        for i in clashing {
            len[i] += 1;
        }
    }
    let mut out = kept.clone();
    for ((cap, l), n) in letters.iter().zip(&len) {
        let prefix = &l[..*n];
        let other = out
            .iter()
            .find(|(c, p)| *p == prefix && *c != *cap)
            .map(|(c, _)| c.clone());
        if let Some(other) = other {
            let mut pair = [other.as_str(), cap.as_str()];
            pair.sort();
            bail!(
                "capabilities `{}` and `{}` both use prefix {prefix}; \
                 map one in [ids.prefixes] in specjutsu.toml",
                pair[0],
                pair[1]
            );
        }
        out.insert((*cap).clone(), prefix.to_string());
    }
    Ok(out)
}

/// The `[ids.prefixes]` table `sj ids prefixes` prints: one line per capability folder. A
/// configured prefix is kept, then the most common prefix among a capability's IDs (the
/// alphabetically first on a tie), and the rest get [`suggest_prefixes`].
pub fn prefix_table(
    spec_root: &Path,
    configured: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    table_with_use(spec_root, configured, &prefix_use(spec_root)?)
}

/// [`prefix_table`] given the capabilities' prefix use from [`prefix_use`].
fn table_with_use(
    spec_root: &Path,
    configured: &BTreeMap<String, String>,
    used: &BTreeMap<String, BTreeMap<String, usize>>,
) -> Result<BTreeMap<String, String>> {
    let mut folders = Vec::new();
    for file in spec::discover_spec_files(spec_root)? {
        if let Some(cap) = place(spec_root, &file).capability {
            if !folders.contains(&cap) {
                folders.push(cap);
            }
        }
    }
    let mut kept = configured.clone();
    for cap in &folders {
        if kept.contains_key(cap) {
            continue;
        }
        if let Some(counts) = used.get(cap) {
            let top = counts.values().max().copied().unwrap_or(0);
            if let Some((prefix, _)) = counts.iter().find(|(_, n)| **n == top) {
                kept.insert(cap.clone(), prefix.clone());
            }
        }
    }
    let mut table = suggest_prefixes(&kept, &folders)?;
    table.retain(|cap, _| folders.contains(cap));
    Ok(table)
}

/// Prefix use per capability, archive included: how many IDs carry each prefix.
fn prefix_use(spec_root: &Path) -> Result<BTreeMap<String, BTreeMap<String, usize>>> {
    let mut used: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for file in all_spec_files(spec_root)? {
        let Some(cap) = place(spec_root, &file).capability else {
            continue;
        };
        for s in spec::parse_scenarios(&file, &read(&file)?) {
            if let Some((prefix, _)) = s.id.as_deref().and_then(|id| id.rsplit_once('-')) {
                *used
                    .entry(cap.clone())
                    .or_default()
                    .entry(prefix.to_string())
                    .or_insert(0) += 1;
            }
        }
    }
    Ok(used)
}

/// The [`prefix_table`] entries for capabilities that have no IDs and no configured prefix:
/// the mappings to pin before `plan` assigns their first IDs.
pub fn unmapped_prefixes(
    spec_root: &Path,
    configured: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let used = prefix_use(spec_root)?;
    let mut table = table_with_use(spec_root, configured, &used)?;
    table.retain(|cap, _| !configured.contains_key(cap) && !used.contains_key(cap));
    Ok(table)
}

/// Each capability that `plan` mints IDs for with a derived prefix longer than six letters:
/// the capability, that prefix, and the one `sj ids prefixes --write` would map it to, in
/// capability order. Mapped capabilities are skipped, and so are those whose existing IDs
/// already use the long prefix.
pub fn long_prefixes(
    spec_root: &Path,
    configured: &BTreeMap<String, String>,
    plan: &[Assignment],
) -> Result<Vec<(String, String, String)>> {
    let mut caps: Vec<String> = Vec::new();
    for a in plan {
        if let Some(cap) = place(spec_root, &a.file).capability {
            if !caps.contains(&cap) {
                caps.push(cap);
            }
        }
    }
    caps.sort();
    let table = prefix_table(spec_root, configured)?;
    Ok(caps
        .into_iter()
        .filter(|cap| !configured.contains_key(cap))
        .filter_map(|cap| {
            let long = derive_prefix(&cap);
            let minted = plan.iter().any(|a| a.id.starts_with(&format!("{long}-")));
            let short = table.get(&cap)?.clone();
            (long.len() > WHOLE && minted && short != long).then_some((cap, long, short))
        })
        .collect())
}

/// `toml` (the text of `specjutsu.toml`) with a `key = "VALUE"` line for each of `add`
/// spliced into `[ids.prefixes]`, after the table's last key line; the table is appended at
/// the end when absent. Every other byte is kept, and new lines use the file's line ending.
pub fn add_prefix_lines(toml: &str, add: &BTreeMap<String, String>) -> String {
    if add.is_empty() {
        return toml.to_string();
    }
    let eol = if toml.contains("\r\n") { "\r\n" } else { "\n" };
    let mut new_lines = String::new();
    for (cap, prefix) in add {
        new_lines.push_str(&format!("{} = \"{prefix}\"{eol}", toml_key(cap)));
    }
    let lines: Vec<&str> = toml.split_inclusive('\n').collect();
    // `[ids.prefixes]`, allowing spaces inside the brackets and a trailing comment.
    let is_header = |l: &str| {
        let Some((inner, rest)) = l.trim().strip_prefix('[').and_then(|t| t.split_once(']')) else {
            return false;
        };
        let rest = rest.trim();
        inner.split('.').map(str::trim).eq(["ids", "prefixes"])
            && (rest.is_empty() || rest.starts_with('#'))
    };
    let Some(header) = lines.iter().position(|l| is_header(l)) else {
        let mut out = toml.to_string();
        if !out.is_empty() {
            if !out.ends_with('\n') {
                out.push_str(eol);
            }
            out.push_str(eol);
        }
        out.push_str("[ids.prefixes]");
        out.push_str(eol);
        out.push_str(&new_lines);
        return out;
    };
    // The last key line before the next table header, or the header itself.
    let mut last = header;
    for (i, l) in lines.iter().enumerate().skip(header + 1) {
        let t = l.trim();
        if t.starts_with('[') {
            break;
        }
        if !t.is_empty() && !t.starts_with('#') {
            last = i;
        }
    }
    let mut out = String::with_capacity(toml.len() + new_lines.len() + eol.len());
    for (i, l) in lines.iter().enumerate() {
        out.push_str(l);
        if i == last {
            if !l.ends_with('\n') {
                out.push_str(eol);
            }
            out.push_str(&new_lines);
        }
    }
    out
}

/// `key` as a TOML key: bare when it may be, quoted otherwise.
fn toml_key(key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if bare {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Where a spec file sits relative to the spec root.
struct Place {
    capability: Option<String>,
    /// Under `specs/`, not a change delta.
    in_specs: bool,
}

fn place(spec_root: &Path, file: &Path) -> Place {
    let rel = file.strip_prefix(spec_root).unwrap_or(file);
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let at = |i: usize| -> Option<String> {
        // The capability is a folder: something must follow it.
        (parts.len() > i + 1).then(|| parts[i].clone())
    };
    match parts.first().map(String::as_str) {
        Some("specs") => Place {
            capability: at(1),
            in_specs: true,
        },
        Some("changes") => {
            let specs_at = if parts.get(1).map(String::as_str) == Some("archive") {
                3
            } else {
                2
            };
            let capability = if parts.get(specs_at).map(String::as_str) == Some("specs") {
                at(specs_at + 1)
            } else {
                None
            };
            Place {
                capability,
                in_specs: false,
            }
        }
        _ => Place {
            capability: None,
            in_specs: false,
        },
    }
}

/// Every `spec.md` under `specs/` and `changes/` (archive included), for used numbers.
fn all_spec_files(spec_root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .with_context(|| format!("cannot read {}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.file_name().is_some_and(|n| n == "spec.md") {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(&spec_root.join("specs"), &mut out)?;
    walk(&spec_root.join("changes"), &mut out)?;
    Ok(out)
}

fn read(file: &Path) -> Result<String> {
    std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))
}

/// The highest number used per ID prefix in any `spec.md` under the spec root, archive included.
pub fn highest_numbers(spec_root: &Path) -> Result<BTreeMap<String, u64>> {
    let mut out: BTreeMap<String, u64> = BTreeMap::new();
    for file in all_spec_files(spec_root)? {
        for s in spec::parse_scenarios(&file, &read(&file)?) {
            if let Some((prefix, n)) = s.id.as_deref().and_then(|id| id.rsplit_once('-')) {
                if let Ok(n) = n.parse::<u64>() {
                    let e = out.entry(prefix.to_string()).or_insert(0);
                    *e = (*e).max(n);
                }
            }
        }
    }
    Ok(out)
}

/// The IDs `sj ids --write` would assign, in file and heading order. Errors when a capability
/// has no usable prefix or two capabilities that need IDs share one.
pub fn plan(spec_root: &Path, prefixes: &BTreeMap<String, String>) -> Result<Vec<Assignment>> {
    // Highest number ever used per prefix, archive included (IDS-006).
    let mut next = highest_numbers(spec_root)?;

    // The files we may write, parsed once.
    let mut files = Vec::new();
    for file in spec::discover_spec_files(spec_root)? {
        let content = read(&file)?;
        let located = spec::locate_scenarios(&file, &content);
        let place = place(spec_root, &file);
        files.push((file, place, located));
    }

    // Prefixes for capabilities that need IDs, and collisions among them (IDS-004, IDS-005).
    let mut prefix_of: BTreeMap<String, String> = BTreeMap::new();
    for (file, place, located) in &files {
        if located.iter().all(|l| l.scenario.id.is_some()) {
            continue;
        }
        let Some(cap) = &place.capability else {
            bail!(
                "{} has scenarios without IDs but is not in a capability folder \
                 (specs/<capability>/spec.md)",
                file.display()
            );
        };
        if prefix_of.contains_key(cap) {
            continue;
        }
        let prefix = match prefixes.get(cap) {
            Some(p) => p.clone(),
            None => derive_prefix(cap),
        };
        if prefix.is_empty() {
            bail!(
                "capability `{cap}` has no letters to make a prefix; \
                 map it in [ids.prefixes] in specjutsu.toml"
            );
        }
        prefix_of.insert(cap.clone(), prefix);
    }
    let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
    for (cap, prefix) in &prefix_of {
        if let Some(other) = owner.insert(prefix, cap) {
            let mut pair = [other, cap.as_str()];
            pair.sort();
            bail!(
                "capabilities `{}` and `{}` both use prefix {prefix}; \
                 map one in [ids.prefixes] in specjutsu.toml",
                pair[0],
                pair[1]
            );
        }
    }

    // The first `specs/` scenario per (capability, requirement, name), for delta copies (IDS-007).
    type Key = (String, String, String);
    let key = |cap: &str, l: &spec::Located| -> Key {
        (
            cap.to_string(),
            l.requirement.clone().unwrap_or_default(),
            l.scenario.name.clone(),
        )
    };
    let mut origin: BTreeMap<Key, (usize, usize, Option<String>)> = BTreeMap::new();
    for (fi, (_, place, located)) in files.iter().enumerate() {
        let (true, Some(cap)) = (place.in_specs, &place.capability) else {
            continue;
        };
        for (li, l) in located.iter().enumerate() {
            origin
                .entry(key(cap, l))
                .or_insert((fi, li, l.scenario.id.clone()));
        }
    }

    let mut out = Vec::new();
    for (fi, (file, place, located)) in files.iter().enumerate() {
        let Some(cap) = &place.capability else {
            continue;
        };
        for (li, l) in located.iter().enumerate() {
            if l.scenario.id.is_some() {
                continue;
            }
            let k = key(cap, l);
            let copied = if place.in_specs {
                None
            } else {
                origin.get(&k).and_then(|(_, _, id)| id.clone())
            };
            let id = match copied {
                Some(id) => id,
                None => {
                    let prefix = &prefix_of[cap];
                    let n = next.entry(prefix.clone()).or_insert(0);
                    *n += 1;
                    format!("{prefix}-{:03}", *n)
                }
            };
            if place.in_specs {
                if let Some(o) = origin.get_mut(&k) {
                    if (o.0, o.1) == (fi, li) {
                        o.2 = Some(id.clone());
                    }
                }
            }
            out.push(Assignment {
                id,
                name: l.scenario.name.clone(),
                file: file.clone(),
                line: l.scenario.line,
                title_offset: l.title_offset,
            });
        }
    }
    Ok(out)
}

/// Insert each planned ID before its scenario name. Returns the number of files written.
pub fn write(plan: &[Assignment]) -> Result<usize> {
    let mut by_file: BTreeMap<&Path, Vec<&Assignment>> = BTreeMap::new();
    for a in plan {
        by_file.entry(a.file.as_path()).or_default().push(a);
    }
    let written = by_file.len();
    for (file, mut list) in by_file {
        let mut content = read(file)?;
        list.sort_by_key(|a| std::cmp::Reverse(a.title_offset));
        for a in list {
            let insert = if a.name.is_empty() {
                a.id.clone()
            } else {
                format!("{} ", a.id)
            };
            content.insert_str(a.title_offset, &insert);
        }
        std::fs::write(file, content)
            .with_context(|| format!("cannot write {}", file.display()))?;
    }
    Ok(written)
}
