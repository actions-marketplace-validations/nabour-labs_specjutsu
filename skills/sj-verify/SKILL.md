---
name: sj-verify
description: Use when about to say work is done, fixed or passing, when committing a finished task, when a subagent reports success, and before opening a PR. Runs sj verify and treats every problem line as not done.
---

# Verify before claiming done

"Done" is a checked fact, not a feeling. A claim without fresh evidence is a guess, even
when it turns out to be right.

## The gate

Before you say anything is done, fixed, passing or ready, in any wording, including
"should work now", "looks good" or a thumbs-up:

1. **Name the command** that proves the claim. For a change, that is `sj verify`. For "the
   bug is fixed", it is the bug's scenario test. For "it builds", it is the build.
2. **Run it now**, after your last edit. An earlier run, a run before a merge, or a run in
   another worktree doesn't count.
3. **Read the whole output**: the summary line, every problem line, the exit code.
4. **Check that it says what you're about to claim.** If it doesn't, report what it does
   say instead.
5. **Only then make the claim**, and quote the evidence with it.

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
- **Any problem line means not done.** Don't call the work complete, don't mark it as
  passing, and don't open a PR. Fix it, or tell the user exactly which lines remain and why.
- **`unproven` locally** can mean the red run was made outside `sj test`, for example by
  plain `cargo test`. CI's red-on-base checks it again. Say so; don't hide it.
- **Tests passing is not the requirement met.** `verify: ok` says every scenario has a
  proven test. Read the scenarios once more and check the change does what they say, and
  nothing they don't.

## 3. Subagents and other sessions

A subagent's report is not evidence. "DONE, all tests pass" from an implementer, reviewer
or another session is a claim, made by something that wanted to finish. Before you repeat
it or act on it:

- look at the diff it made (`git diff <base>..HEAD`, or `git log` for its commits),
- run `sj verify <change>` yourself,
- and only then report it, with your own evidence.

## 4. Report

Quote the summary line and every problem line. Never summarize "tests pass" without having
run `sj verify` in this session after the last edit. If you edited anything after the run,
run it again.

## Rationalizations

| Thought | Reality |
| --- | --- |
| "It should work now." | Run it. "Should" is not evidence. |
| "I ran it a minute ago." | You've edited since. Run it again. |
| "The subagent said it passes." | A report is a claim. Check the diff, run `sj verify`. |
| "The tests pass, so it's done." | Tests prove the scenarios you wrote. Check them against the request. |
| "It's a one-line change." | One-line changes break builds too. |
| "I'm confident." | Confidence is not a command output. |

## Red flags

Stop and run the gate if you're about to:

- write "done", "fixed", "passing", "ready" or "should work" without output from this turn,
- commit, push, open a PR or mark a task done with no `sj verify` since the last edit,
- pass on a subagent's success report without having looked at its diff,
- report only the lines that passed,
- describe a problem line as "just the gate being strict".
