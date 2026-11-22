## Context

`merge` in `crates/sj-core/src/authoring/archive.rs` refuses a MODIFIED block unless every
scenario name in the current requirement appears in it, comparing `#### Scenario:` text as is.

## Decisions

- Strip a leading `[keep]` marker after the ID (via `spec::split_marker`) on both sides before
  comparing. Other text, including other markers, still compares literally.
- The merged spec takes the delta's block unchanged, so a `[keep]` heading lands in the main
  spec with its marker. Main specs already carry `[keep]` headings from ADDED blocks, and the
  marker records that the scenario was carried unchanged. Stripping it on merge was the
  alternative; it would make ADDED and MODIFIED behave differently.

## Risks / Trade-offs

- A `[keep]` heading stays in the main spec after archive -> consistent with ADDED blocks today.
