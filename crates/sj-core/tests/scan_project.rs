//! TRACE-007 and TRACE-008 from openspec/changes/polyglot-trace: finding test files per suite.

use std::path::Path;

use sj_core::config::{Config, Runner};
use sj_core::testscan::{scan_project, Scan};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = r#"
schema = 1

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = []

[[package]]
path = "apps/app"
sources = ["apps/app/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/**/*.{test,spec}.{ts,tsx}"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]

[[package]]
path = "python/worker"
sources = ["python/worker/**"]

[[package.suite]]
runner = "pytest"
tests = []

[[package]]
path = "packages/ui"
sources = ["packages/ui/src/**"]

[[package]]
path = "packages/empty"
sources = ["packages/empty/src/**"]
"#;

fn polyglot() -> (tempfile::TempDir, Scan) {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(
        r,
        "apps/api/src/auth.test.ts",
        "it(\"AUTH-003 rejects\", () => {})\n",
    );
    write(
        r,
        "apps/api/tests/unit/models.test.ts",
        "test(\"models\", () => {})\n",
    );
    write(
        r,
        "apps/api/src/helpers.ts",
        "test(\"not a test file\", () => {})\n",
    );
    write(
        r,
        "apps/app/src/view.test.tsx",
        "it(\"renders\", () => {})\n",
    );
    write(
        r,
        "apps/app/e2e/login.spec.ts",
        "test(\"logs in\", async () => {})\n",
    );
    write(
        r,
        "python/worker/tests/test_jobs.py",
        "def test_runs():\n    pass\n",
    );
    write(
        r,
        "python/worker/jobs.py",
        "def test_helper_not_collected():\n    pass\n",
    );
    write(r, "scripts/seed.ts", "test(\"AUTH-003 x\", () => {})\n");
    write(
        r,
        "apps/api/node_modules/lib/x.test.ts",
        "it(\"vendored\", () => {})\n",
    );
    write(
        r,
        "packages/ui/src/button.tsx",
        "export const Button = () => null\n",
    );
    let config = Config::parse(CONFIG).unwrap();
    let scan = scan_project(r, &config).unwrap();
    (dir, scan)
}

fn found(scan: &Scan) -> Vec<(String, String, Option<String>, Option<Runner>)> {
    let mut v: Vec<_> = scan
        .tests
        .iter()
        .map(|t| {
            let file = t.file.to_string_lossy().replace('\\', "/");
            (file, t.name.clone(), t.package.clone(), t.suite)
        })
        .collect();
    v.sort();
    v
}

fn entry(
    file: &str,
    name: &str,
    pkg: &str,
    suite: Runner,
) -> (String, String, Option<String>, Option<Runner>) {
    (file.into(), name.into(), Some(pkg.into()), Some(suite))
}

#[test]
fn trace_007_co_located_and_tests_folder_files_are_both_found() {
    let (_dir, scan) = polyglot();
    assert_eq!(
        found(&scan),
        [
            entry(
                "apps/api/src/auth.test.ts",
                "AUTH-003 rejects",
                "apps/api",
                Runner::Vitest
            ),
            entry(
                "apps/api/tests/unit/models.test.ts",
                "models",
                "apps/api",
                Runner::Vitest
            ),
            entry(
                "apps/app/e2e/login.spec.ts",
                "logs in",
                "apps/app",
                Runner::Playwright
            ),
            entry(
                "apps/app/src/view.test.tsx",
                "renders",
                "apps/app",
                Runner::Vitest
            ),
            entry(
                "python/worker/tests/test_jobs.py",
                "test_runs",
                "python/worker",
                Runner::Pytest
            ),
        ]
    );
}

#[test]
fn trace_007_rust_tests_inline_and_under_tests_are_found_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "Cargo.toml", "[workspace]\n");
    write(
        r,
        "src/lib.rs",
        "#[cfg(test)]\nmod t {\n    #[test]\n    fn inline_one() {}\n}\n",
    );
    write(r, "tests/it.rs", "#[test]\nfn outer_one() {}\n");
    write(
        r,
        "fixtures/demo/src/lib.rs",
        "#[test]\nfn fixture_one() {}\n",
    );
    let config = Config::default_for(r);
    assert_eq!(config.packages[0].suites[0].tests, ["**/*.rs"]);
    let scan = scan_project(r, &config).unwrap();
    let names: Vec<_> = found(&scan).into_iter().map(|e| e.1).collect();
    assert_eq!(names, ["inline_one", "outer_one"]);
    assert!(scan.tests.iter().all(|t| t.suite == Some(Runner::Cargo)));
}

#[test]
fn trace_007_playwright_defaults_to_its_test_dir() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(
        r,
        "playwright.config.ts",
        "export default { testDir: './tests/e2e' }\n",
    );
    write(
        r,
        "tests/e2e/chat.spec.ts",
        "test(\"chats\", async () => {})\n",
    );
    write(
        r,
        "src/other.spec.ts",
        "test(\"not e2e\", async () => {})\n",
    );
    let config = Config::parse(
        "schema = 1\n\n[[package]]\npath = \".\"\nsources = []\n\n\
         [[package.suite]]\nrunner = \"playwright\"\ntests = []\n",
    )
    .unwrap();
    let names: Vec<_> = found(&scan_project(r, &config).unwrap())
        .into_iter()
        .map(|e| e.1)
        .collect();
    assert_eq!(names, ["chats"]);
}

#[test]
fn trace_008_files_outside_every_suite_are_not_scanned() {
    let (_dir, scan) = polyglot();
    let files: Vec<_> = found(&scan).into_iter().map(|e| e.0).collect();
    assert!(
        !files.iter().any(|f| f.starts_with("scripts/")),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|f| f.contains("node_modules")),
        "{files:?}"
    );
    assert!(!files
        .iter()
        .any(|f| f.ends_with("/helpers.ts") || f.ends_with("/jobs.py")));
    assert_eq!(scan.unsuited, ["packages/ui"]);
}
