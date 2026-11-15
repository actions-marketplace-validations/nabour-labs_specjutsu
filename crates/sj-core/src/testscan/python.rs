//! pytest test discovery: `test*` functions at module level or in `Test*` classes, linked by
//! name, `@pytest.mark.spec(…)` markers and `# spec:` comments.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::{ids_in_name, TestFn};

struct Block {
    indent: usize,
    /// `Some(name)` for a class, `None` for a function.
    class: Option<String>,
    collectable: bool,
    ids: Vec<String>,
    skipped: bool,
}

/// What the decorators and comments above a `def`/`class` contributed.
#[derive(Default)]
struct Pending {
    ids: Vec<String>,
    skipped: bool,
}

/// Find tests in one Python file.
pub fn scan_python(file: &Path, content: &str) -> Vec<TestFn> {
    static DEF: OnceLock<Regex> = OnceLock::new();
    static CLASS: OnceLock<Regex> = OnceLock::new();
    static SPEC: OnceLock<Regex> = OnceLock::new();
    static MARK: OnceLock<Regex> = OnceLock::new();
    static QUOTED_ID: OnceLock<Regex> = OnceLock::new();
    static SKIP: OnceLock<Regex> = OnceLock::new();
    let def_re = DEF
        .get_or_init(|| Regex::new(r"^(?:async\s+)?def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());
    let class_re = CLASS.get_or_init(|| Regex::new(r"^class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
    let spec_re = SPEC.get_or_init(|| {
        Regex::new(r"^#+\s*spec:\s*([A-Z][A-Z0-9]*-[0-9]+(?:\s*,\s*[A-Z][A-Z0-9]*-[0-9]+)*)")
            .unwrap()
    });
    let mark_re = MARK.get_or_init(|| Regex::new(r"\bmark\.spec\s*\(([^)]*)\)").unwrap());
    let quoted_id =
        QUOTED_ID.get_or_init(|| Regex::new(r#"["']([A-Z][A-Z0-9]*-[0-9]+)["']"#).unwrap());
    let skip_re = SKIP.get_or_init(|| Regex::new(r"\bmark\.(?:skip|skipif|xfail)\b").unwrap());

    let mut out = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut pending = Pending::default();
    // An open decorator spanning lines: its text so far and paren balance.
    let mut decorator: Option<(String, i32)> = None;
    let mut docstring: Option<&str> = None;

    for (i, raw) in content.lines().enumerate() {
        let t = raw.trim();
        if let Some(q) = docstring {
            if t.contains(q) {
                docstring = None;
            }
            continue;
        }
        if let Some((text, depth)) = decorator.as_mut() {
            text.push_str(t);
            *depth += balance(t);
            if *depth <= 0 {
                let (text, _) = decorator.take().unwrap_or_default();
                apply_decorator(&text, &mut pending, mark_re, quoted_id, skip_re);
            }
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            if let Some(c) = spec_re.captures(t) {
                pending
                    .ids
                    .extend(c[1].split(',').map(|s| s.trim().to_string()));
            }
            continue;
        }

        let indent = indent_of(raw);
        while blocks.last().is_some_and(|b| indent <= b.indent) {
            blocks.pop();
        }

        if let Some(rest) = t.strip_prefix('@') {
            let depth = balance(rest);
            if depth > 0 {
                decorator = Some((rest.to_string(), depth));
            } else {
                apply_decorator(rest, &mut pending, mark_re, quoted_id, skip_re);
            }
            continue;
        }

        let in_test_scope = blocks.iter().all(|b| b.collectable);
        if let Some(c) = class_re.captures(t) {
            let p = std::mem::take(&mut pending);
            let name = c[1].to_string();
            blocks.push(Block {
                indent,
                collectable: in_test_scope && name.starts_with("Test"),
                class: Some(name),
                ids: p.ids,
                skipped: p.skipped,
            });
            continue;
        }
        if let Some(c) = def_re.captures(t) {
            let p = std::mem::take(&mut pending);
            let func = &c[1];
            if in_test_scope && func.starts_with("test") {
                let classes: Vec<&Block> = blocks.iter().collect();
                let mut parts: Vec<&str> =
                    classes.iter().filter_map(|b| b.class.as_deref()).collect();
                parts.push(func);
                let mut comment_ids: Vec<String> =
                    classes.iter().flat_map(|b| b.ids.clone()).collect();
                comment_ids.extend(p.ids);
                dedup(&mut comment_ids);
                let mut name_ids: Vec<String> = parts.iter().flat_map(|s| ids_in_name(s)).collect();
                dedup(&mut name_ids);
                out.push(TestFn {
                    name: parts.join("::"),
                    file: file.to_path_buf(),
                    line: i + 1,
                    comment_ids,
                    name_ids,
                    skipped: p.skipped || classes.iter().any(|b| b.skipped),
                    ..TestFn::default()
                });
            }
            blocks.push(Block {
                indent,
                class: None,
                collectable: false,
                ids: Vec::new(),
                skipped: false,
            });
            continue;
        }

        pending = Pending::default();
        docstring = opens_docstring(t);
    }
    out
}

fn apply_decorator(
    text: &str,
    pending: &mut Pending,
    mark_re: &Regex,
    quoted_id: &Regex,
    skip_re: &Regex,
) {
    for m in mark_re.captures_iter(text) {
        pending
            .ids
            .extend(quoted_id.captures_iter(&m[1]).map(|c| c[1].to_string()));
    }
    if skip_re.is_match(text) {
        pending.skipped = true;
    }
}

/// The quote that opens a docstring left unclosed on this line.
fn opens_docstring(t: &str) -> Option<&'static str> {
    ["\"\"\"", "'''"]
        .into_iter()
        .find(|q| t.matches(q).count() % 2 == 1)
}

/// Bracket balance of one line, ignoring brackets in string literals and after `#`.
fn balance(s: &str) -> i32 {
    let mut depth = 0;
    let mut quote: Option<char> = None;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(_), '\\') => {
                chars.next();
            }
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '#') => break,
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => depth -= 1,
            _ => {}
        }
    }
    depth
}

fn indent_of(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn dedup(v: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    v.retain(|x| seen.insert(x.clone()));
}
