//! RUN-011..017 CLI scenarios from openspec/changes/polyglot-runners: `sj test` across suites.
//!
//! Fake `pnpm` and `python3` executables on PATH log their arguments and write canned JUnit
//! reports, so the default launchers are exercised without real runners (Unix only). The junit
//! runner test copies a report with the OS shell and runs everywhere.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const SPEC: &str = "## Requirements\n\n### Requirement: R\nThe system SHALL work.\n\n\
    #### Scenario: AUTH-003 Reject expired token\n- **WHEN** x\n- **THEN** y\n\n\
    #### Scenario: CHAT-002 Send a message\n- **WHEN** x\n- **THEN** y\n\n\
    #### Scenario: SVC-001 Service works\n- **WHEN** x\n- **THEN** y\n";

const CONFIG: &str = r#"schema = 1

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = []

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/src/**/*.test.ts"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]

[[package]]
path = "python/worker"
sources = ["python/worker/**"]

[[package.suite]]
runner = "pytest"
tests = []
"#;

const API_XML: &str = r#"<testsuites><testsuite name="src/auth.test.ts">
<testcase classname="src/auth.test.ts" name="auth &gt; AUTH-003 rejects expired token"><failure message="AssertionError: expected 401, got 200">AssertionError: expected 401, got 200
 ❯ src/auth.test.ts:2:40</failure></testcase>
<testcase classname="src/auth.test.ts" name="auth &gt; passes"></testcase>
</testsuite></testsuites>"#;

const APP_VITEST_XML: &str = r#"<testsuites><testsuite name="src/chat.test.ts">
<testcase classname="src/chat.test.ts" name="CHAT-002 sends"></testcase>
<testcase classname="src/chat.test.ts" name="other"><skipped/></testcase>
</testsuite></testsuites>"#;

const APP_PW_XML: &str = r#"<testsuites><testsuite name="e2e/chat.spec.ts">
<testcase name="sends a message" classname="e2e/chat.spec.ts"></testcase>
</testsuite></testsuites>"#;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn polyglot() -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "openspec/specs/all/spec.md", SPEC);
    write(r, "pnpm-lock.yaml", "");
    write(
        r,
        "apps/api/src/auth.test.ts",
        "describe(\"auth\", () => {\n  it(\"AUTH-003 rejects expired token\", () => {})\n  it(\"passes\", () => {})\n})\n",
    );
    write(
        r,
        "apps/app/src/chat.test.ts",
        "it(\"CHAT-002 sends\", () => {})\nit(\"other\", () => {})\n",
    );
    write(
        r,
        "apps/app/e2e/chat.spec.ts",
        "import { test } from '@playwright/test'\n\n// spec: CHAT-002\ntest(\"sends a message\", async () => {})\n",
    );
    let tests: String = ('a'..='l')
        .map(|c| format!("def test_{c}():\n    pass\n\n"))
        .collect();
    write(r, "python/worker/tests/test_jobs.py", &tests);
    let cases: String = ('a'..='l')
        .map(|c| format!("<testcase classname=\"tests.test_jobs\" name=\"test_{c}\" />"))
        .collect();
    write(
        r,
        "canned/py.xml",
        &format!("<testsuites><testsuite name=\"pytest\">{cases}</testsuite></testsuites>"),
    );
    write(r, "canned/api.xml", API_XML);
    write(r, "canned/app-vitest.xml", APP_VITEST_XML);
    write(r, "canned/app-pw.xml", APP_PW_XML);
    repo
}

#[cfg(unix)]
fn fake_bin(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    // pnpm exec <vitest|playwright> …: log the call, then write the canned report for the package
    // (named by FAKE_<PKG>_<RUNNER>, where PKG is the cwd's basename) or fail without one.
    let pnpm = r#"#!/bin/sh
echo "pnpm $(basename "$PWD") $*" >> "$FAKE_LOG"
shift
runner="$1"
pkg=$(basename "$PWD" | tr 'a-z-' 'A-Z_')
var="FAKE_${pkg}_$(echo "$runner" | tr 'a-z' 'A-Z')"
eval src=\$$var
out="$PLAYWRIGHT_JUNIT_OUTPUT_NAME"
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
if [ -z "$src" ]; then
  echo "SyntaxError: Unexpected token (3:5)" >&2
  echo "  at src/auth.test.ts:3:5" >&2
  exit 1
fi
cp "$src" "$out"
grep -q '<failure' "$out" && exit 1
exit 0
"#;
    let python = r#"#!/bin/sh
echo "python3 $(basename "$PWD") $*" >> "$FAKE_LOG"
for a in "$@"; do
  case "$a" in --junitxml=*) cp "$FAKE_PY" "${a#--junitxml=}" ;; esac
done
exit 0
"#;
    for (name, body) in [("pnpm", pnpm), ("python3", python)] {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// `sj test <args>` with the fakes first on PATH; returns the assert and the logged calls.
#[cfg(unix)]
fn sj_test(
    repo: &Repo,
    args: &[&str],
    env: &[(&str, &str)],
) -> (assert_cmd::assert::Assert, Vec<String>) {
    let bin = repo.path().join("fakebin");
    std::fs::create_dir_all(&bin).unwrap();
    fake_bin(&bin);
    let log = repo.path().join("calls.log");
    let _ = std::fs::remove_file(&log);
    let canned = repo.path().join("canned");
    let mut cmd = Command::cargo_bin("sj").unwrap();
    cmd.arg("test")
        .args(args)
        .arg("--root")
        .arg(repo.path())
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("FAKE_LOG", &log)
        .env("FAKE_PY", canned.join("py.xml"));
    for (k, v) in env {
        let v = if v.ends_with(".xml") {
            canned.join(v).to_string_lossy().into_owned()
        } else {
            v.to_string()
        };
        cmd.env(k, v);
    }
    let assert = cmd.assert();
    let calls = std::fs::read_to_string(&log)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
    (assert, calls)
}

#[cfg(unix)]
#[test]
fn run_011_a_vitest_failure_is_reported_with_its_scenario_and_message() {
    let repo = polyglot();
    let (assert, calls) = sj_test(&repo, &["apps/api"], &[("FAKE_API_VITEST", "api.xml")]);
    assert
        .code(1)
        .stdout(predicate::str::starts_with(
            "test: fail — 1 failed, 1 passed\n",
        ))
        .stdout(predicate::str::contains(
            "fail    apps/api/src/auth.test.ts::auth > AUTH-003 rejects expired token  AUTH-003",
        ))
        .stdout(predicate::str::contains("expected 401, got 200"));
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(
        calls[0].starts_with(
            "pnpm api exec vitest run --reporter=default --reporter=junit --outputFile.junit="
        ),
        "{calls:?}"
    );
}

#[cfg(unix)]
#[test]
fn run_012_a_pytest_pass_prints_one_line() {
    let repo = polyglot();
    let (assert, calls) = sj_test(&repo, &["python/worker"], &[]);
    assert.success().stdout("test: pass — 12 passed\n");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(
        calls[0].starts_with("python3 worker -m pytest --junitxml="),
        "{calls:?}"
    );
    assert!(calls[0].ends_with("-o junit_family=xunit1"), "{calls:?}");
}

#[cfg(unix)]
#[test]
fn run_013_no_report_means_a_build_failure() {
    let repo = polyglot();
    let (assert, _) = sj_test(&repo, &["apps/api"], &[]);
    assert
        .code(1)
        .stdout(predicate::str::starts_with("test: fail — build failed\n"))
        .stdout(predicate::str::contains(
            "SyntaxError: Unexpected token (3:5)",
        ));
}

#[cfg(unix)]
#[test]
fn run_016_a_scenario_with_tests_in_two_runners_runs_both() {
    let repo = polyglot();
    let (assert, calls) = sj_test(
        &repo,
        &["CHAT-002"],
        &[
            ("FAKE_APP_VITEST", "app-vitest.xml"),
            ("FAKE_APP_PLAYWRIGHT", "app-pw.xml"),
        ],
    );
    assert
        .success()
        .stdout(predicate::str::starts_with("test: pass — 2 passed\n"));
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(
        calls[0].starts_with("pnpm app exec vitest run ")
            && calls[0].ends_with(" src/chat.test.ts -t CHAT-002 sends"),
        "{calls:?}"
    );
    assert_eq!(
        calls[1],
        "pnpm app exec playwright test --reporter=junit e2e/chat.spec.ts:4"
    );
}

#[cfg(unix)]
#[test]
fn run_017_a_single_suite_can_be_selected() {
    let repo = polyglot();
    let (assert, calls) = sj_test(
        &repo,
        &["apps/app:playwright"],
        &[("FAKE_APP_PLAYWRIGHT", "app-pw.xml")],
    );
    assert.success().stdout("test: pass — 1 passed\n");
    assert_eq!(calls, ["pnpm app exec playwright test --reporter=junit"]);
}

#[cfg(unix)]
#[test]
fn run_016_failed_suites_are_named_when_several_ran() {
    let repo = polyglot();
    let (assert, _) = sj_test(
        &repo,
        &[],
        &[
            ("FAKE_API_VITEST", "api.xml"),
            ("FAKE_APP_VITEST", "app-vitest.xml"),
            ("FAKE_APP_PLAYWRIGHT", "app-pw.xml"),
        ],
    );
    assert.code(1).stdout(predicate::str::starts_with(
        "test: fail — 1 failed, 15 passed, 1 ignored (failed: apps/api:vitest)\n",
    ));
}

#[test]
fn run_014_the_generic_junit_runner_runs_any_command() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    let command = if cfg!(windows) {
        r"if not exist build mkdir build && copy /Y report.src.xml build\junit.xml >NUL"
    } else {
        "mkdir -p build && cp report.src.xml build/junit.xml"
    };
    write(
        r,
        "specjutsu.toml",
        &format!(
            "schema = 1\n\n[[package]]\npath = \"svc\"\nsources = []\n\n[[package.suite]]\n\
             runner = \"junit\"\ntests = [\"svc/tests/**\"]\ncommand = '{command}'\nreport = \"build/junit.xml\"\n"
        ),
    );
    write(r, "openspec/specs/all/spec.md", SPEC);
    write(
        r,
        "svc/tests/a.test.js",
        "it(\"SVC-001 works\", () => {})\n",
    );
    write(
        r,
        "svc/report.src.xml",
        "<testsuite><testcase classname=\"svc\" name=\"SVC-001 works\"><failure message=\"boom\">boom at a.test.js:1</failure></testcase>\
         <testcase classname=\"svc\" name=\"unrelated\"/></testsuite>",
    );
    // A stale report from an earlier run must not be read.
    write(
        r,
        "svc/build/junit.xml",
        "<testsuite><testcase name=\"SVC-001 works\"/></testsuite>",
    );

    for scope in [None, Some("SVC-001")] {
        let mut cmd = Command::cargo_bin("sj").unwrap();
        cmd.arg("test");
        if let Some(s) = scope {
            cmd.arg(s);
        }
        cmd.arg("--root")
            .arg(r)
            .assert()
            .code(1)
            .stdout(predicate::str::contains(
                "fail    svc/tests/a.test.js::SVC-001 works  SVC-001",
            ))
            .stdout(predicate::str::contains("boom"));
    }
}
