//! `sj spec infer`: draft scenarios from existing tests.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::config::{Config, Runner};
use crate::{ids, spec, testscan, trace};

/// One scenario `sj spec infer` would draft from an untraced test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub id: String,
    pub heading: String,
    /// The test's file, relative to the project root.
    pub file: PathBuf,
    pub line: usize,
}

/// What a dry run would draft, and the tests under its paths it left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DryRun {
    /// The capability the drafts belong to.
    pub capability: String,
    pub drafts: Vec<Draft>,
    /// Tests that already carry a scenario ID.
    pub linked: usize,
    /// Untraced tests marked skip, todo or fixme.
    pub skipped: usize,
}

/// Draft scenarios for the untraced tests in suite files under `paths` (SPEC-048, SPEC-049,
/// SPEC-056). Paths are relative to `root` or absolute. Tests that carry a scenario ID and
/// skipped tests are left out; drafts are numbered after the highest number the capability's
/// prefix has used under `spec_root`. Errors when no untraced test is left, and writes nothing.
pub fn dry_run(
    root: &Path,
    config: &Config,
    spec_root: &Path,
    paths: &[PathBuf],
    capability: Option<&str>,
) -> Result<DryRun> {
    let scenarios = spec::load_scenarios(spec_root)?;
    let scan = testscan::scan_project(root, config)?;
    let targets: Vec<PathBuf> = paths.iter().map(|p| resolve(root, p)).collect();
    let targeted: Vec<testscan::TestFn> = scan
        .tests
        .into_iter()
        .filter(|t| {
            let file = resolve(root, &t.file);
            targets.iter().any(|p| file.starts_with(p))
        })
        .collect();
    let untraced = trace::untraced(&scenarios, &targeted);
    let linked = targeted.len() - untraced.len();
    let (skipped, selected): (Vec<_>, Vec<_>) = untraced.into_iter().partition(|t| t.skipped);
    if selected.is_empty() {
        let given: Vec<String> = paths.iter().map(|p| shown(p)).collect();
        bail!("no untraced tests under {}", given.join(" "));
    }
    let capability = match capability {
        Some(c) => c.to_string(),
        None => capability_from_folders(&selected)?,
    };
    ids::check_prefix_free(spec_root, &config.id_prefixes, &capability)?;
    let prefix = ids::prefix_for(&config.id_prefixes, &capability);
    if prefix.is_empty() {
        bail!(
            "capability `{capability}` has no letters to make a prefix; \
             map it in [ids.prefixes] in specjutsu.toml"
        );
    }
    let mut next = ids::highest_numbers(spec_root)?
        .get(&prefix)
        .copied()
        .unwrap_or(0);
    let drafts = selected
        .into_iter()
        .map(|t| {
            next += 1;
            Draft {
                id: format!("{prefix}-{next:03}"),
                heading: heading_from_test(&t.name, t.suite),
                file: t.file.clone(),
                line: t.line,
            }
        })
        .collect();
    Ok(DryRun {
        capability,
        drafts,
        linked,
        skipped: skipped.len(),
    })
}

/// Write the dry run's drafts as `changes/<change>/` under `spec_root` (SPEC-050, SPEC-053):
/// a proposal, one task per test file, and an ADDED delta with one `[keep]` requirement per
/// test file whose scenarios ask for the behaviour. Refuses, writing nothing, when `change`
/// is not kebab-case or the folder exists (SPEC-051).
pub fn write(spec_root: &Path, change: &str, run: &DryRun) -> Result<PathBuf> {
    if !super::is_kebab(change) {
        bail!("change name `{change}` must be kebab-case (lowercase letters, digits and hyphens)");
    }
    let dir = spec_root.join("changes").join(change);
    if change == "archive" || dir.exists() {
        bail!("change {change} already exists");
    }
    let mut files: Vec<(String, Vec<&Draft>)> = Vec::new();
    for d in &run.drafts {
        let file = shown(&d.file);
        match files.iter_mut().find(|(f, _)| *f == file) {
            Some((_, ds)) => ds.push(d),
            None => files.push((file, vec![d])),
        }
    }
    let cap = &run.capability;
    let mut spec = String::from("## ADDED Requirements\n");
    let mut tasks = String::from("# Tasks\n\n## 1. Keep the tested behaviour\n");
    for (i, (file, drafts)) in files.iter().enumerate() {
        spec.push_str(&format!(
            "\n### Requirement: {}\nThe system SHALL keep the behaviour that {file} tests.\n",
            file_words(file)
        ));
        for d in drafts {
            let at = format!("{file}:{}", d.line);
            spec.push_str(&format!(
                "\n#### Scenario: {} [keep] {}\n\
                 - **WHEN** [NEEDS CLARIFICATION: what the test sets up — see {at}]\n\
                 - **THEN** [NEEDS CLARIFICATION: what the test checks — see {at}]\n",
                d.id, d.heading
            ));
        }
        let ids: Vec<&str> = drafts.iter().map(|d| d.id.as_str()).collect();
        tasks.push_str(&format!(
            "- [ ] 1.{} Keep the behaviour that {file} tests ({})\n",
            i + 1,
            id_list(&ids)
        ));
    }
    let proposal = format!(
        "## Why\n\nExisting tests prove behaviour of {cap} that no scenario describes. \
         `sj spec infer` drafted these scenarios from them.\n\n\
         ## What Changes\n\n- Add `[keep]` scenarios for {cap}, one per existing test. Each \
         `[NEEDS CLARIFICATION]` marker must be replaced with the behaviour the test proves.\n\n\
         ## Capabilities\n\n### New Capabilities\n\n- `{cap}`\n\n### Modified Capabilities\n\n\
         None.\n\n## Impact\n\nSpecs only; the tests stay as they are.\n"
    );
    let cap_dir = dir.join("specs").join(cap);
    std::fs::create_dir_all(&cap_dir)
        .with_context(|| format!("cannot create {}", cap_dir.display()))?;
    for (path, text) in [
        (dir.join("proposal.md"), proposal),
        (dir.join("tasks.md"), tasks),
        (cap_dir.join("spec.md"), spec),
    ] {
        std::fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
    }
    Ok(dir)
}

/// A test file's name in words: the name before its first `.`, `_` and `-` as spaces, the
/// first letter upper-cased.
fn file_words(file: &str) -> String {
    let name = file.rsplit('/').next().unwrap_or(file);
    let stem = name.split('.').next().unwrap_or(name);
    upper_first(&join(stem.split(['_', '-'])))
}

/// IDs as a task names them: `A-001..003` when they run on without a gap, else `A-001, A-004`.
fn id_list(ids: &[&str]) -> String {
    let num = |id: &str| id.rsplit('-').next().and_then(|n| n.parse::<u32>().ok());
    let nums: Vec<Option<u32>> = ids.iter().map(|id| num(id)).collect();
    let contiguous = nums.windows(2).all(|w| match (w[0], w[1]) {
        (Some(a), Some(b)) => b == a + 1,
        _ => false,
    });
    match (ids.first(), ids.last()) {
        (Some(first), Some(last)) if ids.len() > 1 && contiguous => {
            format!("{first}..{}", last.rsplit('-').next().unwrap_or(last))
        }
        _ => ids.join(", "),
    }
}

/// Folders that hold tests rather than name a capability, compared case-insensitively.
const TEST_FOLDERS: [&str; 8] = [
    "tests",
    "test",
    "__tests__",
    "spec",
    "unit",
    "integration",
    "e2e",
    "src",
];

/// The capability a test file's folders name (SPEC-058, SPEC-061): the nearest folder above
/// `file` that is not a test folder, in kebab-case. `None` when there is no such folder.
pub fn capability_from_path(file: &Path) -> Option<String> {
    file.parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .filter_map(|p| p.file_name()?.to_str())
        .find(|n| !TEST_FOLDERS.iter().any(|t| t.eq_ignore_ascii_case(n)))
        .map(kebab_case)
}

/// The one capability the tests' folders name (SPEC-062, SPEC-063). Errors when a file has
/// none or the files name different capabilities.
fn capability_from_folders(tests: &[&testscan::TestFn]) -> Result<String> {
    let mut found: Vec<String> = Vec::new();
    for t in tests {
        let Some(cap) = capability_from_path(&t.file) else {
            bail!(
                "{} is in no capability folder; pass --capability to name the capability",
                shown(&t.file)
            );
        };
        if !found.contains(&cap) {
            found.push(cap);
        }
    }
    if found.len() > 1 {
        bail!(
            "the tests' folders name capabilities {}; pass --capability to pick one",
            found.join(", ")
        );
    }
    Ok(found.pop().unwrap_or_default())
}

/// `name` in kebab-case: lower-case words joined by `-`, split at `_`, spaces, `-`, `.` and
/// a lower-to-upper case change.
fn kebab_case(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in name.chars() {
        if c == '_' || c == ' ' || c == '-' || c == '.' {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
            prev_lower = false;
        } else {
            if c.is_uppercase() && prev_lower && !out.ends_with('-') {
                out.push('-');
            }
            prev_lower = c.is_lowercase() || c.is_ascii_digit();
            out.extend(c.to_lowercase());
        }
    }
    out.trim_end_matches('-').to_string()
}

/// A path as given, with `/` separators and no trailing separator.
fn shown(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    match s.trim_end_matches('/') {
        "" => s,
        t => t.to_string(),
    }
}

/// `path` joined to `root` when relative, canonical when it exists.
fn resolve(root: &Path, path: &Path) -> PathBuf {
    let joined = root.join(path);
    joined.canonicalize().unwrap_or(joined)
}

/// The scenario heading for a test named as the scanners report it (SPEC-052).
///
/// JS and TS titles (`Refunds > auto-approves under $100`) are joined with a space and keep
/// their own text. Rust and pytest names are identifiers: a leading `test` word is dropped
/// and `_` and `-` become spaces. A pytest `Class::func` (or `Outer::Inner::func`) becomes
/// each class name in order, with a leading `Test` word dropped by the same rule, followed
/// by the function's words. The first letter is upper-cased; when nothing is left the
/// heading is `Test`.
pub fn heading_from_test(name: &str, runner: Option<Runner>) -> String {
    let words = match runner {
        Some(Runner::Jest | Runner::Vitest | Runner::Playwright) => join(name.split(" > ")),
        _ => {
            let mut parts: Vec<&str> = name.split("::").collect();
            let func = parts.pop().unwrap_or_default();
            let classes = parts.into_iter().map(|c| identifier_words(c, "Test"));
            join(classes.chain([identifier_words(func, "test")]))
        }
    };
    if words.is_empty() {
        return "Test".to_string();
    }
    upper_first(&words)
}

/// The non-empty parts, trimmed and joined with a space.
fn join<S: AsRef<str>>(parts: impl Iterator<Item = S>) -> String {
    let parts: Vec<String> = parts
        .map(|p| p.as_ref().trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    parts.join(" ")
}

/// An identifier's words: a leading `word` dropped when it is a whole word (the whole name,
/// `word_`, or `word` before an upper-case letter), then `_` and `-` as spaces.
fn identifier_words(name: &str, word: &str) -> String {
    let rest = match name.strip_prefix(word) {
        Some(r) if r.is_empty() || r.starts_with(|c: char| c == '_' || c.is_uppercase()) => r,
        _ => name,
    };
    join(rest.split(['_', '-']))
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
