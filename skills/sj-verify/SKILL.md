---
name: sj-verify
description: Use when about to say work is done, fixed or passing, when committing a finished task, and before opening a PR. Runs sj verify and treats every problem line as not done.
---

# Verify before claiming done

"Done" is a checked fact, not a feeling. Run the check and report what it says.

## 1. Run it

```bash
sj verify            # every open change; or: sj verify <change>
```

It runs the change's scenario tests and records them, then reports what CI's gate would
fail on, except red-on-base:

| Line | Meaning | What to do |
| --- | --- | --- |
| `untested <ID>` | no test is linked to the scenario | write the test (`sj-apply`) |
| `failing <test>  <ID>` | the test fails now | fix it, or `sj-debug` if unexpected |
| `build-failed <ID>` | the tests don't compile | fix the build |
| `unproven <ID> (<why>)` | no red run before the green one | see below |
| `orphan <ID> <test>` | a test names an ID no spec defines | fix the ID or add the scenario |
| `uncovered` / `unknown` / `clarify` | `sj spec check` problems | `sj-propose` or `sj-clarify` |
| `unspecified change: <files>` | source changed with no scenario | add a scenario, or an exemption with the user's OK |

## 2. Judge the result

- **`verify: ok`** is the only result that means done.
- **Any problem line means not done.** Don't call the work complete, don't mark it as passing,
  and don't open a PR. Fix it, or tell the user exactly which lines remain and why.
- **`unproven` locally** can mean the red run was made outside `sj test`, for example by
  plain `cargo test`. CI's red-on-base checks it again. Say so; don't hide it.

## 3. Report

Quote the summary line and every problem line. Never summarize "tests pass" without having
run `sj verify` in this session after the last edit. If you edited anything after the run,
run it again.
