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
        .collect();
    names.sort();
    for name in &names {
        pairs.push((
            repo.join("skills").join(name).join("SKILL.md"),
            assets.join("skills").join(name).join("SKILL.md"),
        ));
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
