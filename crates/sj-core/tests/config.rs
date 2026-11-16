//! CONF scenarios from openspec/changes/project-config: the project file model and loader.

use sj_core::config::{self, Config, Mode, Runner};

const VALID: &str = r#"
schema = 1
spec_root = "openspec"
mode = "guided"

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/api/**/*.test.ts"]

[[package]]
path = "python/worker"
sources = ["python/worker/src/**"]

[[package.suite]]
runner = "pytest"
tests = ["python/worker/**/test_*.py"]
"#;

fn load_text(text: &str) -> anyhow::Result<Config> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(config::FILE), text).unwrap();
    config::load(dir.path())
}

fn err_text(text: &str) -> String {
    format!("{:#}", load_text(text).unwrap_err())
}

#[test]
fn conf_001_a_valid_project_file_is_loaded() {
    let c = load_text(VALID).unwrap();
    assert_eq!(c.schema, 1);
    assert_eq!(c.spec_root, "openspec");
    assert_eq!(c.mode, Mode::Guided);
    let paths: Vec<_> = c.packages.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, ["apps/api", "python/worker"]);

    let api = &c.packages[0];
    assert_eq!(api.sources, ["apps/api/src/**"]);
    assert_eq!(api.suites.len(), 1);
    assert_eq!(api.suites[0].runner, Runner::Vitest);
    assert_eq!(api.suites[0].tests, ["apps/api/**/*.test.ts"]);

    let worker = &c.packages[1];
    assert_eq!(worker.suites[0].runner, Runner::Pytest);
    assert_eq!(worker.suites[0].tests, ["python/worker/**/test_*.py"]);
}

#[test]
fn conf_001_a_package_may_hold_several_suites() {
    let c = load_text(
        r#"
schema = 1

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/**/*.test.ts"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
"#,
    )
    .unwrap();
    let runners: Vec<_> = c.packages[0].suites.iter().map(|s| s.runner).collect();
    assert_eq!(runners, [Runner::Vitest, Runner::Playwright]);
    // Omitted top-level keys take their defaults.
    assert_eq!(c.spec_root, "openspec");
    assert_eq!(c.mode, Mode::Guided);
}

#[test]
fn conf_002_missing_project_file_falls_back_to_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let c = config::load(dir.path()).unwrap();
    assert_eq!(c.spec_root, "openspec");
    assert_eq!(c.mode, Mode::Guided);
    assert!(c.packages.is_empty());

    std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    let c = config::load(dir.path()).unwrap();
    assert_eq!(c.packages.len(), 1);
    assert_eq!(c.packages[0].path, ".");
    assert_eq!(c.packages[0].suites.len(), 1);
    assert_eq!(c.packages[0].suites[0].runner, Runner::Cargo);
}

#[test]
fn conf_003_an_invalid_project_file_is_an_environment_error() {
    let e = err_text(
        r#"
schema = 1

[[package]]
path = "apps/web"
sources = ["apps/web/src/**"]

[[package.suite]]
runner = "jestt"
tests = ["apps/web/**/*.test.ts"]
"#,
    );
    assert!(e.contains("specjutsu.toml"), "{e}");
    assert!(e.contains("apps/web"), "{e}");
    assert!(e.contains("jestt"), "{e}");
    assert!(
        e.contains("cargo, jest, junit, playwright, pytest, vitest"),
        "{e}"
    );
}

#[test]
fn conf_003_paths_and_schema_are_validated() {
    let pkg = |path: &str| format!("schema = 1\n\n[[package]]\npath = \"{path}\"\nsources = []\n");
    for bad in ["../x", "apps\\web", "./apps", "/apps", "apps/"] {
        let e = err_text(&pkg(bad));
        assert!(e.contains("specjutsu.toml"), "{bad}: {e}");
        assert!(e.contains("path"), "{bad}: {e}");
    }
    let dup = format!(
        "{}\n{}",
        pkg("apps"),
        pkg("apps").replace("schema = 1\n", "")
    );
    assert!(err_text(&dup).contains("duplicate"), "{}", err_text(&dup));
    assert!(err_text("schema = 2\n").contains("schema"));
    assert!(err_text("spec_root = \"openspec\"\n").contains("schema"));
}

#[test]
fn conf_004_unknown_keys_are_rejected() {
    let e = err_text("schema = 1\nspec_rot = \"specs\"\n");
    assert!(e.contains("specjutsu.toml"), "{e}");
    assert!(e.contains("spec_rot"), "{e}");

    let e = err_text("schema = 1\n\n[[package]]\npath = \"a\"\nsource = []\n");
    assert!(e.contains("source"), "{e}");
}

#[test]
fn conf_005_nested_packages_resolve_to_the_deepest_one() {
    let c = load_text(
        "schema = 1\n\n[[package]]\npath = \"apps\"\nsources = []\n\n\
         [[package]]\npath = \"apps/api\"\nsources = []\n",
    )
    .unwrap();
    let owner = |f: &str| c.package_for(f).map(|p| p.path.as_str());
    assert_eq!(owner("apps/api/src/x.ts"), Some("apps/api"));
    assert_eq!(owner("apps/admin/src/y.ts"), Some("apps"));
    assert_eq!(owner("apps-admin/z.ts"), None);
    assert_eq!(owner("README.md"), None);

    let rooted = load_text(
        "schema = 1\n\n[[package]]\npath = \".\"\nsources = []\n\n\
         [[package]]\npath = \"apps/api\"\nsources = []\n",
    )
    .unwrap();
    assert_eq!(rooted.package_for("README.md").unwrap().path, ".");
    assert_eq!(
        rooted.package_for("apps/api/a.ts").unwrap().path,
        "apps/api"
    );
}

#[test]
fn conf_006_the_written_file_parses_back_to_the_same_config() {
    let c = load_text(VALID).unwrap();
    assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    let d = Config::default_for(dir.path());
    assert_eq!(Config::parse(&d.to_toml()).unwrap(), d);

    let quoted = Config {
        spec_root: "spec \"root\" x".into(),
        ..Config::default_for(dir.path())
    };
    assert_eq!(Config::parse(&quoted.to_toml()).unwrap(), quoted);
}

#[test]
fn conf_006_a_config_without_packages_is_three_lines() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        Config::default_for(dir.path()).to_toml(),
        "schema = 1\nspec_root = \"openspec\"\nmode = \"guided\"\n"
    );
}

#[test]
fn conf_003_junit_suites_need_a_command_and_a_report() {
    let suite = |body: &str| {
        format!(
            "schema = 1\n\n[[package]]\npath = \"svc\"\nsources = []\n\n[[package.suite]]\n{body}"
        )
    };
    let e = err_text(&suite("runner = \"junit\"\nreport = \"r.xml\"\n"));
    assert!(e.contains("svc") && e.contains("command"), "{e}");
    let e = err_text(&suite("runner = \"junit\"\ncommand = \"make test\"\n"));
    assert!(e.contains("svc") && e.contains("report"), "{e}");
    let e = err_text(&suite("runner = \"vitest\"\nreport = \"r.xml\"\n"));
    assert!(e.contains("report") && e.contains("junit"), "{e}");

    let c = load_text(&suite(
        "runner = \"junit\"\ncommand = \"make test\"\nreport = \"build/junit.xml\"\nenv = { CI = \"1\" }\n",
    ))
    .unwrap();
    assert_eq!(c.packages[0].suites[0].runner, Runner::Junit);
    assert_eq!(
        c.packages[0].suites[0].command.as_deref(),
        Some("make test")
    );
    assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);
}

const GATE_KEYS: &str = r#"schema = 1
non_source = ["infra/**", "*.tf"]

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/src/**/*.test.ts"]
setup = "pnpm install --frozen-lockfile"
support = ["apps/app/test/**"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
red_on_base = "skip"
reason = "needs staging API"
"#;

#[test]
fn gate_034_suites_declare_setup_support_and_a_red_on_base_policy() {
    let c = load_text(GATE_KEYS).unwrap();
    let [vitest, pw] = &c.packages[0].suites[..] else {
        panic!("two suites")
    };
    assert_eq!(
        vitest.setup.as_deref(),
        Some("pnpm install --frozen-lockfile")
    );
    assert_eq!(vitest.support, vec!["apps/app/test/**".to_string()]);
    assert_eq!(vitest.red_on_base, config::RedOnBase::Run);
    assert_eq!(pw.red_on_base, config::RedOnBase::Skip);
    assert_eq!(pw.reason.as_deref(), Some("needs staging API"));
    assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);
}

#[test]
fn gate_034_skip_needs_a_reason_and_a_reason_needs_skip() {
    let suite = |body: &str| {
        format!(
            "schema = 1\n\n[[package]]\npath = \"web\"\nsources = []\n\n[[package.suite]]\n\
             runner = \"playwright\"\ntests = []\n{body}"
        )
    };
    let e = err_text(&suite("red_on_base = \"skip\"\n"));
    assert!(e.contains("web") && e.contains("reason"), "{e}");
    let e = err_text(&suite("reason = \"slow\"\n"));
    assert!(e.contains("reason") && e.contains("skip"), "{e}");
    let e = err_text(&suite("red_on_base = \"sometimes\"\n"));
    assert!(e.contains("sometimes") && e.contains("run, skip"), "{e}");
}

#[test]
fn gate_038_non_source_is_a_top_level_list() {
    let c = load_text(GATE_KEYS).unwrap();
    assert_eq!(c.non_source, vec!["infra/**".to_string(), "*.tf".into()]);
    let dir = tempfile::tempdir().unwrap();
    assert!(Config::default_for(dir.path()).non_source.is_empty());
}
