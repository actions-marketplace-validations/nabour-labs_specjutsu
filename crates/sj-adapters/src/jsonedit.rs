//! A JSON reader that keeps byte spans, so callers can splice text into a file without
//! reformatting the rest of it.

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Object(Vec<Member>),
    Array(Vec<Node>),
    Str(String),
    /// Numbers, `true`, `false` and `null`.
    Other,
}

/// A value and its byte range in the text, `start..end`.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub key: String,
    /// Offset of the key's opening quote.
    pub key_start: usize,
    pub value: Node,
}

impl Node {
    pub fn members(&self) -> &[Member] {
        match &self.kind {
            Kind::Object(m) => m,
            _ => &[],
        }
    }

    pub fn elements(&self) -> &[Node] {
        match &self.kind {
            Kind::Array(e) => e,
            _ => &[],
        }
    }

    pub fn get(&self, key: &str) -> Option<&Member> {
        self.members().iter().find(|m| m.key == key)
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.kind {
            Kind::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.kind, Kind::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.kind, Kind::Array(_))
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn err<T>(&self, what: &str) -> Result<T> {
        bail!("invalid JSON at byte {}: {what}", self.i)
    }

    fn value(&mut self) -> Result<Node> {
        self.ws();
        let start = self.i;
        let Some(&c) = self.b.get(self.i) else {
            return self.err("unexpected end");
        };
        let kind = match c {
            b'{' => {
                self.i += 1;
                let mut members = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b'}') {
                    self.i += 1;
                } else {
                    loop {
                        self.ws();
                        let key_start = self.i;
                        if self.b.get(self.i) != Some(&b'"') {
                            return self.err("expected a key");
                        }
                        let key = self.string()?;
                        self.ws();
                        if self.b.get(self.i) != Some(&b':') {
                            return self.err("expected `:`");
                        }
                        self.i += 1;
                        let value = self.value()?;
                        members.push(Member {
                            key,
                            key_start,
                            value,
                        });
                        self.ws();
                        match self.b.get(self.i) {
                            Some(b',') => self.i += 1,
                            Some(b'}') => {
                                self.i += 1;
                                break;
                            }
                            _ => return self.err("expected `,` or `}`"),
                        }
                    }
                }
                Kind::Object(members)
            }
            b'[' => {
                self.i += 1;
                let mut elements = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b']') {
                    self.i += 1;
                } else {
                    loop {
                        elements.push(self.value()?);
                        self.ws();
                        match self.b.get(self.i) {
                            Some(b',') => self.i += 1,
                            Some(b']') => {
                                self.i += 1;
                                break;
                            }
                            _ => return self.err("expected `,` or `]`"),
                        }
                    }
                }
                Kind::Array(elements)
            }
            b'"' => Kind::Str(self.string()?),
            _ => {
                while self.i < self.b.len()
                    && matches!(self.b[self.i], b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'+' | b'.')
                {
                    self.i += 1;
                }
                let word = std::str::from_utf8(&self.b[start..self.i]).unwrap_or("");
                let ok = matches!(word, "true" | "false" | "null")
                    || (!word.is_empty() && word.parse::<f64>().is_ok());
                if !ok {
                    return self.err("unexpected value");
                }
                Kind::Other
            }
        };
        Ok(Node {
            start,
            end: self.i,
            kind,
        })
    }

    fn string(&mut self) -> Result<String> {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            let Some(&c) = self.b.get(self.i) else {
                return self.err("unterminated string");
            };
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    let Some(&e) = self.b.get(self.i + 1) else {
                        return self.err("unterminated escape");
                    };
                    self.i += 2;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self.b.get(self.i..self.i + 4).unwrap_or(&[]);
                            let code = std::str::from_utf8(hex)
                                .ok()
                                .and_then(|h| u32::from_str_radix(h, 16).ok());
                            let Some(code) = code else {
                                return self.err("bad \\u escape");
                            };
                            self.i += 4;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        _ => return self.err("bad escape"),
                    }
                }
                _ => {
                    // Copy one UTF-8 character.
                    let len = match c {
                        0x00..=0x7f => 1,
                        0xc0..=0xdf => 2,
                        0xe0..=0xef => 3,
                        _ => 4,
                    };
                    let end = (self.i + len).min(self.b.len());
                    out.push_str(&String::from_utf8_lossy(&self.b[self.i..end]));
                    self.i = end;
                }
            }
        }
    }
}

/// Parse a whole document: one value, surrounded only by whitespace.
pub fn parse(text: &str) -> Result<Node> {
    let mut p = Parser {
        b: text.as_bytes(),
        i: 0,
    };
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return p.err("trailing characters");
    }
    Ok(v)
}

/// The whitespace at the start of the line holding `pos`.
pub fn line_indent(text: &str, pos: usize) -> &str {
    let start = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let line = &text[start..];
    let n = line.len() - line.trim_start_matches([' ', '\t']).len();
    &line[..n]
}

/// The file's indent unit: the leading whitespace of its first indented line, else two spaces.
pub fn indent_unit(text: &str) -> &str {
    for line in text.lines().skip(1) {
        let n = line.len() - line.trim_start_matches([' ', '\t']).len();
        if n > 0 && n < line.len() {
            return &line[..n];
        }
    }
    "  "
}
