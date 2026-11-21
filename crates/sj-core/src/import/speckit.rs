//! `sj import speckit`: convert Spec Kit features (`specs/NNN-<name>/`) into the specjutsu layout.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;

use crate::ids;

/// What a target file holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Content {
    /// A byte copy of this source file.
    Copy(PathBuf),
    /// Generated text.
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Output {
    pub path: PathBuf,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Feature {
    pub name: String,
    /// The feature folder, `specs/NNN-<name>/`.
    pub folder: PathBuf,
    /// No unchecked task: the feature becomes current specs plus an archived change.
    pub finished: bool,
    pub requirements: usize,
    pub scenarios: usize,
    /// FR IDs no acceptance scenario names.
    pub gaps: Vec<String>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub features: Vec<Feature>,
    pub principles: Option<Output>,
}

impl Plan {
    pub fn outputs(&self) -> impl Iterator<Item = &Output> {
        self.features
            .iter()
            .flat_map(|f| &f.outputs)
            .chain(self.principles.as_ref())
    }

    /// Target paths that already exist.
    pub fn conflicts(&self) -> Vec<PathBuf> {
        self.outputs()
            .filter(|o| o.path.exists())
            .map(|o| o.path.clone())
            .collect()
    }
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).unwrap())
}

fn read(p: &Path) -> Result<String> {
    let t = std::fs::read_to_string(p).with_context(|| format!("cannot read {}", p.display()))?;
    Ok(t.strip_prefix('\u{feff}')
        .unwrap_or(&t)
        .replace("\r\n", "\n")
        .replace('\r', "\n"))
}

/// Cut `s` to at most `max` characters at a word boundary.
fn cut_words(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    for w in s.split_whitespace() {
        let next = if out.is_empty() {
            w.chars().count()
        } else {
            out.chars().count() + 1 + w.chars().count()
        };
        if next > max {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(w);
    }
    if out.is_empty() {
        s.chars().take(max).collect()
    } else {
        out
    }
}

/// Trim blank lines at both ends of a block of lines.
fn trim_lines(lines: &[&str]) -> String {
    let start = lines.iter().position(|l| !l.trim().is_empty());
    let end = lines.iter().rposition(|l| !l.trim().is_empty());
    match (start, end) {
        (Some(s), Some(e)) => lines[s..=e].join("\n"),
        _ => String::new(),
    }
}

/// A `#`-level section: its heading line and body lines.
struct Section<'a> {
    title: String,
    heading: &'a str,
    body: Vec<&'a str>,
}

/// Split `lines` into sections at headings of exactly `level` `#`s, outside fences. Lines before
/// the first heading come back as the preamble.
fn sections<'a>(lines: &[&'a str], level: usize) -> (Vec<&'a str>, Vec<Section<'a>>) {
    let marker = format!("{} ", "#".repeat(level));
    let mut pre = Vec::new();
    let mut out: Vec<Section> = Vec::new();
    let mut fence = false;
    for &l in lines {
        if l.trim_start().starts_with("```") || l.trim_start().starts_with("~~~") {
            fence = !fence;
        }
        if !fence && l.starts_with(&marker) {
            out.push(Section {
                title: l[marker.len()..].trim().to_string(),
                heading: l,
                body: Vec::new(),
            });
            continue;
        }
        match out.last_mut() {
            Some(s) => s.body.push(l),
            None => pre.push(l),
        }
    }
    (pre, out)
}

/// A section title without a trailing `*(mandatory)*`-style note, lowercased.
fn key(title: &str) -> String {
    let t = title.split(" *(").next().unwrap_or(title);
    t.trim().to_ascii_lowercase()
}

struct Story {
    n: usize,
    title: String,
    text: String,
    items: Vec<Item>,
}

struct Item {
    given: Option<String>,
    when: Option<String>,
    then: Option<String>,
    raw: String,
}

fn clause(s: &str) -> String {
    s.trim().trim_end_matches(',').trim().to_string()
}

fn parse_item(text: &str) -> Item {
    static MARK: OnceLock<Regex> = OnceLock::new();
    let mark = re(&MARK, r"\*\*(Given|When|Then)\*\*");
    let mut parts: Vec<(String, usize, usize)> = Vec::new();
    for m in mark.captures_iter(text) {
        let whole = m.get(0).unwrap();
        parts.push((m[1].to_string(), whole.start(), whole.end()));
    }
    let mut item = Item {
        given: None,
        when: None,
        then: None,
        raw: text.to_string(),
    };
    for (i, (kind, _, end)) in parts.iter().enumerate() {
        let stop = parts.get(i + 1).map_or(text.len(), |p| p.1);
        let c = clause(&text[*end..stop]);
        match kind.as_str() {
            "Given" => item.given = Some(c),
            "When" => item.when = Some(c),
            _ => item.then = Some(c),
        }
    }
    item
}

fn parse_story(n: usize, title: &str, body: &[&str]) -> Story {
    static ITEM: OnceLock<Regex> = OnceLock::new();
    let item_re = re(&ITEM, r"^\s*\d+\.\s+(.*)$");
    let title = title
        .split(" (Priority")
        .next()
        .unwrap_or(title)
        .trim()
        .to_string();
    let cut = body
        .iter()
        .position(|l| l.trim_start().starts_with("**Acceptance Scenarios**"))
        .unwrap_or(body.len());
    let text = trim_lines(
        &body[..cut]
            .iter()
            .copied()
            .filter(|l| l.trim() != "---")
            .collect::<Vec<_>>(),
    );
    let items = body[cut..]
        .iter()
        .filter_map(|l| item_re.captures(l))
        .map(|c| parse_item(&c[1]))
        .collect();
    Story {
        n,
        title,
        text,
        items,
    }
}

struct Fr {
    id: String,
    text: String,
}

fn parse_frs(body: &[&str]) -> Vec<Fr> {
    static FR: OnceLock<Regex> = OnceLock::new();
    let fr = re(&FR, r"^\s*[-*]\s+\*\*(FR-[0-9]+[a-z]*)\*\*:\s*(.*?)\s*$");
    body.iter()
        .filter_map(|l| fr.captures(l))
        .map(|c| Fr {
            id: c[1].to_string(),
            text: c[2].to_string(),
        })
        .collect()
}

fn first_clause(text: &str) -> String {
    let end = [text.find(';'), text.find(':'), text.find(", ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(text.len());
    cut_words(text[..end].trim(), 60)
}

/// The parts of a Spec Kit `spec.md` the conversion needs.
struct Parsed {
    created: Option<String>,
    purpose: String,
    stories: Vec<Story>,
    frs: Vec<Fr>,
    proposal: String,
}

fn parse_spec(text: &str, folder: &str) -> Parsed {
    static CREATED: OnceLock<Regex> = OnceLock::new();
    static INPUT: OnceLock<Regex> = OnceLock::new();
    let created_re = re(&CREATED, r"^\*\*Created\*\*:\s*(\d{4}-\d{2}-\d{2})\b");
    let input_re = re(&INPUT, r"^\*\*Input\*\*:\s*(.*)$");
    let lines: Vec<&str> = text.split('\n').collect();
    let (pre, top) = sections(&lines, 2);

    let created = pre
        .iter()
        .find_map(|l| created_re.captures(l).map(|c| c[1].to_string()));
    let input = pre
        .iter()
        .find_map(|l| input_re.captures(l).map(|c| c[1].trim().to_string()));

    let mut stories = Vec::new();
    let mut frs = Vec::new();
    let mut overview: Option<String> = None;
    let mut kept: Vec<String> = Vec::new();
    for s in &top {
        match key(&s.title).as_str() {
            "overview" => overview = Some(trim_lines(&s.body)),
            "user scenarios & testing" => {
                let (_, subs) = sections(&s.body, 3);
                for sub in subs {
                    if let Some(rest) = sub.title.strip_prefix("User Story ") {
                        let n = rest
                            .split(|c: char| !c.is_ascii_digit())
                            .next()
                            .and_then(|d| d.parse().ok())
                            .unwrap_or(stories.len() + 1);
                        let title = rest.split_once(" - ").map_or(rest, |(_, t)| t);
                        stories.push(parse_story(n, title, &sub.body));
                    } else {
                        kept.push(format!("## {}\n\n{}", sub.title, trim_lines(&sub.body)));
                    }
                }
            }
            "requirements" => {
                let (_, subs) = sections(&s.body, 3);
                for sub in subs {
                    if key(&sub.title) == "functional requirements" {
                        frs.extend(parse_frs(&sub.body));
                        // Verbatim too, so group labels between FR lines aren't lost.
                        kept.push(format!("## {}\n\n{}", sub.title, trim_lines(&sub.body)));
                    } else {
                        kept.push(format!("## {}\n\n{}", sub.title, trim_lines(&sub.body)));
                    }
                }
            }
            _ => kept.push(format!("{}\n\n{}", s.heading, trim_lines(&s.body))),
        }
    }

    let purpose = overview
        .as_deref()
        .and_then(|o| o.split("\n\n").next())
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .or_else(|| input.clone())
        .unwrap_or_else(|| format!("Imported from Spec Kit feature {folder}."));
    let why = overview
        .filter(|o| !o.is_empty())
        .or(input)
        .unwrap_or_else(|| format!("Imported from Spec Kit feature {folder}."));
    let mut proposal = format!("## Why\n\n{why}\n\n## Source\n\n{}\n", trim_lines(&pre));
    for k in kept {
        proposal.push_str(&format!("\n{}\n", k.trim_end()));
    }
    Parsed {
        created,
        purpose,
        stories,
        frs,
        proposal,
    }
}

/// Requirement blocks for one feature, numbering scenarios from `next`.
fn requirements(p: &Parsed, prefix: &str, next: &mut u64) -> (String, usize, Vec<String>) {
    let mut out = String::new();
    let mut scenarios = 0;
    for s in &p.stories {
        out.push_str(&format!("### Requirement: {}\n", s.title));
        if !s.text.is_empty() {
            out.push_str(&format!("{}\n\n", s.text));
        }
        out.push_str(&format!(
            "The system SHALL support this story: {}.\n",
            s.title.trim_end_matches('.')
        ));
        for (m, item) in s.items.iter().enumerate() {
            *next += 1;
            scenarios += 1;
            let lead = item.when.as_deref().or(item.given.as_deref()).unwrap_or("");
            out.push_str(&format!(
                "\n#### Scenario: {prefix}-{:03} US{}.{} {}\n",
                *next,
                s.n,
                m + 1,
                cut_words(lead, 60)
            ));
            match (&item.given, &item.when) {
                (Some(g), Some(w)) => {
                    out.push_str(&format!("- **GIVEN** {g}\n- **WHEN** {w}\n"));
                }
                (None, Some(w)) => out.push_str(&format!("- **WHEN** {w}\n")),
                (Some(g), None) => out.push_str(&format!("- **WHEN** {g}\n")),
                (None, None) => {}
            }
            if let Some(t) = &item.then {
                out.push_str(&format!("- **THEN** {t}\n"));
            }
        }
        out.push('\n');
    }
    let mut gaps = Vec::new();
    for fr in &p.frs {
        out.push_str(&format!(
            "### Requirement: {} {}\n{}\n",
            fr.id,
            first_clause(&fr.text),
            fr.text
        ));
        let covered = p.stories.iter().flat_map(|s| &s.items).any(|i| {
            Regex::new(&format!(r"\b{}\b", regex::escape(&fr.id)))
                .map(|r| r.is_match(&i.raw))
                .unwrap_or(false)
        });
        if !covered {
            out.push_str(&format!(
                "\n[NEEDS CLARIFICATION: add a scenario for {}]\n",
                fr.id
            ));
            gaps.push(fr.id.clone());
        }
        out.push('\n');
    }
    (out.trim_end().to_string() + "\n", scenarios, gaps)
}

/// Files under `dir`, recursively, sorted.
fn files_in(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("cannot read {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            out.extend(files_in(&p)?);
        } else {
            out.push(p);
        }
    }
    Ok(out)
}

fn has_open_task(tasks: &Path) -> Result<bool> {
    static OPEN: OnceLock<Regex> = OnceLock::new();
    let open = re(&OPEN, r"(?m)^\s*[-*] \[ \]");
    if !tasks.is_file() {
        return Ok(false);
    }
    Ok(open.is_match(&read(tasks)?))
}

/// Work out every file the import would write. `spec_root` is absolute; `today` names the
/// archive of a finished feature without a `**Created**` date.
pub fn plan(
    root: &Path,
    spec_root: &Path,
    prefixes: &BTreeMap<String, String>,
    today: &str,
) -> Result<Plan> {
    static FOLDER: OnceLock<Regex> = OnceLock::new();
    let folder_re = re(&FOLDER, r"^\d+-(.+)$");
    let mut highest = ids::highest_numbers(spec_root)?;
    let mut plan = Plan::default();

    let specs = root.join("specs");
    let mut folders: Vec<PathBuf> = if specs.is_dir() {
        std::fs::read_dir(&specs)
            .with_context(|| format!("cannot read {}", specs.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join("spec.md").is_file())
            .collect()
    } else {
        Vec::new()
    };
    folders.sort();

    for folder in folders {
        let folder_name = folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let Some(name) = folder_re.captures(&folder_name).map(|c| c[1].to_string()) else {
            continue;
        };
        let parsed = parse_spec(&read(&folder.join("spec.md"))?, &folder_name);
        let prefix = prefixes
            .get(&name)
            .cloned()
            .unwrap_or_else(|| ids::derive_prefix(&name));
        if !ids::is_prefix(&prefix) {
            bail!("no ID prefix for `{name}`; set one under [ids.prefixes] in specjutsu.toml");
        }
        let next = highest.entry(prefix.clone()).or_insert(0);
        let (blocks, scenarios, gaps) = requirements(&parsed, &prefix, next);
        let finished = !has_open_task(&folder.join("tasks.md"))?;

        let (spec_path, spec_text, change_dir) = if finished {
            let date = parsed.created.clone().unwrap_or_else(|| today.to_string());
            (
                spec_root.join("specs").join(&name).join("spec.md"),
                format!(
                    "# {name} Specification\n\n## Purpose\n{}\n\n## Requirements\n\n{blocks}",
                    parsed.purpose
                ),
                spec_root
                    .join("changes")
                    .join("archive")
                    .join(format!("{date}-{name}")),
            )
        } else {
            let dir = spec_root.join("changes").join(&name);
            (
                dir.join("specs").join(&name).join("spec.md"),
                format!("## ADDED Requirements\n\n{blocks}"),
                dir,
            )
        };
        let mut outputs = vec![
            Output {
                path: spec_path,
                content: Content::Text(spec_text),
            },
            Output {
                path: change_dir.join("proposal.md"),
                content: Content::Text(parsed.proposal.clone()),
            },
        ];
        for src in files_in(&folder)? {
            let rel = src.strip_prefix(&folder).unwrap_or(&src);
            let target = match rel.to_str() {
                Some("spec.md") => continue,
                Some("plan.md") => PathBuf::from("design.md"),
                _ => rel.to_path_buf(),
            };
            outputs.push(Output {
                path: change_dir.join(target),
                content: Content::Copy(src.clone()),
            });
        }
        plan.features.push(Feature {
            name,
            folder: folder.clone(),
            finished,
            requirements: parsed.stories.len() + parsed.frs.len(),
            scenarios,
            gaps,
            outputs,
        });
    }

    let constitution = root.join(".specify").join("memory").join("constitution.md");
    if constitution.is_file() {
        plan.principles = Some(Output {
            path: spec_root.join("principles.md"),
            content: Content::Copy(constitution),
        });
    }
    Ok(plan)
}

/// Write every output. Without `force`, any existing target is an error and nothing is written.
pub fn write(plan: &Plan, force: bool) -> Result<()> {
    if !force {
        let conflicts = plan.conflicts();
        if !conflicts.is_empty() {
            bail!(
                "{} already exist(s); pass --force to overwrite: {}",
                conflicts.len(),
                conflicts
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    for o in plan.outputs() {
        if let Some(dir) = o.path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("cannot create {}", dir.display()))?;
        }
        match &o.content {
            Content::Text(t) => std::fs::write(&o.path, t),
            Content::Copy(src) => std::fs::copy(src, &o.path).map(|_| ()),
        }
        .with_context(|| format!("cannot write {}", o.path.display()))?;
    }
    Ok(())
}
