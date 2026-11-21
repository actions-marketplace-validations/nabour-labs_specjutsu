//! A JSON file of hook entries keyed by event, as Copilot, Windsurf and Kiro use: one entry
//! added after the user's, removed byte for byte, and the file deleted when only what
//! specjutsu wrote is left.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::claude::{
    has_hooks, hooks_install, hooks_remove, put, read_opt, rel, remove_if_empty, root_object,
    Change,
};
use crate::jsonedit::Node;

/// Where one agent's entry goes and how to recognise it.
pub(crate) struct HookFile {
    /// The file, relative to the root.
    pub path: &'static [&'static str],
    /// The event key under `hooks`.
    pub event: &'static str,
    /// The entry, as JSON text.
    pub entry: &'static str,
    /// Whether an entry is ours.
    pub ours: fn(&Node) -> bool,
    /// Whether a new file starts with `"version": 1`.
    pub versioned: bool,
}

/// Whether `entry[key]` is a command starting with `prefix`.
pub(crate) fn command_is(entry: &Node, key: &str, prefix: &str) -> bool {
    entry
        .get(key)
        .and_then(|c| c.value.as_str())
        .is_some_and(|c| c.starts_with(prefix))
}

impl HookFile {
    fn file(&self, root: &Path) -> PathBuf {
        self.path.iter().fold(root.to_path_buf(), |p, c| p.join(c))
    }

    /// Whether the file has our entry.
    pub fn is_installed(&self, root: &Path) -> bool {
        std::fs::read_to_string(self.file(root)).is_ok_and(|t| has_hooks(&t, self.ours))
    }

    /// Add the entry, keeping every other byte. Nothing is written when the file isn't
    /// valid JSON.
    pub fn plan(&self, root: &Path) -> Result<Vec<crate::plan::Step>> {
        let p = self.file(root);
        let old = read_opt(&p)?;
        let (event, entry) = (self.event, self.entry);
        let new = match old.as_deref() {
            None => {
                let version = if self.versioned {
                    "  \"version\": 1,\n"
                } else {
                    ""
                };
                format!(
                    "{{\n{version}  \"hooks\": {{\n    \"{event}\": [\n      {entry}\n    ]\n  }}\n}}\n"
                )
            }
            Some(t) => hooks_install(t, &[(event, entry)], self.ours)
                .with_context(|| format!("{} is not valid JSON", p.display()))?,
        };
        Ok(crate::plan::put(p, root, old.as_deref(), new)
            .into_iter()
            .collect())
    }

    /// Remove the entry. A file left with nothing but `version` is deleted, with each folder
    /// of its path that is left empty.
    pub fn remove(&self, root: &Path) -> Result<Vec<Change>> {
        let p = self.file(root);
        let mut out = Vec::new();
        let Some(old) = read_opt(&p)? else {
            return Ok(out);
        };
        let new = hooks_remove(&old, self.ours)
            .with_context(|| format!("{} is not valid JSON", p.display()))?;
        if new == old {
            return Ok(out);
        }
        let only_version = root_object(&new)?
            .members()
            .iter()
            .all(|m| m.key == "version");
        if only_version {
            std::fs::remove_file(&p)?;
            let mut dir = p.parent();
            while let Some(d) = dir.filter(|d| *d != root) {
                remove_if_empty(d);
                dir = d.parent();
            }
            out.push(Change::Removed(rel(&p, root)));
        } else {
            put(&p, root, Some(&old), &new, &mut out)?;
        }
        Ok(out)
    }
}
