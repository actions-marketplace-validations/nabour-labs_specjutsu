---
name: sj-debug
description: Use when something is broken — a bug report, a failing test, an error or unexpected behaviour. Finds the root cause and captures it as a failing scenario test before any fix.
---

# Debug from evidence

No fix without a failing test that reproduces the bug, and no guessing at causes.

## 1. Reproduce

- Get the exact symptom: input, expected output, actual output, error text.
- Run the narrowest existing tests first. `sj test <ID>` for the scenario that covers the
  behaviour, or `sj test <package>`. Read the failure excerpt it prints, not raw logs.
- If you can't reproduce it, say so and ask for what's missing. Don't fix what you can't see.

## 2. Find the root cause

Work from the symptom back to the cause, one hypothesis at a time:

1. State the hypothesis: "the token is compared before it's decoded".
2. Name what would confirm or refute it, then check that one thing (a log line, a debugger
   step, a smaller input).
3. If refuted, write down what you ruled out and form the next one.

Stop at the cause, not the first place the symptom shows. After three refuted hypotheses,
step back: re-read the failing path end to end, check recent changes (`git log -p` on the
files involved), and question your assumptions about inputs.

## 3. Capture it as a scenario

- If an existing scenario describes the right behaviour and its test missed the bug, add the
  case to that scenario's test, or add a new scenario.
- If no scenario describes it, add one to the open change, or create a change with
  `sj spec new fix-<bug>`. Write the scenario with the concrete failing values, and name it
  in a task.

Write the test, linked by the scenario ID, and record it red:

```bash
sj test FIX-001
```

It must fail the way the bug report says. Commit it: `test(FIX-001): red`.

## 4. Fix

Make the smallest change that fixes the cause, run `sj test FIX-001` until it's green, then
run the package's tests (`sj test <package>`) to check nothing else broke. Commit it as
`feat(FIX-001): green`: the gate's history check pairs `test(…): red` with `feat(…)` commits.

## 5. Report

In a few lines: the cause, why the existing tests missed it, the fix, and any similar code
that may have the same bug. Suggest a scenario for that code, don't fix it unasked.
<!-- specjutsu:generated 55fae085b608acd5 -->
