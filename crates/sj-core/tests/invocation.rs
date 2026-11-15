//! RUN-014 and RUN-015 from openspec/changes/polyglot-runners: the command each suite runs.

use std::path::{Path, PathBuf};

use sj_core::config::Config;
use sj_core::run::suite::{invocation, Invocation, Pick};
use sj_core::testscan::TestFn;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn suite_config(path: &str, runner: &str, extra: &str) -> Config {
    Config::parse(&format!(
        "schema = 1\n\n[[package]]\npath = \"{path}\"\nsources = []\n\n\
         [[package.suite]]\nrunner = \"{runner}\"\ntests = []\n{extra}"
    ))
    .unwrap()
}

fn inv(root: &Path, config: &Config, pick: &Pick) -> Invocation {
    let p = &config.packages[0];
    invocation(root, p, &p.suites[0], pick, Path::new("/tmp/r.xml")).unwrap()
}

fn test(file: &str, name: &str, line: usize) -> TestFn {
    TestFn {
        name: name.into(),
        file: PathBuf::from(file),
        line,
        ..TestFn::default()
    }
}

const VITEST_ARGS: [&str; 4] = [
    "run",
    "--reporter=default",
    "--reporter=junit",
    "--outputFile.junit=/tmp/r.xml",
];

#[test]
fn run_015_node_runners_use_the_projects_package_manager() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let c = suite_config("apps/api", "vitest", "");

    // npm (no lockfile): npx without installing.
    let i = inv(root, &c, &Pick::All);
    assert_eq!(i.program, "npx");
    assert_eq!(i.args[..2], ["--no-install", "vitest"]);
    assert_eq!(i.args[2..], VITEST_ARGS);
    assert_eq!(i.cwd, root.join("apps/api"));

    // A yarn lockfile in the package wins over npm.
    write(root, "apps/api/yarn.lock", "");
    let i = inv(root, &c, &Pick::All);
    assert_eq!((i.program.as_str(), i.args[0].as_str()), ("yarn", "vitest"));

    // The nearest lockfile wins: yarn in the package beats pnpm at the root.
    write(root, "pnpm-lock.yaml", "");
    let i = inv(root, &c, &Pick::All);
    assert_eq!(i.program, "yarn");
    std::fs::remove_file(root.join("apps/api/yarn.lock")).unwrap();
    let i = inv(root, &c, &Pick::All);
    assert_eq!(i.program, "pnpm");
    assert_eq!(i.args[..2], ["exec", "vitest"]);
}

#[test]
fn run_015_vitest_and_jest_select_files_and_titles() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "pnpm-lock.yaml", "");
    let c = suite_config("apps/api", "vitest", "");
    let pick = Pick::Tests(vec![
        test(
            "apps/api/src/auth.test.ts",
            "AUTH-004 refresh > rejects expired token",
            3,
        ),
        test("apps/api/src/auth.test.ts", "case %i", 6),
        test("apps/api/src/x.test.ts", "a (b)", 1),
    ]);
    let i = inv(root, &c, &pick);
    assert_eq!(
        i.args[2..],
        [
            &VITEST_ARGS[..],
            &[
                "src/auth.test.ts",
                "src/x.test.ts",
                "-t",
                r"rejects expired token|case .*|a \(b\)"
            ]
        ]
        .concat()
    );

    let c = suite_config("apps/web", "jest", "");
    let i = inv(root, &c, &Pick::All);
    assert_eq!(i.program, "pnpm");
    assert_eq!(
        i.args,
        [
            "exec",
            "jest",
            "--ci",
            "--reporters=default",
            "--reporters=jest-junit"
        ]
    );
    assert_eq!(i.env["JEST_JUNIT_OUTPUT_FILE"], "/tmp/r.xml");
    assert_eq!(i.env["JEST_JUNIT_ADD_FILE_ATTRIBUTE"], "true");
    assert_eq!(i.env["JEST_JUNIT_CLASSNAME"], "{classname}");
    assert_eq!(i.env["JEST_JUNIT_TITLE"], "{title}");
    assert_eq!(i.env["JEST_JUNIT_ANCESTOR_SEPARATOR"], " > ");
}

#[test]
fn run_015_playwright_selects_by_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "pnpm-lock.yaml", "");
    let c = suite_config("apps/app", "playwright", "");
    let pick = Pick::Tests(vec![test("apps/app/e2e/chat.spec.ts", "chat > sends", 4)]);
    let i = inv(root, &c, &pick);
    assert_eq!(
        i.args,
        [
            "exec",
            "playwright",
            "test",
            "--reporter=junit",
            "e2e/chat.spec.ts:4"
        ]
    );
    assert_eq!(i.env["PLAYWRIGHT_JUNIT_OUTPUT_NAME"], "/tmp/r.xml");
}

#[test]
fn run_015_pytest_in_a_uv_project_runs_through_uv() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let c = suite_config("backend", "pytest", "");
    let pytest_args = ["--junitxml=/tmp/r.xml", "-o", "junit_family=xunit1"];

    let i = inv(root, &c, &Pick::All);
    let python = if cfg!(windows) { "python" } else { "python3" };
    assert_eq!(i.program, python);
    assert_eq!(i.args, [&["-m", "pytest"][..], &pytest_args[..]].concat());

    write(root, "backend/uv.lock", "");
    let pick = Pick::Tests(vec![
        test("backend/tests/test_inv.py", "TestDiff::test_added", 9),
        test("backend/tests/test_inv.py", "test_inv_002_removes_item", 3),
    ]);
    let i = inv(root, &c, &pick);
    assert_eq!(i.program, "uv");
    assert_eq!(i.cwd, root.join("backend"));
    assert_eq!(
        i.args,
        [
            &["run", "pytest"][..],
            &pytest_args[..],
            &[
                "tests/test_inv.py::TestDiff::test_added",
                "tests/test_inv.py::test_inv_002_removes_item"
            ]
        ]
        .concat()
    );

    // A uv workspace lock at the root counts too.
    std::fs::remove_file(root.join("backend/uv.lock")).unwrap();
    write(root, "uv.lock", "");
    assert_eq!(inv(root, &c, &Pick::All).program, "uv");
}

#[test]
fn run_014_the_generic_junit_runner_runs_any_command() {
    let dir = tempfile::tempdir().unwrap();
    let c = suite_config(
        "svc",
        "junit",
        "command = \"make test\"\nreport = \"build/junit.xml\"\n",
    );
    let i = inv(dir.path(), &c, &Pick::All);
    if cfg!(windows) {
        assert_eq!(
            (i.program.as_str(), &i.args[..]),
            ("cmd", &["/C".to_string(), "make test".into()][..])
        );
    } else {
        assert_eq!(
            (i.program.as_str(), &i.args[..]),
            ("sh", &["-c".to_string(), "make test".into()][..])
        );
    }
    assert_eq!(i.cwd, dir.path().join("svc"));
    let p = &c.packages[0];
    assert_eq!(
        sj_core::run::suite::declared_report(dir.path(), p, &p.suites[0]),
        Some(dir.path().join("svc/build/junit.xml"))
    );
}

#[test]
fn run_014_command_override_and_env() {
    let dir = tempfile::tempdir().unwrap();
    let c = suite_config(
        "apps/api",
        "vitest",
        "command = \"npm run test --\"\nenv = { CI = \"1\", TZ = \"UTC\" }\n",
    );
    let i = inv(dir.path(), &c, &Pick::All);
    assert_eq!(i.program, "npm");
    assert_eq!(
        i.args,
        [&["run", "test", "--"][..], &VITEST_ARGS[..]].concat()
    );
    assert_eq!(i.env["CI"], "1");
    assert_eq!(i.env["TZ"], "UTC");
}
