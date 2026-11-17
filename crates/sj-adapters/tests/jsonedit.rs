//! Text edits to `.claude/settings.json` (openspec/changes/claude-adapter).

use sj_adapters::claude::{settings_install, settings_remove};

fn commands(text: &str) -> Vec<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(text).unwrap();
    let mut out = Vec::new();
    if let Some(hooks) = v.get("hooks").and_then(|h| h.as_object()) {
        for (event, list) in hooks {
            for entry in list.as_array().unwrap() {
                for h in entry["hooks"].as_array().unwrap() {
                    out.push((event.clone(), h["command"].as_str().unwrap().to_string()));
                }
            }
        }
    }
    out
}

fn roundtrip(original: &str) -> String {
    let installed = settings_install(Some(original)).unwrap();
    assert_eq!(
        settings_remove(&installed).unwrap(),
        original,
        "installed:\n{installed}"
    );
    installed
}

#[test]
fn adapt_001_a_missing_file_gets_both_hooks() {
    let text = settings_install(None).unwrap();
    let got = commands(&text);
    assert!(got.contains(&("PreToolUse".into(), "sj hook claude pre-edit".into())));
    assert!(got.contains(&("SessionStart".into(), "sj hook claude session-start".into())));
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        v["hooks"]["PreToolUse"][0]["matcher"],
        "Edit|Write|MultiEdit|NotebookEdit"
    );
    let removed = settings_remove(&text).unwrap();
    assert_eq!(removed.trim(), "{}");
}

#[test]
fn adapt_001_existing_hooks_come_first_and_survive() {
    let original = "{\n\t\"hooks\": {\n\t\t\"PreToolUse\": [\n\t\t\t{ \"matcher\": \"Edit\", \"hooks\": [{ \"type\": \"command\", \"command\": \"node a.mjs\" }] }\n\t\t]\n\t},\n\t\"model\": \"opus\"\n}\n";
    let installed = roundtrip(original);
    assert_eq!(
        commands(&installed),
        vec![
            ("PreToolUse".to_string(), "node a.mjs".to_string()),
            (
                "PreToolUse".to_string(),
                "sj hook claude pre-edit".to_string()
            ),
            (
                "SessionStart".to_string(),
                "sj hook claude session-start".to_string()
            ),
        ]
    );
    // Our entry takes the indentation of the one before it.
    assert!(
        installed.contains(",\n\t\t\t{\"matcher\": \"Edit|Write|MultiEdit|NotebookEdit\""),
        "{installed}"
    );
}

#[test]
fn adapt_010_empty_minified_and_hookless_files_roundtrip() {
    for original in [
        "{}",
        "{}\n",
        "{\"permissions\":{\"deny\":[\"Bash(rm:*)\"]}}",
        "{\n  \"permissions\": {\n    \"deny\": []\n  }\n}\n",
    ] {
        let installed = roundtrip(original);
        assert_eq!(commands(&installed).len(), 2, "{original} → {installed}");
    }
}

#[test]
fn adapt_003_install_is_idempotent() {
    let once = settings_install(Some("{\n  \"model\": \"opus\"\n}\n")).unwrap();
    assert_eq!(settings_install(Some(&once)).unwrap(), once);
}

#[test]
fn adapt_010_invalid_json_is_an_error() {
    assert!(settings_install(Some("{\"hooks\": [")).is_err());
    assert!(settings_remove("not json").is_err());
}
