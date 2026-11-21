## Context

`check` removed every `` `…` `` span from a line and then matched markers in what was left. That
hides markers quoted in code, but also deletes code inside a real marker's question.

## Decisions

Match markers on the original line and skip a match only when it starts inside an inline-code
span. Alternative considered: replace spans with placeholders and restore them afterwards; more
code for the same result.

## Risks / Trade-offs

A question containing `]` inside backticks still ends at that `]`, as before.
