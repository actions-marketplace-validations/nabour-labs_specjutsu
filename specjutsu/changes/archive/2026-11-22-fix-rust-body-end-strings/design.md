## Context

`rust_body_end` scanned each line with a fresh `in_str` flag and only knew `"` strings and the
char literals `'{'` and `'}'`.

## Decisions

- Scan the rest of the file as one byte stream from the header line, keeping lexer state
  (normal string, raw string with its `#` count, block comment depth) across lines, and map the
  closing brace back to its line. Alternative: patch the per-line loop to carry `in_str`; it
  would still not know raw strings, whose `"` and `\` mean something else.
- Char literals are recognised as `'` + one char or an escape + `'`; anything else after `'`
  is a lifetime or label and is skipped as a plain byte.
- Raw strings start at `r`, `br` or `cr` followed by `#`* and `"`, only when the `r` does not
  continue an identifier.

## Risks / Trade-offs

- [An unterminated string runs to end of file] -> same fallback as before: `start + 1`.
