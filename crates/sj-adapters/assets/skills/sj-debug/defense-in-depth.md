# Defense in depth

When bad data caused a bug, one check at the root cause fixes this bug. It doesn't stop the
next caller, a refactor or a test fixture from sending the same bad data along another
path. Where the bad value passed through several layers, add a check at each one, so the
bug becomes impossible instead of merely fixed.

Do this after the root cause is fixed and its scenario test is green, not instead of it.

## The layers

1. **Where it enters.** The CLI argument parser, the API handler, the config loader. Reject
   input that is clearly wrong: empty, missing, the wrong type, a path that doesn't exist.
   Say what was wrong and what was expected.
2. **Where it's used.** The function that acts on the value checks the assumption it relies
   on ("the project folder must not be empty"). This catches callers that skipped the
   entry point: other modules, tests, future code.
3. **Where it's dangerous.** Before an operation that can do damage (deleting files,
   running `git` in a folder, writing outside the repository, spending money), refuse in a
   context where it can't be right. For example, refuse to run `git init` outside a temp
   folder while tests are running.
4. **Where you'll need it next time.** Where a value crosses a boundary that is hard to see
   into, such as a subprocess, a hook or a background job, log enough context to trace it
   back.

## How to apply it

- List the path the bad value took, from where it entered to where it did damage.
- Add a check at each layer on that path that could have stopped it. Not every layer needs
  all four kinds; pick the ones that would have caught this bug.
- Each check gets a test, linked to the bug's scenario or to a `[keep]` scenario for the
  layer.
- Then try to get past each check: call the inner function directly, pass the value from a
  test fixture, set the environment differently. If one of these still causes the bug, a
  layer is missing.

## Keep it proportionate

- Checks go on the path the bug actually took, not on every function in the codebase.
- A check that can never fail, because the type system already rules the value out, is
  noise. Use the type instead.
- Don't swallow the error at the outer layer to make it go away. A check fails loudly with a
  message that says what was wrong.
