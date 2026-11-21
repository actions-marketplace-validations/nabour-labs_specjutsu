---
name: specjutsu
description: Use when a project should start using specjutsu — "install specjutsu", "set up specjutsu", "add sj to this repo", "use spec-driven development here". Installs the `sj` CLI if it is missing, sets up a new or existing project, installs the agent adapters, and checks the result. Works before `sj` is installed.
---

# Set up specjutsu in this project

specjutsu is a CLI, `sj`, that ties every behaviour to a spec scenario with an ID, every
scenario to a test, and every test to a red run before its code exists. This skill gets a
project from nothing to a checked, working setup. Run each step, show the user what matters in
its output, and stop on anything you don't expect.

## 1. Check for sj

```bash
sj --version
```

If it prints `sj 0.4.0` or later, go to step 3. If the command is not found, or the version
is older, install it.

## 2. Install sj

Show the user the command and ask before running it: it downloads a binary.

macOS and Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/nabour-labs/specjutsu/main/install.sh | sh
```

`install.sh` picks the release for the machine, checks it against the release's
`SHA256SUMS`, and installs `sj` into `~/.local/bin` (set `SJ_INSTALL_DIR` to change that,
`SJ_VERSION` to pin a release). If it says to add the folder to your `PATH`, do that for
this shell (`export PATH="$HOME/.local/bin:$PATH"`) and tell the user to add it to their
shell profile.

Windows, other platforms, or no `curl`: with a Rust toolchain, run
`cargo install specjutsu`; otherwise point the user at the zip on
https://github.com/nabour-labs/specjutsu/releases. Run `sj --version` again before going on.

## 3. Set up the project

Run `git status` first. Setup writes files, so tell the user if the tree has uncommitted
changes and let them decide whether to go on.

If `specjutsu.toml` already exists, keep it: the project is set up. Don't replace it, and
skip to step 4.

Otherwise, pick one:

- **Existing code with tests** (a `Cargo.toml`, `package.json`, `pyproject.toml`, or a
  workspace): preview, then write.

  ```bash
  sj adopt --dry-run   # show the user the detected packages and runners
  sj adopt             # write specjutsu.toml and create the spec folders
  ```

  `sj adopt` reports `no runner detected: <path>` for packages with no test runner. Tell
  the user; they can add a `[[package.suite]]` by hand later.
- **New or empty project:** `sj init`.

If the project already uses OpenSpec, Spec Kit or Superpowers, say so and offer
`sj migrate --from <tool>`. It is a dry run until `--write`, and needs a clean tree.

## 4. Install the agent adapters

```bash
sj init --agents auto
```

`auto` installs for every agent the repo already has files for (`CLAUDE.md`, `.cursor/`,
`AGENTS.md`, …) and falls back to an `AGENTS.md` block. To pick, name them:
`sj init --agents claude,codex`. This adds an instructions block, the `sj-*` skills, and,
where the agent supports it, a hook that blocks source edits until the task's tests have
failed once.

## 5. Check the setup

```bash
sj doctor
sj status
```

`sj doctor` must say `doctor: ok`. For `missing runner <name>`, the test runner is not
installed: install the project's dependencies (`npm install`, `uv sync`, …) and rerun.
`sj status` shows the open changes; a fresh project has none.

## 6. Hand over

Tell the user what was written (`specjutsu.toml`, the spec folders, the adapter files) so
they can review and commit it. Then point onward: `sj-onboard` walks one toy change end to
end, `sj-explore` thinks an idea through, and `sj-propose` turns an agreed idea into a
change. For CI, the GitHub Action is `uses: nabour-labs/specjutsu@v0`; see the manual at
https://nabour-labs.github.io/specjutsu/manual.html.
