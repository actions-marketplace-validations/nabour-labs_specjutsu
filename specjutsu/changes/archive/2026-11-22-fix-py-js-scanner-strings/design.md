## Context

Python `block_end` decides by indentation alone. JS `quoted` skips the character after a `\`,
including a newline, without telling the caller.

## Decisions

- Python: walk the body lines with a small state of the open triple quote (`"""` or `'''`),
  toggled by counting unescaped occurrences on each line; a line that starts, or is wholly,
  inside an open triple-quoted string never ends the block. Single-quoted strings cannot span
  lines in Python without `\`, and a `\` continuation keeps its own line indented in practice,
  so they are left alone. Alternative: a full Python tokenizer; far more than this needs.
- JS: `quoted` takes the `line` counter like `template` already does and increments it on the
  newline it skips.

## Risks / Trade-offs

- [A `"""` inside a `'''` string or a comment toggles the wrong state] -> each line tracks only
  the quote kind that is open; `#` comments outside strings stop the line scan.
