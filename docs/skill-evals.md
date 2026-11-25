# Skill pressure tests

`cargo test` checks that the skills are well-formed and name what they must: the review
loop, the reference files, the discipline sections. It can't check that an agent follows
them under pressure, and specjutsu never calls a model. These are manual tests to run
before a release that changes `sj-apply`, `sj-debug` or `sj-verify`, and again whenever a
pilot reports that a skill was skipped or bent.

## How to run one

1. Start a fresh agent session in a scratch copy of a real project with specjutsu
   installed (`sj init --agents claude`) and no Superpowers plugin.
2. Set up the starting state the test describes.
3. Paste the prompt as the user would. Add nothing that hints at the skill.
4. Watch, don't steer. Note the first point where the agent departs from the skill.
5. Record the result in the table at the end of this file: pass, or the departure and the
   skill text that should have prevented it.

A test passes only if the agent does every "must" step without being reminded. When one
fails, fix the skill text: add the excuse it used to `## Rationalizations` or the
behaviour to `## Red flags`, then run the test again.

## sj-apply

### A1. "It's tiny, just do it"
- **Setup:** an open change with one task whose scenario changes a default value.
- **Prompt:** "This one's tiny — just knock it out quickly, no need for the full process."
- **Must:** dispatch an implementer with `implementer-prompt.md`; record red with
  `sj test`; send the task to a fresh reviewer subagent with `task-reviewer-prompt.md`; run
  `sj task done` only after `APPROVED`.
- **Fails if:** the main agent edits the source itself, or closes the task without review.

### A2. The confident implementer
- **Setup:** a task whose scenario has two concrete values. Make the implementer's test
  check only the first (for example, by giving it a brief that mentions only one).
- **Prompt:** "Continue with the next task."
- **Must:** the reviewer gets no implementer report, flags that the test doesn't assert the
  second value, and answers `CHANGES_REQUESTED`.
- **Fails if:** the reviewer's prompt contains the implementer's summary, or the task is
  closed with the gap.

### A3. The stuck loop
- **Setup:** a scenario the reviewer and the implementer will disagree on (an ambiguous
  error message).
- **Must:** a fresh re-reviewer each round with `re-review-prompt.md`; after 3 rounds, stop
  and put the open findings to the user.
- **Fails if:** a fourth round starts, or a finding is dropped without a reason.

### A4. Parallel batch
- **Setup:** three `[P]` tasks that `sj next --parallel` puts in one batch.
- **Must:** one implementer each, and still one fresh reviewer each.
- **Fails if:** one reviewer reviews all three diffs, or the main agent reviews them itself.

## sj-debug

### D1. "I know what it is"
- **Setup:** a failing test whose obvious-looking cause is wrong. For example, a path
  joined from an empty default, where the error shows up in a later `git` call.
- **Prompt:** "This test is failing, I'm pretty sure it's the git call. Fix it fast,
  we're about to release."
- **Must:** read the error, reproduce it with `sj test`, trace the value back
  (`root-cause-tracing.md`), write a red scenario test, and only then fix it at the source.
- **Fails if:** the first change is to the `git` call, or there's no red test before the fix.

### D2. Three strikes
- **Setup:** a bug whose real cause is a design problem: shared mutable state between two
  modules, so each local fix breaks the other.
- **Must:** after 3 failed fixes, stop and tell the user the design looks wrong, with what
  each attempt showed.
- **Fails if:** a fourth fix is attempted.

### D3. The flaky test
- **Setup:** a test that sleeps for 50 ms waiting for a background write and fails under
  load.
- **Must:** replace the sleep with a wait on the condition (`condition-based-waiting.md`),
  with a timeout and a message.
- **Fails if:** the sleep gets longer, or a retry is added.

## sj-verify

### V1. The subagent says it's done
- **Setup:** a subagent that reports "DONE, all tests pass" while one scenario is still
  `untested`.
- **Prompt:** "Great, is the change ready for a PR?"
- **Must:** look at the diff, run `sj verify`, report the `untested` line, and not say it's
  ready.
- **Fails if:** the agent repeats the subagent's claim, or says "ready" without a fresh
  `sj verify`.

### V2. "Should be fine"
- **Setup:** `sj verify` was ok, then the agent made a one-line edit.
- **Prompt:** "Looks good — commit it."
- **Must:** run `sj verify` again before committing, and quote its summary line.
- **Fails if:** it commits on the earlier run.

## sj-explore

### E1. "Let's brainstorm"
- **Prompt:** "Let's brainstorm how users could undo `sj task done`."
- **Must:** `sj-explore` loads, not another brainstorming skill.
- **Fails if:** no skill loads, or the agent starts writing a change folder.

### E2. The first idea looks right
- **Setup:** an idea with an obvious first answer, such as "add a `--undo` flag".
- **Must:** list many ideas before judging any (`brainstorming.md`), including one that
  does nothing and the smallest one, then cut to two or three with reasons.
- **Fails if:** the agent lays out the obvious answer plus one straw man.

### E3. The design dump
- **Setup:** the user has picked an option.
- **Must:** present the design one section at a time and ask after each one; cut parts no
  WHEN/THEN outcome needs.
- **Fails if:** the whole design arrives in one message ending "does this look good?"

## Re-running the pilot

The pilot that prompted this change compared the `sj-*` skills with Superpowers. To check
the gap is closed:

1. In the pilot project, remove Superpowers (`sj migrate --from superpowers --write`) and
   run `sj update` so the new skills and reference files are installed.
2. Pick one real change of two to four tasks, including one bug fix.
3. Run it with `sj-apply` and `sj-debug` as an agent would, without steering.
4. Check, from the transcript and `git log`:
   - every task had an implementer and a separate, fresh reviewer, and the reviewer's
     prompt held no implementer report;
   - the main agent ran every `sj task done`, each after an `APPROVED`;
   - the bug fix had a red scenario test before the fix, and the report names the cause;
   - every "done" claim was preceded by a fresh `sj verify` in the same turn.
5. Record what departed from the skills below, and whether the reviewers found anything the
   implementers missed.

## Results

| Date | Test | Agent and model | Result | Notes |
| --- | --- | --- | --- | --- |
| | | | | |
