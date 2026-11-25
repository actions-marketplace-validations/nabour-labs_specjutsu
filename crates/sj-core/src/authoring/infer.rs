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
    /// The test's suite runner, which picks the `spec:` comment syntax.
    pub suite: Option<Runner>,
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
    if let Some(c) = capability {
        if !super::is_kebab(c) {
            bail!("capability `{c}` must be kebab-case (lowercase letters, digits and hyphens)");
        }
    }
    let scenarios = spec::load_scenarios(spec_root)?;
    let scan = testscan::scan_project(root, config)?;
    let carried: Vec<String> = scan
        .tests
        .iter()
        .flat_map(|t| t.name_ids.iter().chain(&t.comment_ids).cloned())
        .collect();
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
    let (skipped, mut selected): (Vec<_>, Vec<_>) = untraced.into_iter().partition(|t| t.skipped);
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
    let number = |id: &str| -> Option<u64> {
        let (p, n) = id.rsplit_once('-')?;
        (p == prefix).then(|| n.parse().ok()).flatten()
    };
    let in_specs = ids::highest_numbers(spec_root)?.get(&prefix).copied();
    let mut next = carried
        .iter()
        .filter_map(|id| number(id))
        .chain(in_specs)
        .max()
        .unwrap_or(0);
    // Tests whose names already carry an ID with this prefix are numbered last, so the
    // others keep the lowest free numbers (SPEC-068).
    selected.sort_by_key(|t| t.name_ids.iter().any(|id| number(id).is_some()));
    let drafts = selected
        .into_iter()
        .map(|t| {
            next += 1;
            Draft {
                id: format!("{prefix}-{next:03}"),
                heading: heading_from_test(&t.name, t.suite),
                file: t.file.clone(),
                line: t.line,
                suite: t.suite,
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
/// is not kebab-case or the folder exists (SPEC-051). Then links each test to its draft with a
/// `spec:` comment (SPEC-054); when that fails, the change folder is removed again. Returns the
/// folder and the drafts with the lines their tests are on once the comments are in (SPEC-067).
pub fn write(
    root: &Path,
    spec_root: &Path,
    change: &str,
    run: &DryRun,
) -> Result<(PathBuf, Vec<Draft>)> {
    if !super::is_kebab(change) {
        bail!("change name `{change}` must be kebab-case (lowercase letters, digits and hyphens)");
    }
    let dir = spec_root.join("changes").join(change);
    if change == "archive" || dir.exists() {
        bail!("change {change} already exists");
    }
    let (edits, lines) = plan_links(root, &run.drafts)?;
    let drafts: Vec<Draft> = run
        .drafts
        .iter()
        .zip(lines)
        .map(|(d, line)| Draft { line, ..d.clone() })
        .collect();
    let mut files: Vec<(String, Vec<&Draft>)> = Vec::new();
    for d in &drafts {
        let file = shown(&d.file);
        match files.iter_mut().find(|(f, _)| *f == file) {
            Some((_, ds)) => ds.push(d),
            None => files.push((file, vec![d])),
        }
    }
    let cap = &run.capability;
    let target = spec_root.join("specs").join(cap).join("spec.md");
    // The capability's spec and every open change's delta for it (SPEC-069).
    let mut sources = vec![target.clone()];
    for c in super::open_changes(spec_root)? {
        sources.push(c.dir.join("specs").join(cap).join("spec.md"));
    }
    let mut existing = Vec::new();
    for s in &sources {
        existing.extend(existing_names(s)?);
    }
    let names = requirement_names(&files, &existing)?;
    let mut spec = String::from("## ADDED Requirements\n");
    let mut tasks = String::from("# Tasks\n\n## 1. Keep the tested behaviour\n");
    for (i, ((file, drafts), name)) in files.iter().zip(&names).enumerate() {
        spec.push_str(&format!(
            "\n### Requirement: {name}\nThe system SHALL keep the behaviour that {file} tests.\n"
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
    let (new, modified) = if sources.iter().any(|s| s.is_file()) {
        ("None.".to_string(), format!("- `{cap}`"))
    } else {
        (format!("- `{cap}`"), "None.".to_string())
    };
    let proposal = format!(
        "## Why\n\nExisting tests prove behaviour of {cap} that no scenario describes. \
         `sj spec infer` drafted these scenarios from them.\n\n\
         ## What Changes\n\n- Add `[keep]` scenarios for {cap}, one per existing test. Each \
         `[NEEDS CLARIFICATION]` marker must be replaced with the behaviour the test proves.\n\n\
         ## Capabilities\n\n### New Capabilities\n\n{new}\n\n### Modified Capabilities\n\n\
         {modified}\n\n## Impact\n\nSpecs, and one `spec:` comment line above each test that links it.\n"
    );
    let written = (|| -> Result<()> {
        let cap_dir = dir.join("specs").join(cap);
        std::fs::create_dir_all(&cap_dir)
            .with_context(|| format!("cannot create {}", cap_dir.display()))?;
        for (path, text) in [
            (dir.join("proposal.md"), proposal),
            (dir.join("tasks.md"), tasks),
            (cap_dir.join("spec.md"), spec),
        ] {
            std::fs::write(&path, text)
                .with_context(|| format!("cannot write {}", path.display()))?;
        }
        Ok(())
    })()
    .and_then(|()| apply_links(&edits));
    if let Err(e) = written {
        // The folder did not exist before this run, so removing it leaves no partial change.
        let _ = std::fs::remove_dir_all(&dir);
        return Err(e);
    }
    Ok((dir, drafts))
}

/// A test file's path, its old text and its text with the `spec:` comments in.
type Edit = (PathBuf, String, String);

/// Work out the `spec: <ID>` comment above each draft's test (SPEC-054), and each draft's line
/// once the comments are in: its line plus the comments inserted at or above it (SPEC-067).
fn plan_links(root: &Path, drafts: &[Draft]) -> Result<(Vec<Edit>, Vec<usize>)> {
    let mut files: Vec<(PathBuf, Vec<usize>)> = Vec::new();
    for (i, d) in drafts.iter().enumerate() {
        let path = root.join(&d.file);
        match files.iter_mut().find(|(f, _)| *f == path) {
            Some((_, ds)) => ds.push(i),
            None => files.push((path, vec![i])),
        }
    }
    let mut lines: Vec<usize> = drafts.iter().map(|d| d.line).collect();
    let mut edits: Vec<Edit> = Vec::new();
    for (path, ds) in files {
        let old = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        let python = ds.iter().any(|&i| is_python(&drafts[i]));
        let mark = if python { "#" } else { "//" };
        let text_lines: Vec<&str> = old
            .split_inclusive('\n')
            .map(|s| s.trim_end_matches('\n').trim_end_matches('\r'))
            .collect();
        let comments: Vec<(usize, String)> = ds
            .iter()
            .map(|&i| {
                let d = &drafts[i];
                let at = first_header_line(&text_lines, d.line.saturating_sub(1), python);
                (at, format!("{mark} spec: {}", d.id))
            })
            .collect();
        for &i in &ds {
            let decl = drafts[i].line.saturating_sub(1);
            lines[i] += comments.iter().filter(|(at, _)| *at <= decl).count();
        }
        let new = with_spec_comments(&old, &comments);
        edits.push((path, old, new));
    }
    Ok((edits, lines))
}

/// Write each test file's new text; if a write fails, the files already written get their old
/// text back.
fn apply_links(edits: &[Edit]) -> Result<()> {
    for (i, (path, _, new)) in edits.iter().enumerate() {
        if let Err(e) = std::fs::write(path, new) {
            for (done, old, _) in &edits[..i] {
                let _ = std::fs::write(done, old);
            }
            return Err(e).with_context(|| format!("cannot write {}", path.display()));
        }
    }
    Ok(())
}

/// Whether a draft's test is Python: its suite is pytest, or it has no suite and a `.py` file.
fn is_python(d: &Draft) -> bool {
    match d.suite {
        Some(r) => r == Runner::Pytest,
        None => d.file.extension().is_some_and(|e| e == "py"),
    }
}

/// `content` with each comment on its own line above its 0-based line, at that line's
/// indentation. Each inserted line ends like the line below it, so line endings, a missing final
/// newline and a leading BOM are kept.
fn with_spec_comments(content: &str, comments: &[(usize, String)]) -> String {
    let segments: Vec<&str> = content.split_inclusive('\n').collect();
    let mut at: Vec<(usize, &str)> = comments.iter().map(|(k, c)| (*k, c.as_str())).collect();
    at.sort_by_key(|(k, _)| *k);
    let mut out = String::with_capacity(content.len() + 32 * at.len());
    let mut next = at.iter().peekable();
    for (k, seg) in segments.iter().enumerate() {
        let (bom, seg) = match seg.strip_prefix('\u{feff}') {
            Some(rest) if k == 0 => ("\u{feff}", rest),
            _ => ("", *seg),
        };
        out.push_str(bom);
        let eol = if seg.ends_with("\r\n") {
            "\r\n"
        } else if seg.ends_with('\n') {
            "\n"
        } else if k > 0 && segments[k - 1].ends_with("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let indent = &seg[..seg.len() - seg.trim_start_matches([' ', '\t']).len()];
        while let Some((_, c)) = next.next_if(|(line, _)| *line == k) {
            out.push_str(indent);
            out.push_str(c);
            out.push_str(eol);
        }
        out.push_str(seg);
    }
    out
}

/// The 0-based line of the first attribute (`#[...]`) or decorator (`@...`, possibly spanning
/// lines in Python) above the declaration on 0-based line `decl`, or `decl` itself. Comment
/// lines between attributes or decorators are walked past; comments above the first stay above.
fn first_header_line(lines: &[&str], decl: usize, python: bool) -> usize {
    let comment = |l: &str| {
        let t = l.trim_start();
        if python {
            t.starts_with('#')
        } else {
            t.starts_with("//")
        }
    };
    let mut k = decl.min(lines.len());
    loop {
        let mut m = k;
        while m > 0 && comment(lines[m - 1]) {
            m -= 1;
        }
        match m
            .checked_sub(1)
            .and_then(|end| header_start(lines, end, python))
        {
            Some(start) => k = start,
            None => return k,
        }
    }
}

/// The 0-based first line of the attribute or decorator that ends on 0-based line `end`:
/// `end` itself for a line that is a whole `#[...]` attribute (not Python; an attribute that
/// shares its line with an item belongs to that item) or an `@...` decorator, or the `@` line
/// of a Python decorator whose brackets stay open until exactly `end`.
fn header_start(lines: &[&str], end: usize, python: bool) -> Option<usize> {
    let decorator = |l: &str| l.trim_start().starts_with('@');
    let line = lines[end];
    let t = line.trim();
    let rust_attr = t.starts_with("#[") && t.ends_with(']') && bracket_balance(t) == 0;
    if (!python && rust_attr) || decorator(line) {
        return Some(end);
    }
    if !python || line.trim().is_empty() {
        return None;
    }
    let start = (0..end).rev().find(|&j| decorator(lines[j]))?;
    let mut depth = 0;
    let closes_here = (start..=end).all(|i| {
        depth += bracket_balance(lines[i]);
        if i == end {
            depth == 0
        } else {
            depth > 0
        }
    });
    closes_here.then_some(start)
}

/// Opening minus closing brackets on a line.
fn bracket_balance(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            '(' | '[' | '{' => 1,
            ')' | ']' | '}' => -1,
            _ => 0,
        })
        .sum()
}

/// The requirement names in `target`, as comparison keys; none when it does not exist.
fn existing_names(target: &Path) -> Result<Vec<String>> {
    if !target.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(target)
        .with_context(|| format!("cannot read {}", target.display()))?;
    Ok(text
        .lines()
        .filter_map(super::blocks::requirement_name)
        .map(|n| name_key(&n))
        .collect())
}

/// A requirement name as archive compares it: lower-case, single spaces.
fn name_key(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// One requirement name per test file (SPEC-065, SPEC-066): the file's stem in words, or its
/// path in words when the stem name is empty, used by another file in the delta, or already
/// a requirement of the capability. Errors when the path name is taken too.
fn requirement_names<T>(files: &[(String, T)], existing: &[String]) -> Result<Vec<String>> {
    let stems: Vec<String> = files.iter().map(|(f, _)| file_words(f)).collect();
    let mut names: Vec<String> = Vec::new();
    for (i, (file, _)) in files.iter().enumerate() {
        let key = name_key(&stems[i]);
        let clash = key.is_empty()
            || existing.contains(&key)
            || stems
                .iter()
                .enumerate()
                .any(|(j, s)| j != i && name_key(s) == key);
        let name = if clash {
            path_words(file)
        } else {
            stems[i].clone()
        };
        let key = name_key(&name);
        if key.is_empty() || existing.contains(&key) || names.iter().any(|n| name_key(n) == key) {
            bail!(
                "requirement name `{name}` for {file} is already taken; \
                 rename the test file or draft it in another run"
            );
        }
        names.push(name);
    }
    Ok(names)
}

/// A test file's root-relative path in words, extensions included: `/`, `_`, `-` and `.` as
/// spaces, the first letter upper-cased.
fn path_words(file: &str) -> String {
    upper_first(&join(file.split(['/', '_', '-', '.'])))
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
