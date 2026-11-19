//! `sj instructions`: what to write next for one artifact of a change (LOOP-012..019).

use std::path::Path;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::authoring::{self, find_change, spec_files, Change};
use crate::config::Config;
use crate::context::{self, ContextText, Rules};

/// The artifacts of a change, in writing order.
pub const ARTIFACTS: [&str; 4] = ["proposal", "specs", "design", "tasks"];

/// The structural keywords that stay English in every language.
pub const KEYWORDS: &str =
    "`### Requirement:`, `#### Scenario:`, `WHEN`, `THEN`, `SHALL`, `MUST`, \
`## Purpose` and the delta section headers `## ADDED Requirements`, `## MODIFIED Requirements`, \
`## REMOVED Requirements`, `## RENAMED Requirements`";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Instructions {
    pub change: String,
    pub artifact: String,
    /// The file to write, relative to the repository root.
    pub file: String,
    /// The resolved template path, or `built-in:<name>`.
    pub template: String,
    pub context: Vec<ContextText>,
    pub rules: Vec<Rules>,
    /// The next free ID for every prefix in use, e.g. `AUTH-006`.
    pub next_ids: Vec<String>,
    pub language: Option<String>,
    /// The command that checks the artifact.
    pub check: String,
    #[serde(skip)]
    pub warnings: Vec<String>,
}

pub enum Outcome {
    Artifact(Box<Instructions>),
    /// Every artifact is written.
    AllWritten {
        change: String,
    },
}

fn template_name(artifact: &str) -> &'static str {
    match artifact {
        "proposal" => "proposal.md",
        "specs" => "spec.md",
        "design" => "design.md",
        _ => "tasks.md",
    }
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Whether `artifact` is missing or still identical to its resolved template (decision 3).
fn unwritten(spec_root: &Path, change: &Change, artifact: &str) -> Result<bool> {
    let template = authoring::template_text(spec_root, template_name(artifact))?;
    if artifact == "specs" {
        for f in spec_files(&change.dir.join("specs"))? {
            if std::fs::read_to_string(&f)? != template {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    let file = change.dir.join(format!("{artifact}.md"));
    Ok(match std::fs::read_to_string(&file) {
        Ok(text) => text == template,
        Err(_) => true,
    })
}

/// Instructions for `artifact` of `change`, or for its first unwritten artifact.
pub fn instructions(
    root: &Path,
    spec_root: &Path,
    config: &Config,
    change: &str,
    artifact: Option<&str>,
) -> Result<Outcome> {
    if let Some(a) = artifact {
        if !ARTIFACTS.contains(&a) {
            bail!("unknown artifact `{a}` (known: {})", ARTIFACTS.join(", "));
        }
    }
    let c = find_change(spec_root, change)?;
    let artifact = match artifact {
        Some(a) => a,
        None => {
            let mut first = None;
            for a in ARTIFACTS {
                if unwritten(spec_root, &c, a)? {
                    first = Some(a);
                    break;
                }
            }
            match first {
                Some(a) => a,
                None => {
                    return Ok(Outcome::AllWritten {
                        change: change.to_string(),
                    })
                }
            }
        }
    };

    let project = context::load(root, spec_root, config)?;
    let file = if artifact == "specs" {
        format!("{}/specs/<capability>/spec.md", rel(&c.dir, root))
    } else {
        format!("{}/{artifact}.md", rel(&c.dir, root))
    };
    let next_ids = crate::ids::highest_numbers(spec_root)?
        .into_iter()
        .map(|(prefix, n)| format!("{prefix}-{:03}", n + 1))
        .collect();
    let check = if artifact == "tasks" {
        format!("sj spec check {change}")
    } else {
        format!("sj spec validate {change}")
    };
    Ok(Outcome::Artifact(Box::new(Instructions {
        change: change.to_string(),
        artifact: artifact.to_string(),
        file,
        template: authoring::template(root, spec_root, template_name(artifact)).path,
        rules: project.rules_for(artifact),
        context: project.context,
        next_ids,
        language: project.language,
        check,
        warnings: project.warnings,
    })))
}

impl Instructions {
    /// The text form: one `key value` line per field, context and rules indented below
    /// their source.
    pub fn render(&self) -> String {
        let mut out = format!(
            "change {}\nartifact {}\nwrite {}\ntemplate {}\n",
            self.change, self.artifact, self.file, self.template
        );
        for c in &self.context {
            out.push_str(&format!("context ({})\n", c.source));
            for l in c.text.lines() {
                out.push_str(&format!("  {l}\n").replace("  \n", "\n"));
            }
        }
        for r in &self.rules {
            out.push_str(&format!("rules for {} ({})\n", self.artifact, r.source));
            for l in &r.rules {
                out.push_str(&format!("  - {l}\n"));
            }
        }
        for id in &self.next_ids {
            out.push_str(&format!("next {id}\n"));
        }
        if let Some(l) = &self.language {
            out.push_str(&format!(
                "language {l}: write prose in {l}; keep {KEYWORDS} in English\n"
            ));
        }
        out.push_str(&format!("check {}\n", self.check));
        out
    }
}
