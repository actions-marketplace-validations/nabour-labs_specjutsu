//! Project context and per-artifact rules, read live on every run (CONF-014..016, CONF-029).
//!
//! Sources, in order: `<spec_root>/principles.md` (text before `## Rules` is the context, each
//! `### <artifact>` under `## Rules` holds that artifact's rules), then an unmigrated OpenSpec
//! `<spec_root>/config.yaml` (`context`, `rules.<artifact>` and `language`), which is never
//! written.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, Result};
use serde::Serialize;

use crate::config::{is_language_tag, Config};

/// Context text from one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextText {
    pub source: String,
    pub text: String,
}

/// One artifact's rules from one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Rules {
    pub source: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Project {
    pub context: Vec<ContextText>,
    /// Artifact name to its rules, one entry per source that has any.
    pub rules: BTreeMap<String, Vec<Rules>>,
    /// `specjutsu.toml`'s `language`, else a valid one from `config.yaml`.
    pub language: Option<String>,
    pub warnings: Vec<String>,
}

impl Project {
    pub fn rules_for(&self, artifact: &str) -> Vec<Rules> {
        self.rules.get(artifact).cloned().unwrap_or_default()
    }
}

fn label(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Read both sources under `spec_root` (a path under `root`).
pub fn load(root: &Path, spec_root: &Path, config: &Config) -> Result<Project> {
    let mut p = Project {
        language: config.language.clone(),
        ..Project::default()
    };

    let principles = spec_root.join("principles.md");
    if principles.is_file() {
        let text = std::fs::read_to_string(&principles)
            .with_context(|| format!("cannot read {}", principles.display()))?;
        let (context, rules) = parse_principles(&text);
        add(&mut p, &label(&principles, root), context, rules);
    }

    let yaml = spec_root.join("config.yaml");
    if yaml.is_file() {
        let source = label(&yaml, root);
        let text = std::fs::read_to_string(&yaml)
            .with_context(|| format!("cannot read {}", yaml.display()))?;
        let y = parse_yaml(&text);
        for w in y.warnings {
            p.warnings.push(format!("{source}: {w}; ignored"));
        }
        if p.language.is_none() {
            if let Some(tag) = y.language {
                if is_language_tag(&tag) {
                    p.language = Some(tag);
                } else {
                    p.warnings.push(format!(
                        "{source}: language `{tag}` is not a BCP-47 tag; ignored"
                    ));
                }
            }
        }
        add(&mut p, &source, y.context.unwrap_or_default(), y.rules);
    }
    Ok(p)
}

fn add(p: &mut Project, source: &str, context: String, rules: BTreeMap<String, Vec<String>>) {
    let context = context.trim();
    if !context.is_empty() {
        p.context.push(ContextText {
            source: source.to_string(),
            text: context.to_string(),
        });
    }
    for (artifact, list) in rules {
        if list.is_empty() {
            continue;
        }
        p.rules.entry(artifact).or_default().push(Rules {
            source: source.to_string(),
            rules: list,
        });
    }
}

/// Split `principles.md` into its context and its `## Rules` sections.
fn parse_principles(text: &str) -> (String, BTreeMap<String, Vec<String>>) {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let mut fence = false;
    let mut rules_at = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim_start().starts_with("```") {
            fence = !fence;
        }
        if !fence && l.trim_end() == "## Rules" {
            rules_at = Some(i);
            break;
        }
    }
    let Some(start) = rules_at else {
        return (text.trim().to_string(), BTreeMap::new());
    };
    let context = lines[..start].join("\n").trim().to_string();
    let mut rules: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for l in &lines[start + 1..] {
        let t = l.trim();
        if l.starts_with("## ") {
            break;
        }
        if let Some(name) = l.strip_prefix("### ") {
            let name = name.trim().to_lowercase();
            rules.entry(name.clone()).or_default();
            current = Some(name);
            continue;
        }
        let Some(name) = &current else { continue };
        if t.is_empty() {
            continue;
        }
        let item = t
            .strip_prefix("- ")
            .or_else(|| t.strip_prefix("* "))
            .unwrap_or(t);
        if let Some(list) = rules.get_mut(name) {
            list.push(item.to_string());
        }
    }
    (context, rules)
}

#[derive(Debug, Default)]
struct Yaml {
    context: Option<String>,
    language: Option<String>,
    rules: BTreeMap<String, Vec<String>>,
    warnings: Vec<String>,
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\\\", "\\")
    } else if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        s[1..s.len() - 1].replace("''", "'")
    } else {
        s.to_string()
    }
}

/// Split a flow sequence `[a, "b, c"]` into its items.
fn flow_list(v: &str) -> Option<Vec<String>> {
    let inner = v.trim().strip_prefix('[')?.strip_suffix(']')?;
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match (quote, c) {
            (None, '"' | '\'') => {
                quote = Some(c);
                cur.push(c);
            }
            (Some(q), c) if c == q => {
                quote = None;
                cur.push(c);
            }
            (None, ',') => {
                out.push(unquote(&cur));
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(unquote(&cur));
    }
    Some(out.into_iter().filter(|s| !s.is_empty()).collect())
}

/// A block scalar starting at line `i`, indented deeper than `ind`; returns the text and the
/// index of the first line after it.
fn block(lines: &[&str], mut i: usize, ind: usize, folded: bool) -> (String, usize) {
    let mut body: Vec<&str> = Vec::new();
    let mut inner = None;
    while i < lines.len() {
        let l = lines[i];
        if l.trim().is_empty() {
            body.push("");
            i += 1;
            continue;
        }
        let li = indent(l);
        if li <= ind {
            break;
        }
        let n = *inner.get_or_insert(li);
        body.push(l.get(n.min(li)..).unwrap_or(""));
        i += 1;
    }
    while body.last() == Some(&"") {
        body.pop();
    }
    let text = if folded {
        body.join(" ")
    } else {
        body.join("\n")
    };
    (text, i)
}

/// Line-based reader for the keys OpenSpec's `config.yaml` uses: `context` (plain, quoted or
/// block scalar), `language`, and `rules` as a map of `- item` lists or `[a, b]` flow lists.
/// Other keys are skipped.
fn parse_yaml(text: &str) -> Yaml {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let mut y = Yaml::default();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        i += 1;
        if t.is_empty() || t.starts_with('#') || indent(line) > 0 {
            continue;
        }
        let Some((key, value)) = t.split_once(':') else {
            y.warnings.push(format!("cannot read line {i}"));
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "context" => match value {
                "|" | "|-" | "|+" | ">" | ">-" | ">+" => {
                    let (body, next) = block(&lines, i, 0, value.starts_with('>'));
                    y.context = Some(body);
                    i = next;
                }
                v => y.context = Some(unquote(v)),
            },
            "language" => y.language = Some(unquote(value)),
            "rules" => {
                if !value.is_empty() {
                    y.warnings.push("rules must be a map of lists".into());
                    continue;
                }
                let mut current: Option<String> = None;
                while i < lines.len() {
                    let l = lines[i];
                    let lt = l.trim();
                    if lt.is_empty() || lt.starts_with('#') {
                        i += 1;
                        continue;
                    }
                    if indent(l) == 0 {
                        break;
                    }
                    i += 1;
                    if let Some(item) = lt.strip_prefix("- ") {
                        match current.as_ref().and_then(|a| y.rules.get_mut(a)) {
                            Some(list) => list.push(unquote(item)),
                            None => y.warnings.push(format!("cannot read rules line {i}")),
                        }
                        continue;
                    }
                    let Some((a, v)) = lt.split_once(':') else {
                        y.warnings.push(format!("cannot read rules line {i}"));
                        continue;
                    };
                    let a = a.trim().to_lowercase();
                    let list = y.rules.entry(a.clone()).or_default();
                    let v = v.trim();
                    if v.is_empty() {
                        current = Some(a);
                    } else if let Some(items) = flow_list(v) {
                        list.extend(items);
                        current = None;
                    } else {
                        list.push(unquote(v));
                        current = None;
                    }
                }
            }
            _ => {}
        }
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principles_split_context_and_rules() {
        let (ctx, rules) = parse_principles(
            "# P\n\nRust CLI\n\n## Rules\n\n### tasks\n\n- One\n- Two\n\n### design\n\n- Three\n",
        );
        assert_eq!(ctx, "# P\n\nRust CLI");
        assert_eq!(rules["tasks"], vec!["One", "Two"]);
        assert_eq!(rules["design"], vec!["Three"]);
    }

    #[test]
    fn yaml_reads_block_context_flow_and_block_rules() {
        let y = parse_yaml(
            "schema: spec-driven\ncontext: |\n  Line one\n  Line two\nlanguage: 'fr'\nrules:\n  proposal: [\"A, b\", C]\n  tasks:\n    - D\n",
        );
        assert_eq!(y.context.as_deref(), Some("Line one\nLine two"));
        assert_eq!(y.language.as_deref(), Some("fr"));
        assert_eq!(y.rules["proposal"], vec!["A, b", "C"]);
        assert_eq!(y.rules["tasks"], vec!["D"]);
        assert!(y.warnings.is_empty());
    }
}
