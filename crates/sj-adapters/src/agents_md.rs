//! The AGENTS.md adapter: one specjutsu block in `AGENTS.md`, read by Codex, Cursor, Cline,
//! Amp, Roo Code and other agents. It names `sj` commands, not skills, since not every agent
//! that reads AGENTS.md can load skills.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::claude::{
    block_install_with, block_remove, block_with, put, read_opt, rel, Change, BEGIN,
};

/// The AGENTS.md block body.
pub const BLOCK_BODY: &str = include_str!("../assets/agents-md/AGENTS.block.md");

fn path(root: &Path) -> PathBuf {
    root.join("AGENTS.md")
}

/// The block as written, markers included.
pub fn block() -> String {
    block_with(BLOCK_BODY)
}

/// Whether `AGENTS.md` holds the specjutsu block.
pub fn is_installed(root: &Path) -> bool {
    std::fs::read_to_string(path(root)).is_ok_and(|t| t.contains(BEGIN))
}

/// Write the block into `AGENTS.md`, or refresh it in place.
pub fn install(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let old = read_opt(&p)?;
    let new = block_install_with(old.as_deref(), &block());
    let mut out = Vec::new();
    put(&p, root, old.as_deref(), &new, &mut out)?;
    Ok(out)
}

/// Remove the block; an `AGENTS.md` left empty is deleted.
pub fn remove(root: &Path) -> Result<Vec<Change>> {
    let p = path(root);
    let mut out = Vec::new();
    if let Some(old) = read_opt(&p)? {
        let new = block_remove(&old);
        if new.is_empty() {
            std::fs::remove_file(&p)?;
            out.push(Change::Removed(rel(&p, root)));
        } else if new != old {
            put(&p, root, Some(&old), &new, &mut out)?;
        }
    }
    Ok(out)
}
