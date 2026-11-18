---
name: sj-apply
description: Use when implementing a change's tasks — "start implementing", "continue", "work through the tasks". Runs the test-first loop per task with sj next, sj test and sj task done, including parallel tasks with subagents.
---

# Implement a change, test first

The ledger records every `sj test` run. A scenario counts as proven only when a failing run
was recorded before a passing one on different code, and CI checks the same thing again on
the base branch. Don't skip the red step.

## The loop, one task at a time

### 1. Get the task

```bash
sj next            # or: sj next <change>
```

It prints the task, the full text of each scenario it names, the files changed so far, the
principles file and the exact test commands. That is all the context you need. Don't read
the whole change folder or every spec.

### 2. Write the failing test

- One test per scenario, linked by ID: put it in the test name (`fn auth_005_refreshes_token`,
  `it("AUTH-005 refreshes the token")`) or a comment line above it (`// spec: AUTH-005`).
- Put Rust tests in the crate's `tests/` directory, and other tests where the suite's `tests`
  globs find them, so CI can run them against the base branch.
- The test asserts the THEN, with the scenario's concrete values.

Delegate: scaffolding a test from a scenario's WHEN/THEN is mechanical and fine for a cheaper
model or subagent.

### 3. Record red

```bash
sj test AUTH-005
```

It must fail, and for the right reason: the behaviour is missing, not a typo in the test. A
compile error because the function doesn't exist yet counts as red. If it passes, either the
behaviour already exists (tell the user) or the test doesn't test the scenario (fix the test).

Commit the failing test: `test(AUTH-005): red`.

### 4. Make it pass

Write the least code that makes the test pass, then run it again:

```bash
sj test AUTH-005
```

When it's green, refactor if needed and run it again. Commit: `feat(AUTH-005): green`.
Output shows failures only. Read the excerpt it prints, not raw logs.

### 5. Close the task

```bash
sj task done <change> <task-number>
```

In strict mode it refuses until each scenario is proven, and names the missing evidence.
Don't edit the checkbox by hand. `sj status` shows the change's progress at any time.

## Parallel tasks

```bash
sj next --parallel
```

It lists open `[P]` tasks in batches whose tests share no file. Give each task in one batch
to its own subagent with only that task's `sj next <change>` output. Each runs steps 2–4.
When they finish, review each diff, then run `sj task done` for each task yourself. Run one
batch at a time.

## When something goes wrong

- A test you didn't touch starts failing: stop and use `sj-debug`.
- The scenario turns out wrong or incomplete: stop and fix the spec first (`sj-propose`),
  then come back. Don't implement behaviour no scenario describes.
- Code that needs no scenario (a refactor, a build script) needs an exemption line in the
  proposal. Ask the user before adding one.

When every task is done, continue with `sj-verify`.
<!-- specjutsu:generated 98964065eccedfdd -->
