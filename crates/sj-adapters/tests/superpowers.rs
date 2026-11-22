//! Superpowers removal from `.claude/settings.json`, and the adapter's dry run
//! (specjutsu/changes/tool-migration).

use sj_adapters::claude::{self, Change};
use sj_adapters::superpowers::settings_remove;

const HOOKS: &str = r#"  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "node \"$CLAUDE_PROJECT_DIR/scripts/clean-merged-branches.mjs\"",
            "async": true,
            "timeout": 120
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Edit|Write|NotebookEdit|Bash",
        "hooks": [
          {
            "type": "command",
            "command": "node \"$CLAUDE_PROJECT_DIR/scripts/hooks/require-worktree.mjs\""
          }
        ]
      }
    ]
  }"#;

fn with_superpowers() -> String {
    format!(
        r#"{{
  "extraKnownMarketplaces": {{
    "superpowers-marketplace": {{
      "source": {{
        "source": "github",
        "repo": "obra/superpowers-marketplace"
      }}
    }}
  }},
  "enabledPlugins": {{
    "superpowers@superpowers-marketplace": true
  }},
  "permissions": {{
    "deny": [
      "Skill(superpowers:brainstorming)",
      "Skill(superpowers:writing-plans)",
      "Skill(superpowers:executing-plans)",
      "Edit(/docs/superpowers/**)"
    ]
  }},
{HOOKS}
}}
"#
    )
}

#[test]
fn mig_006_project_hooks_survive_superpowers_removal() {
    let out = settings_remove(&with_superpowers()).unwrap();
    assert_eq!(out, format!("{{\n{HOOKS}\n}}\n"));
}

#[test]
fn mig_006_other_plugins_and_rules_are_kept() {
    let text = r#"{
  "enabledPlugins": {
    "superpowers@superpowers-marketplace": true,
    "other@market": true
  },
  "extraKnownMarketplaces": {
    "superpowers-marketplace": {"source": {"source": "github", "repo": "obra/superpowers-marketplace"}},
    "market": {"source": {"source": "github", "repo": "acme/market"}}
  },
  "permissions": {
    "allow": ["Bash(ls)", "Skill(superpowers:test-driven-development)"],
    "deny": ["Read(./.env*)"]
  }
}
"#;
    let out = settings_remove(text).unwrap();
    assert_eq!(
        out,
        r#"{
  "enabledPlugins": {
    "other@market": true
  },
  "extraKnownMarketplaces": {
    "market": {"source": {"source": "github", "repo": "acme/market"}}
  },
  "permissions": {
    "allow": ["Bash(ls)"],
    "deny": ["Read(./.env*)"]
  }
}
"#
    );
}

#[test]
fn mig_006_settings_without_superpowers_are_unchanged() {
    let text = format!("{{\n  \"permissions\": {{\"deny\": [\"Read(./.env*)\"]}},\n{HOOKS}\n}}\n");
    assert_eq!(settings_remove(&text).unwrap(), text);
}

#[test]
fn mig_001_adapter_plan_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let planned = claude::plan_install(dir.path(), false).unwrap();
    assert!(planned.contains(&Change::Created("CLAUDE.md".into())));
    assert!(planned.contains(&Change::Created(".claude/settings.json".into())));
    assert_eq!(
        planned.len(),
        2 + claude::SKILLS.len() + claude::REFERENCES.len()
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    assert_eq!(claude::install(dir.path(), false).unwrap(), planned);
}
