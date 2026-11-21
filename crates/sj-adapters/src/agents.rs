//! Which adapters a repository gets: the names `--agents` takes, detection of the agents a
//! repository already has files for, and the combined plan for a list of them. `sj init`,
//! `sj update` and `sj migrate` share it.

use std::path::Path;

use anyhow::Result;

use crate::{
    agents_md, antigravity, claude, codex, copilot, cursor, gemini, kiro, plan::Step,
    shared_skills, windsurf,
};

/// Every name `--agents` accepts.
pub const ALL: [&str; 11] = [
    "claude",
    "codex",
    "cursor",
    "cline",
    "copilot",
    "gemini",
    "windsurf",
    "kiro",
    "antigravity",
    "agents-md",
    "auto",
];

/// A usage error naming the first unknown agent; checked before anything is written.
pub fn check(agents: &[String]) -> Result<()> {
    for a in agents {
        if !ALL.contains(&a.as_str()) {
            anyhow::bail!("unknown agent {a} (supported: {})", ALL.join(", "));
        }
    }
    Ok(())
}

/// The agents whose files the repository already has, in `ALL` order.
pub fn detect(root: &Path) -> Vec<&'static str> {
    let has = |p: &str| root.join(p).exists();
    let mut out = Vec::new();
    if has(".claude") || has("CLAUDE.md") {
        out.push("claude");
    }
    if has(".codex") {
        out.push("codex");
    }
    if has(".cursor") {
        out.push("cursor");
    }
    if has(".clinerules") {
        out.push("cline");
    }
    if copilot::is_present(root) {
        out.push("copilot");
    }
    if has(".gemini") || has("GEMINI.md") {
        out.push("gemini");
    }
    if has(".windsurf") {
        out.push("windsurf");
    }
    if has(".kiro") {
        out.push("kiro");
    }
    if has(".agents/rules") || has(".agent/rules") {
        out.push("antigravity");
    }
    if has("AGENTS.md") {
        out.push("agents-md");
    }
    out
}

/// The list with `auto` expanded, `agents-md` added for the agents that read AGENTS.md, and
/// duplicates dropped, in `ALL` order. The flag is false when `auto` found no agent files
/// and fell back to `agents-md`.
pub fn resolve(root: &Path, agents: &[String]) -> (Vec<String>, bool) {
    let mut wanted: Vec<&str> = agents.iter().map(String::as_str).collect();
    let mut found = true;
    if wanted.contains(&"auto") {
        let detected = detect(root);
        found = !detected.is_empty();
        wanted.extend(if found { detected } else { vec!["agents-md"] });
    }
    if wanted.iter().any(|a| {
        matches!(
            *a,
            "codex"
                | "cursor"
                | "cline"
                | "copilot"
                | "gemini"
                | "windsurf"
                | "kiro"
                | "antigravity"
        )
    }) {
        wanted.push("agents-md");
    }
    let out = ALL
        .iter()
        .filter(|a| **a != "auto" && wanted.contains(a))
        .map(|a| a.to_string())
        .collect();
    (out, found)
}

/// The adapters already installed in `root`, which `sj update` refreshes.
pub fn installed(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if claude::is_installed(root) {
        out.push("claude".to_string());
    }
    if codex::is_installed(root) {
        out.push("codex".to_string());
    }
    if cursor::is_installed(root) {
        out.push("cursor".to_string());
    }
    if copilot::is_installed(root) {
        out.push("copilot".to_string());
    }
    if gemini::is_installed(root) {
        out.push("gemini".to_string());
    }
    if windsurf::is_installed(root) {
        out.push("windsurf".to_string());
    }
    if kiro::is_installed(root) {
        out.push("kiro".to_string());
    }
    if antigravity::is_installed(root) {
        out.push("antigravity".to_string());
    }
    if agents_md::is_installed(root) {
        out.push("agents-md".to_string());
    }
    out
}

/// Every file installing or updating `agents` (resolved: no `auto`) would write, keep or
/// remove.
pub fn plan(root: &Path, agents: &[String], force: bool) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    for a in agents {
        match a.as_str() {
            "claude" => steps.extend(claude::plan(root, force)?),
            "codex" => steps.extend(codex::plan(root)?),
            "cursor" => steps.extend(cursor::plan(root)?),
            "copilot" => steps.extend(copilot::plan(root)?),
            "gemini" => steps.extend(gemini::plan(root)?),
            "windsurf" => steps.extend(windsurf::plan(root)?),
            "kiro" => steps.extend(kiro::plan(root)?),
            "antigravity" => steps.extend(antigravity::plan(root)?),
            // Cline's hooks run only in its CLI, so it gets the shared instructions only.
            "cline" => {}
            "agents-md" => steps.extend(agents_md::plan(root)?),
            other => anyhow::bail!("unknown agent {other}"),
        }
    }
    if agents
        .iter()
        .any(|a| shared_skills::READERS.contains(&a.as_str()))
    {
        steps.extend(shared_skills::plan(root, force)?);
    }
    Ok(steps)
}
