//! How strongly each installed agent is held to the loop, for `sj doctor`.
//!
//! An agent counts as installed from the files in the repository. Claude Code counts when its
//! CLAUDE.md block or hooks exist. Codex and Cursor count when their hook entry exists, or when
//! the AGENTS.md block exists and the agent's folder does (`.codex/`, `.cursor/`): then the
//! agent reads the instructions but its hook entry is gone. Cline's adapter writes only the
//! AGENTS.md block, so Cline counts when that block exists and the repository has
//! `.clinerules`.

use std::path::Path;

use crate::{agents_md, claude, codex, cursor};

/// `hook`: edits are checked before they happen. `instructions`: the agent is told the loop
/// and the CI gate is the backstop. `hook missing`: the adapter is installed but its hook
/// entry is gone.
pub fn levels(root: &Path) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    let shared = agents_md::is_installed(root);
    let hooked = |agent: &'static str, hook: bool, present: bool| match (hook, present) {
        (true, _) => Some((agent, "hook")),
        (false, true) => Some((agent, "hook missing")),
        (false, false) => None,
    };
    out.extend(hooked(
        "claude",
        claude::has_hook(root),
        claude::is_installed(root),
    ));
    out.extend(hooked(
        "codex",
        codex::is_installed(root),
        shared && root.join(".codex").exists(),
    ));
    out.extend(hooked(
        "cursor",
        cursor::is_installed(root),
        shared && root.join(".cursor").exists(),
    ));
    if shared && root.join(".clinerules").exists() {
        out.push(("cline", "instructions"));
    }
    if shared {
        out.push(("agents-md", "instructions"));
    }
    out
}
