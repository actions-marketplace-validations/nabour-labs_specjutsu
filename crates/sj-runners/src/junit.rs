//! JUnit XML reports, as written by vitest, Jest (jest-junit), Playwright and pytest.

use anyhow::{bail, Context, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseOutcome {
    Passed,
    /// A `<failure>` or an `<error>`.
    Failed,
    Skipped,
}

/// One `<testcase>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub classname: String,
    pub name: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub outcome: CaseOutcome,
    /// The failure's `message` attribute followed by its text; empty unless failed.
    pub message: String,
}

/// Every test case in a report. A report that is truncated, empty or has no `<testsuite>` is an
/// error, so a runner that died mid-write is never read as zero tests.
pub fn parse(xml: &str) -> Result<Vec<Case>> {
    let mut reader = Reader::from_str(xml);
    let mut cases = Vec::new();
    let mut depth = 0usize;
    let mut saw_suite = false;
    let mut current: Option<Case> = None;
    // Inside a <failure>/<error>: collect its text.
    let mut in_failure = false;

    loop {
        let event = reader
            .read_event()
            .with_context(|| format!("invalid JUnit XML at byte {}", reader.buffer_position()))?;
        match event {
            Event::Start(e) => {
                depth += 1;
                match e.local_name().as_ref() {
                    b"testsuite" | b"testsuites" => saw_suite = true,
                    b"testcase" => current = Some(start_case(&e)?),
                    b"failure" | b"error" => {
                        if let Some(c) = current.as_mut() {
                            fail(c, &e)?;
                            in_failure = true;
                        }
                    }
                    b"skipped" => {
                        if let Some(c) = current.as_mut() {
                            c.outcome = CaseOutcome::Skipped;
                        }
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => match e.local_name().as_ref() {
                b"testsuite" | b"testsuites" => saw_suite = true,
                b"testcase" => cases.push(start_case(&e)?),
                b"failure" | b"error" => {
                    if let Some(c) = current.as_mut() {
                        fail(c, &e)?;
                    }
                }
                b"skipped" => {
                    if let Some(c) = current.as_mut() {
                        if c.outcome != CaseOutcome::Failed {
                            c.outcome = CaseOutcome::Skipped;
                        }
                    }
                }
                _ => {}
            },
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                match e.local_name().as_ref() {
                    b"testcase" => {
                        if let Some(c) = current.take() {
                            cases.push(c);
                        }
                    }
                    b"failure" | b"error" => in_failure = false,
                    _ => {}
                }
            }
            Event::Text(t) if in_failure => {
                let text = t.unescape().context("invalid text in JUnit XML")?;
                push_text(current.as_mut(), &text);
            }
            Event::CData(t) if in_failure => {
                let text = String::from_utf8_lossy(&t.into_inner()).into_owned();
                push_text(current.as_mut(), &text);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if depth != 0 {
        bail!("truncated JUnit XML ({depth} unclosed elements)");
    }
    if !saw_suite {
        bail!("not a JUnit report: no <testsuite> element");
    }
    for c in &mut cases {
        c.message = c.message.trim().to_string();
    }
    Ok(cases)
}

fn start_case(e: &BytesStart) -> Result<Case> {
    let mut c = Case {
        classname: String::new(),
        name: String::new(),
        file: None,
        line: None,
        outcome: CaseOutcome::Passed,
        message: String::new(),
    };
    for a in e.attributes() {
        let a = a.context("invalid attribute in JUnit XML")?;
        let v = a.unescape_value()?.into_owned();
        match a.key.local_name().as_ref() {
            b"classname" => c.classname = v,
            b"name" => c.name = v,
            b"file" => c.file = Some(v),
            b"line" => c.line = v.parse().ok(),
            _ => {}
        }
    }
    Ok(c)
}

fn fail(c: &mut Case, e: &BytesStart) -> Result<()> {
    c.outcome = CaseOutcome::Failed;
    for a in e.attributes() {
        let a = a.context("invalid attribute in JUnit XML")?;
        if a.key.local_name().as_ref() == b"message" {
            let m = a.unescape_value()?;
            if !m.is_empty() {
                c.message.push_str(&m);
                c.message.push('\n');
            }
        }
    }
    Ok(())
}

fn push_text(c: Option<&mut Case>, text: &str) {
    if let Some(c) = c {
        c.message.push_str(text);
    }
}
