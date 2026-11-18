//! `openspec/config.yaml` as principles Markdown.
//!
//! Line-based, over the subset OpenSpec writes: a `context` block scalar, nested keys and
//! `- item` lists. Anything else falls back to the verbatim file in a fence.

const SOURCE: &str = "openspec/config.yaml";

/// The Markdown for `yaml`. `appended` is for an existing principles file: no title, and the
/// whole text sits under `## From openspec/config.yaml`.
pub fn to_principles(yaml: &str, appended: bool) -> String {
    let base = if appended { 3 } else { 2 };
    let body = match convert(yaml, base) {
        Some(blocks) => blocks,
        None => vec![format!(
            "```yaml\n{}\n```",
            yaml.replace("\r\n", "\n").trim_end()
        )],
    };
    let mut blocks = Vec::new();
    if appended {
        blocks.push(format!("## From {SOURCE}"));
    } else {
        blocks.push("# Principles".to_string());
        if body.first().is_some_and(|b| b.starts_with("```")) {
            blocks.push(format!("## From {SOURCE}"));
        }
    }
    blocks.extend(body);
    format!("{}\n", blocks.join("\n\n"))
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        s[1..s.len() - 1].replace("''", "'")
    } else {
        s.to_string()
    }
}

/// A plain `key` (no quotes, flow or anchors).
fn is_key(k: &str) -> bool {
    !k.is_empty()
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn title(key: &str) -> String {
    let mut c = key.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Markdown blocks, or `None` for a shape this converter doesn't know.
fn convert(yaml: &str, base: usize) -> Option<Vec<String>> {
    let text = yaml.replace("\r\n", "\n");
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks: Vec<String> = Vec::new();
    // Indents of the open keys; the depth of a key is how many are open above it.
    let mut open: Vec<usize> = Vec::new();
    let mut bullets: Vec<String> = Vec::new();
    let flush = |bullets: &mut Vec<String>, blocks: &mut Vec<String>| {
        if !bullets.is_empty() {
            blocks.push(bullets.join("\n"));
            bullets.clear();
        }
    };
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        i += 1;
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if line.starts_with('\t') {
            return None;
        }
        let ind = indent(line);
        if let Some(item) = t.strip_prefix("- ") {
            let item = item.trim();
            if item.starts_with(['[', '{', '&', '*', '|', '>']) || item.ends_with(':') {
                return None;
            }
            if let Some((k, _)) = item.split_once(": ") {
                if is_key(k) && !item.starts_with(['"', '\'']) {
                    return None;
                }
            }
            bullets.push(format!("- {}", unquote(item)));
            continue;
        }
        let (key, value) = t.split_once(':')?;
        if !is_key(key) {
            return None;
        }
        let value = value.trim();
        while open.last().is_some_and(|&o| o >= ind) {
            open.pop();
        }
        let depth = open.len();
        flush(&mut bullets, &mut blocks);
        if depth == 0 && key == "schema" {
            continue;
        }
        let heading = |blocks: &mut Vec<String>| {
            let name = if depth == 0 {
                title(key)
            } else {
                key.to_string()
            };
            blocks.push(format!("{} {name}", "#".repeat(base + depth)));
        };
        match value {
            "" => {
                heading(&mut blocks);
                open.push(ind);
            }
            "|" | "|-" | "|+" | ">" | ">-" | ">+" => {
                let mut body: Vec<&str> = Vec::new();
                let mut inner = None;
                while i < lines.len() {
                    let l = lines[i];
                    if l.trim().is_empty() {
                        body.push("");
                        i += 1;
                        continue;
                    }
                    let li = indent(l);
                    if li <= ind {
                        break;
                    }
                    let n = *inner.get_or_insert(li);
                    if li < n {
                        return None;
                    }
                    body.push(&l[n..]);
                    i += 1;
                }
                while body.last() == Some(&"") {
                    body.pop();
                }
                if !(depth == 0 && key == "context") {
                    heading(&mut blocks);
                }
                if !body.is_empty() {
                    blocks.push(body.join("\n"));
                }
            }
            v if v.starts_with(['[', '{', '&', '*', '!']) => return None,
            v => {
                if v.starts_with(['"', '\'']) && unquote(v) == v {
                    return None;
                }
                bullets.push(format!("- {key}: {}", unquote(v)));
            }
        }
    }
    flush(&mut bullets, &mut blocks);
    Some(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_shapes_are_kept_verbatim() {
        let yaml = "schema: spec-driven\nrules:\n  tasks: [a, b]\n";
        assert_eq!(
            to_principles(yaml, false),
            format!(
                "# Principles\n\n## From {SOURCE}\n\n```yaml\n{}\n```\n",
                yaml.trim_end()
            )
        );
    }

    #[test]
    fn appended_text_sits_one_level_down() {
        let yaml = "context: |\n  Hello.\nrules:\n  specs:\n    - One\n";
        assert_eq!(
            to_principles(yaml, true),
            format!("## From {SOURCE}\n\nHello.\n\n### Rules\n\n#### specs\n\n- One\n")
        );
    }
}
