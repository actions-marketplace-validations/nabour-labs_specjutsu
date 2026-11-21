## ADDED Requirements

### Requirement: sj spec check keeps inline code in open questions
`sj spec check <change>` SHALL print each `clarify` question with the text of its inline-code spans intact. A marker that itself starts inside an inline-code span SHALL still be ignored.

#### Scenario: SPEC-012 Clarify questions keep their inline code
- **WHEN** `design.md` contains a clarify marker whose question is "do `PreToolUse` hooks fire for `apply_patch` file edits?"
- **THEN** check prints `clarify do PreToolUse hooks fire for apply_patch file edits? (design.md:<line>)`, with the inline code text kept (backticks may be kept)
