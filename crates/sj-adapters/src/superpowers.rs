//! Removing the Superpowers plugin from `.claude/settings.json`, keeping every other byte.

use anyhow::{Context, Result};

use crate::claude::{remove_element, remove_member, root_object};
use crate::jsonedit::{self, Node};

/// The plugin entry `superpowers@<marketplace>` and the marketplace it names.
fn is_plugin(key: &str) -> bool {
    key.starts_with("superpowers@")
}

/// A permission rule naming a Superpowers skill or its plans folder.
fn is_rule(rule: &Node) -> bool {
    rule.as_str()
        .is_some_and(|r| r.contains("superpowers:") || r.contains("docs/superpowers/"))
}

/// One edit, or `None` when nothing Superpowers is left. Each edit removes one member or
/// element, taking its parent with it when it was the parent's last.
fn next_edit(t: &str, marketplaces: &[String]) -> Result<Option<String>> {
    let root = root_object(t)?;
    if let Some(p) = root.get("enabledPlugins").filter(|p| p.value.is_object()) {
        if let Some(m) = p.value.members().iter().find(|m| is_plugin(&m.key)) {
            return Ok(Some(if p.value.members().len() > 1 {
                remove_member(t, &p.value, &m.key)
            } else {
                remove_member(t, &root, "enabledPlugins")
            }));
        }
    }
    if let Some(k) = root
        .get("extraKnownMarketplaces")
        .filter(|k| k.value.is_object())
    {
        let found = k
            .value
            .members()
            .iter()
            .find(|m| marketplaces.contains(&m.key) || m.key.contains("superpowers"));
        if let Some(m) = found {
            return Ok(Some(if k.value.members().len() > 1 {
                remove_member(t, &k.value, &m.key)
            } else {
                remove_member(t, &root, "extraKnownMarketplaces")
            }));
        }
    }
    if let Some(perm) = root.get("permissions").filter(|p| p.value.is_object()) {
        for list in perm.value.members() {
            let Some(i) = list.value.elements().iter().position(is_rule) else {
                continue;
            };
            return Ok(Some(if list.value.elements().len() > 1 {
                remove_element(t, &list.value, i)
            } else if perm.value.members().len() > 1 {
                remove_member(t, &perm.value, &list.key)
            } else {
                remove_member(t, &root, "permissions")
            }));
        }
    }
    Ok(None)
}

/// The settings text without the Superpowers plugin, its marketplace and the permission rules
/// naming `superpowers:` skills or `docs/superpowers/`. Containers left empty are removed.
pub fn settings_remove(text: &str) -> Result<String> {
    let root = root_object(text)?;
    let marketplaces: Vec<String> = root
        .get("enabledPlugins")
        .map(|p| {
            p.value
                .members()
                .iter()
                .filter(|m| is_plugin(&m.key))
                .filter_map(|m| m.key.split_once('@').map(|(_, mk)| mk.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let mut t = text.to_string();
    while let Some(next) = next_edit(&t, &marketplaces)? {
        t = next;
    }
    jsonedit::parse(&t).context("internal error: the edited settings are not valid JSON")?;
    Ok(t)
}

/// Whether the settings text names Superpowers at all.
pub fn is_present(text: &str) -> bool {
    settings_remove(text).is_ok_and(|t| t != text)
}
