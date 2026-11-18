//! PR-facing output: the gate as a markdown comment (CI-001..003).

use crate::gate::{GateReport, Row};

/// First line of every gate comment; the action finds the comment to update by it.
pub const GATE_MARKER: &str = "<!-- specjutsu:gate -->";

/// GitHub's comment limit is 65,536 characters; stay well under it.
pub const MAX_BYTES: usize = 60_000;

/// Room kept for the truncation note.
const FOOTER_ROOM: usize = 200;

/// The one-line fix hint for a row's evidence, or empty when it needs none.
pub fn hint(row: &Row) -> String {
    match row.evidence.as_str() {
        "untested" => format!("add a test that names {}", row.id),
        "green-on-base" => {
            "the test passes without your change; make it assert the new behaviour".to_string()
        }
        "failing" => "make the test pass at head".to_string(),
        "not-run" => {
            "the runner did not report this test; check it compiles and is not ignored".to_string()
        }
        "not-run-on-base" => {
            "the test did not run on base; run `sj gate` locally to see why".to_string()
        }
        "unverifiable" => {
            "move the test where it can run on base, or pass --allow-unverifiable".to_string()
        }
        _ => String::new(),
    }
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn table_row(row: &Row) -> String {
    let tests = if row.tests.is_empty() {
        "—".to_string()
    } else {
        row.tests
            .iter()
            .map(|t| format!("`{}`", cell(t)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "| {} | {} | {} | {} |",
        row.id,
        cell(&row.evidence),
        tests,
        cell(&hint(row))
    )
}

/// Failures that belong to no scenario row (unspecified changes, orphans, exemptions, …).
fn other_failures(report: &GateReport) -> Vec<String> {
    report
        .failures
        .iter()
        .filter(|f| {
            !report
                .rows
                .iter()
                .any(|r| f.split_whitespace().nth(1) == Some(&r.id))
        })
        .cloned()
        .collect()
}

/// Render the gate result as a PR comment body of at most [`MAX_BYTES`] bytes. `base` is the
/// ref the gate compared against, used in the truncation note.
pub fn render_markdown(report: &GateReport, base: &str) -> String {
    let icon = if report.pass { "✅" } else { "❌" };
    let mut out = format!("{GATE_MARKER}\n### {icon} {}\n", report.summary);

    // Items in order of importance; each is one line. Section headers are emitted lazily.
    enum Item {
        Row(String),
        Other(String),
        Warn(String),
        Exempt(String),
    }
    let mut rows: Vec<&Row> = report.rows.iter().collect();
    rows.sort_by_key(|r| !r.failed); // failures first, order otherwise kept
    let mut items: Vec<Item> = rows.iter().map(|r| Item::Row(table_row(r))).collect();
    items.extend(other_failures(report).into_iter().map(Item::Other));
    items.extend(report.warnings.iter().cloned().map(Item::Warn));
    items.extend(report.exemptions.iter().cloned().map(Item::Exempt));

    let budget = MAX_BYTES - FOOTER_ROOM;
    let mut section = "";
    let mut omitted = 0usize;
    for item in &items {
        let (name, header, line) = match item {
            Item::Row(l) => (
                "rows",
                "\n| Scenario | Evidence | Test | Fix |\n| --- | --- | --- | --- |\n",
                l.clone(),
            ),
            Item::Other(l) => (
                "other",
                "\n**Other problems**\n\n",
                format!("- {}", cell(l)),
            ),
            Item::Warn(l) => ("warn", "\n**Warnings**\n\n", format!("- {}", cell(l))),
            Item::Exempt(l) => ("exempt", "\n**Exemptions**\n\n", format!("- {}", cell(l))),
        };
        let head = if section == name { "" } else { header };
        if omitted > 0 || out.len() + head.len() + line.len() + 1 > budget {
            omitted += 1;
            continue;
        }
        out.push_str(head);
        section = name;
        out.push_str(&line);
        out.push('\n');
    }
    if omitted > 0 {
        let what = if omitted == 1 { "row" } else { "rows" };
        out.push_str(&format!(
            "\n_{omitted} more {what} not shown; run `sj gate --base {base}` locally for the full list._\n"
        ));
    }
    out
}
