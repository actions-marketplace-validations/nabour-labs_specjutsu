//! GATE-026, GATE-030..035 and GATE-038 CLI scenarios from openspec/changes/polyglot-gate: the
//! gate across suites.
//!
//! A fake `pnpm` on PATH stands in for vitest and Playwright (Unix only). Its vitest run fails
//! without a report unless `pnpm install` ran in the package first, fails to load the test file
//! when the test imports `refreshToken` and `src/auth.ts` has none, and otherwise passes only
//! when `src/auth.ts` holds the fix (`rejectExpired`).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const BASE_SPEC: &str =
    "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
    #### Scenario: AUTH-001 Accepts a valid token\n- **WHEN** x\n- **THEN** y\n";

const AUTH_004: &str =
    "\n#### Scenario: AUTH-004 Rejects an expired token\n- **WHEN** x\n- **THEN** y\n";

const CHAT_002: &str = "\n#### Scenario: CHAT-002 Sends a message\n- **WHEN** x\n- **THEN** y\n";

const CONFIG: &str = r#"schema = 1

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/api/src/**/*.test.ts"]
setup = "pnpm install --frozen-lockfile"

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
red_on_base = "skip"
reason = "needs staging API"
"#;

const BUGGY: &str = "export const check = (t) => true\n";
const FIXED: &str = "export const check = (t) => rejectExpired(t)\n";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A repository on `main` with the config, a buggy `auth.ts` and installed head dependencies,
/// switched to branch `feature`.
fn repo() -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, ".gitignore", "node_modules/\n");
    write(r, "pnpm-lock.yaml", "");
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "apps/api/src/auth.ts", BUGGY);
    write(r, "apps/app/src/chat.ts", "export const send = () => {}\n");
    // The head checkout has its dependencies; a base worktree only gets them from `setup`.
    write(r, "apps/api/node_modules/.installed", "");
    repo.commit("base");
    repo.branch("feature");
    repo
}

fn add_auth_004(repo: &Repo, test: &str, source: &str) {
    let r = repo.path();
    write(
        r,
        "openspec/specs/auth/spec.md",
        &format!("{BASE_SPEC}{AUTH_004}"),
    );
    write(r, "apps/api/src/auth.test.ts", test);
    write(r, "apps/api/src/auth.ts", source);
    repo.commit("AUTH-004");
}

const TEST: &str = "import { check } from './auth'\n\
    it(\"AUTH-004 rejects expired token\", () => { expect(check('old')).toBe(false) })\n";

#[cfg(unix)]
fn fake_bin(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let pnpm = r#"#!/bin/sh
echo "$1 $2 $PWD" >> "$FAKE_LOG"
if [ "$1" = install ]; then
  if [ -n "$FAKE_SETUP_FAIL" ]; then
    echo " ERR_PNPM_OUTDATED_LOCKFILE  Cannot install with frozen-lockfile" >&2
    exit 1
  fi
  mkdir -p node_modules && touch node_modules/.installed
  exit 0
fi
if [ "$2" = playwright ]; then
  printf '<testsuites><testsuite name="e2e"><testcase classname="e2e/chat.spec.ts" name="sends a message"/></testsuite></testsuites>' > "$PLAYWRIGHT_JUNIT_OUTPUT_NAME"
  exit 0
fi
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
[ -f node_modules/.installed ] || { echo "Error: Cannot find module 'vitest'" >&2; exit 1; }
[ -f src/auth.test.ts ] || { echo "No test files found" >&2; exit 1; }
case_='<testcase classname="src/auth.test.ts" name="AUTH-004 rejects expired token"'
if grep -q refreshToken src/auth.test.ts && ! grep -q refreshToken src/auth.ts; then
  body='<testcase classname="src/auth.test.ts" name="src/auth.test.ts"><failure message="SyntaxError: The requested module does not provide an export named refreshToken"/></testcase>'
  code=1
elif grep -q rejectExpired src/auth.ts; then
  body="$case_/>"
  code=0
else
  body="$case_><failure message=\"expected false, got true\"/></testcase>"
  code=1
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;
    let p = dir.join("pnpm");
    std::fs::write(&p, pnpm).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `sj gate --base main` with the fake `pnpm` first on PATH; returns the command and its log.
#[cfg(unix)]
fn gate(repo: &Repo, bin: &tempfile::TempDir) -> (Command, PathBuf) {
    fake_bin(bin.path());
    let log = bin.path().join("calls.log");
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut cmd = Command::cargo_bin("sj").unwrap();
    cmd.args(["gate", "--base", "main", "--root"])
        .arg(repo.path())
        .env("PATH", path)
        .env("FAKE_LOG", &log);
    (cmd, log)
}

#[cfg(unix)]
fn calls(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

#[cfg(unix)]
#[test]
fn gate_030_a_co_located_test_is_overlaid_without_its_source_file() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert()
        .success()
        .stdout("gate: pass — 1 scenario proven (1 red:assert)\n");
    // The head checkout is untouched.
    assert_eq!(
        std::fs::read_to_string(repo.path().join("apps/api/src/auth.ts")).unwrap(),
        FIXED
    );
}

#[cfg(unix)]
#[test]
fn gate_031_base_dependencies_are_installed_before_the_base_run() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, log) = gate(&repo, &bin);
    cmd.assert().success();
    let calls = calls(&log);
    let installs: Vec<usize> = (0..calls.len())
        .filter(|&i| calls[i].starts_with("install --frozen-lockfile"))
        .collect();
    assert_eq!(installs.len(), 1, "{calls:#?}");
    let install_dir = calls[installs[0]].rsplit(' ').next().unwrap();
    assert!(install_dir.ends_with("/apps/api"), "{calls:#?}");
    // The base vitest run comes after the install, in the same directory.
    assert!(
        calls[installs[0] + 1..]
            .iter()
            .any(|c| c.starts_with("exec vitest") && c.ends_with(install_dir)),
        "{calls:#?}"
    );
}

#[cfg(unix)]
#[test]
fn gate_032_a_failing_base_setup_is_an_environment_error() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.env("FAKE_SETUP_FAIL", "1")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("pnpm install --frozen-lockfile"))
        .stderr(predicate::str::contains("ERR_PNPM_OUTDATED_LOCKFILE"));
}

#[cfg(unix)]
#[test]
fn gate_033_a_missing_symbol_on_base_is_red_compile() {
    let repo = repo();
    add_auth_004(
        &repo,
        "import { refreshToken } from './auth'\n\
         it(\"AUTH-004 rejects expired token\", () => { expect(refreshToken('old')).toBe(null) })\n",
        "export const refreshToken = (t) => rejectExpired(t)\n",
    );
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert()
        .success()
        .stdout("gate: pass — 1 scenario proven (1 red:compile)\n");
}

#[cfg(unix)]
#[test]
fn gate_034_an_e2e_suite_that_skips_red_on_base_is_unverifiable() {
    let repo = repo();
    let r = repo.path();
    write(
        r,
        "openspec/specs/auth/spec.md",
        &format!("{BASE_SPEC}{CHAT_002}"),
    );
    write(
        r,
        "apps/app/e2e/chat.spec.ts",
        "import { test } from '@playwright/test'\n\n// spec: CHAT-002\ntest(\"sends a message\", async () => {})\n",
    );
    repo.commit("CHAT-002");
    let line = "unverifiable CHAT-002 sends a message (needs staging API)";

    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, log) = gate(&repo, &bin);
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains(format!("fail    {line}")));
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.arg("--allow-unverifiable")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("warn    {line}")));
    // Nothing ran on base: no install, and Playwright only at head.
    let calls = calls(&log);
    assert!(
        !calls.iter().any(|c| c.starts_with("install")),
        "{calls:#?}"
    );
    assert!(
        calls.iter().any(|c| c.starts_with("exec playwright")),
        "{calls:#?}"
    );
}

#[test]
fn gate_035_only_files_matching_a_package_s_sources_need_a_scenario() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(
        r,
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \"backend\"\nsources = [\"backend/src/**/*.py\"]\n\n\
         [[package.suite]]\nrunner = \"pytest\"\ntests = [\"backend/tests/**\"]\n",
    );
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "backend/src/diff.py", "A = 1\n");
    write(r, "backend/notebooks/explore.py", "B = 1\n");
    repo.commit("base");
    repo.branch("feature");
    write(r, "backend/src/diff.py", "A = 2\n");
    write(r, "backend/notebooks/explore.py", "B = 2\n");
    repo.commit("change");

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main", "--root"])
        .arg(r)
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    unspecified change: backend/src/diff.py\n",
        ));
}

#[test]
fn gate_038_files_on_the_non_source_list_pass_without_an_exemption() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(
        r,
        "specjutsu.toml",
        "schema = 1\nnon_source = [\"infra/**\"]\n\n[[package]]\npath = \"backend\"\n\
         sources = [\"backend/src/**\"]\n\n[[package.suite]]\nrunner = \"pytest\"\ntests = []\n",
    );
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "infra/main.tf", "a = 1\n");
    repo.commit("base");
    repo.branch("feature");
    write(r, "infra/main.tf", "a = 2\n");
    repo.commit("change");

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main", "--root"])
        .arg(r)
        .assert()
        .success()
        .stdout("gate: pass — no changed scenarios to prove\n");
}
