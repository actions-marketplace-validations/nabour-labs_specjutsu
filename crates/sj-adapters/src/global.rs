//! `--global`: the `sj-*` skills in the user's home folders, for every project.
//!
//! Only skills are written there. Hooks and instruction blocks are a repository's rules and
//! stay in the repository, where CI and teammates see them.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::claude::{self, remove_if_empty, Change};
use crate::plan::Step;
use crate::shared_skills;

/// The user's home folder: `HOME`, else `USERPROFILE` on Windows.
pub fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

fn reads_shared(agents: &[String]) -> bool {
    agents
        .iter()
        .any(|a| shared_skills::READERS.contains(&a.as_str()))
}

/// Every skill file installing `agents` into `home` would write, keep or remove:
/// `~/.claude/skills/` for Claude Code, `~/.agents/skills/` for the agents that read it.
pub fn plan(home: &Path, agents: &[String], force: bool) -> Result<Vec<Step>> {
    let mut out = Vec::new();
    if agents.iter().any(|a| a == "claude") {
        out.extend(claude::skills_plan(home, &claude::skills_dir(home), force)?);
    }
    if reads_shared(agents) {
        out.extend(shared_skills::plan(home, force)?);
    }
    Ok(out)
}

/// The update plan for the skill folders in `home` that already hold `sj-*` skills.
pub fn update_plan(home: &Path, force: bool) -> Result<Vec<Step>> {
    let mut out = Vec::new();
    if claude::has_skills(home) {
        out.extend(claude::skills_plan(home, &claude::skills_dir(home), force)?);
    }
    if shared_skills::is_installed(home) {
        out.extend(shared_skills::plan(home, force)?);
    }
    Ok(out)
}

/// Remove the skills `agents` installed into `home`. Nothing records which agents a home
/// folder was installed for, so naming any agent that reads `~/.agents/skills/` removes that
/// copy.
pub fn remove(home: &Path, agents: &[String]) -> Result<Vec<Change>> {
    let mut out = Vec::new();
    if agents.iter().any(|a| a == "claude") {
        out.extend(claude::skills_remove(home, &claude::skills_dir(home))?);
        remove_if_empty(&home.join(".claude"));
    }
    if reads_shared(agents) {
        out.extend(shared_skills::remove(home)?);
    }
    Ok(out)
}

/// Whether `base` (a repository root or a home folder) holds `sj-*` skills in either folder.
pub fn has_skills(base: &Path) -> bool {
    claude::has_skills(base) || shared_skills::is_installed(base)
}
