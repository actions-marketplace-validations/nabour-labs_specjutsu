## Context

The skill check (`sj_adapters::skills::check`) already enforces phrases and named
references for `sj-apply`, `sj-debug` and `sj-verify` (SKILL-014..016). `sj-explore` has no
rules beyond the shared size, frontmatter and foreign-command rules.

## Goals / Non-Goals

**Goals:** brainstorming triggers `sj-explore`; the skill diverges before it converges; the
technique lives in a reference file that coverage points at.

**Non-Goals:** a separate `sj-brainstorm` skill. It would compete with `sj-explore` for the
same requests and add a step without adding a stage.

## Decisions

1. **Phrase rules, as for sj-debug.** The check looks for `brainstorm` in the description,
   `brainstorming.md` and `go wide` in the body (whitespace and case normalised). The
   alternative, checking section headings, would tie the rule to layout rather than content.
2. **The reference carries the technique.** `brainstorming.md` must name `YAGNI` and say to
   present the design one section at a time. The check reads it only when it exists; a
   missing file is already reported by SKILL-010.
3. **Coverage depth moves to `brainstorming.md`.** The existing depth rule (SKILL-017)
   then proves the file ships.

## Risks / Trade-offs

- [Phrase rules can be satisfied by wording alone] -> They catch the regression that
  matters (the technique removed or the trigger dropped); quality is a review matter, as
  for the other skills.
