//! The crate embeds copies of `skills/` and `adapters/claude/` so it can be published on its
//! own. The repository-root files are the source; this keeps the copies identical.

use std::path::Path;

#[test]
fn embedded_assets_match_the_repository_sources() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = crate_dir.join("../..");
    // A packaged crate has no repository around it; there is nothing to compare.
    if !repo.join("skills").is_dir() {
        return;
    }
    let assets = crate_dir.join("assets");
    let mut pairs = vec![
        (
            repo.join("adapters/claude/CLAUDE.block.md"),
            assets.join("claude/CLAUDE.block.md"),
        ),
        (
            repo.join("adapters/agents-md/AGENTS.block.md"),
            assets.join("agents-md/AGENTS.block.md"),
        ),
    ];
    let mut names: Vec<String> = std::fs::read_dir(repo.join("skills"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("SKILL.md").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        // Only the `sj-*` skills are embedded; the setup skill `specjutsu` is not.
        .filter(|n| n.starts_with("sj-"))
        .collect();
    names.sort();
    for name in &names {
        // Every Markdown file in the skill folder: SKILL.md and its reference files, and any
        // file the crate embeds that the repository no longer has.
        let mut files: Vec<String> = Vec::new();
        for dir in [
            repo.join("skills").join(name),
            assets.join("skills").join(name),
        ] {
            for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let f = e.file_name().to_string_lossy().into_owned();
                if f.ends_with(".md") && !files.contains(&f) {
                    files.push(f);
                }
            }
        }
        files.sort();
        for f in files {
            pairs.push((
                repo.join("skills").join(name).join(&f),
                assets.join("skills").join(name).join(&f),
            ));
        }
    }
    let stale: Vec<String> = pairs
        .iter()
        .filter(|(src, copy)| std::fs::read(src).ok() != std::fs::read(copy).ok())
        .map(|(_, copy)| copy.strip_prefix(crate_dir).unwrap().display().to_string())
        .collect();
    assert!(
        stale.is_empty(),
        "stale embedded assets: {stale:?}\nrun ./scripts/sync-adapter-assets.sh"
    );
}

#[test]
fn every_embedded_reference_file_is_installed() {
    let skills = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills");
    let mut on_disk: Vec<(String, String)> = Vec::new();
    for e in std::fs::read_dir(&skills).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        for f in std::fs::read_dir(e.path()).unwrap().flatten() {
            let file = f.file_name().to_string_lossy().into_owned();
            if file != "SKILL.md" && file.ends_with(".md") {
                on_disk.push((name.clone(), file));
            }
        }
    }
    on_disk.sort();
    let mut listed: Vec<(String, String)> = sj_adapters::claude::REFERENCES
        .iter()
        .map(|(s, f, _)| (s.to_string(), f.to_string()))
        .collect();
    listed.sort();
    assert_eq!(
        on_disk, listed,
        "claude::REFERENCES must list every embedded reference"
    );
}
