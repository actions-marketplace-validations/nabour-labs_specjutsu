//! The `sj-*` skills in `.agents/skills/`, the folder Codex, Gemini CLI and Antigravity read.
//!
//! They are written as plain files with the same content rules as Claude Code's copy under
//! `.claude/skills/`, so the two copies are byte-identical and `sj update` keeps both in step.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::claude::{remove_if_empty, skills_plan, skills_remove, Change, SKILLS};
use crate::plan::Step;

/// The agents that read skills from `.agents/skills/`.
pub const READERS: [&str; 3] = ["codex", "gemini", "antigravity"];

/// `.agents/skills/` under `base`, a repository root or a home folder.
pub fn dir(base: &Path) -> PathBuf {
    base.join(".agents").join("skills")
}

/// Whether any `sj-*` skill is in `.agents/skills/` under `base`.
pub fn is_installed(base: &Path) -> bool {
    let d = dir(base);
    SKILLS
        .iter()
        .any(|(name, _)| d.join(name).join("SKILL.md").exists())
}

/// Every step that brings the shared skills up to date; hand-edited ones are kept unless
/// `force`.
pub fn plan(base: &Path, force: bool) -> Result<Vec<Step>> {
    skills_plan(base, &dir(base), force)
}

/// Delete the unmodified shared skills, leaving every other skill in the folder.
pub fn remove(base: &Path) -> Result<Vec<Change>> {
    let out = skills_remove(base, &dir(base))?;
    remove_if_empty(&base.join(".agents"));
    Ok(out)
}
