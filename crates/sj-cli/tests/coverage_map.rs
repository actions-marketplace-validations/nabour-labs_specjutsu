//! DRIFT-002..004 from specjutsu/changes/spec-drift: `sj coverage map` with a fixture coverage
//! command that copies canned LCOV files, so no coverage tool is needed.

use std::path::{Path, PathBuf};
use std::process::Command as Std;

use assert_cmd::Command;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Std::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// LCOV for `file` with `ranges` hit once, plus one line that was not hit and a test file
/// that is not a source.
fn lcov(file: &str, ranges: &[(usize, usize)]) -> String {
    let mut s = format!("TN:\nSF:{file}\n");
    for (a, b) in ranges {
        for l in *a..=*b {
            s.push_str(&format!("DA:{l},1\n"));
        }
    }
    s.push_str("DA:999,0\nend_of_record\nSF:tests/cov.rs\nDA:1,1\nend_of_record\n");
    s
}

struct Project {
    dir: tempfile::TempDir,
    log: PathBuf,
    _logdir: tempfile::TempDir,
}

impl Project {
    /// `scenarios`: (ID, LCOV body). Each scenario gets one test, `t_<n>`, whose coverage is
    /// the LCOV body.
    fn new(scenarios: &[(&str, String)]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let logdir = tempfile::tempdir().unwrap();
        let log = logdir.path().join("runs.log");
        let r = dir.path();
        let copy = if cfg!(windows) {
            format!(
                "copy /Y lcov\\{{tests}}.lcov {{lcov}} >NUL && echo run>> \"{}\"",
                log.display()
            )
        } else {
            format!(
                "cp lcov/{{tests}}.lcov {{lcov}} && echo run >> '{}'",
                log.display()
            )
        };
        write(
            r,
            "specjutsu.toml",
            &format!(
                "schema = 1\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n[[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\ncoverage = '''{copy}'''\n"
            ),
        );
        write(r, ".gitignore", ".specjutsu/\n");
        write(r, "src/lib.rs", "// lib\n");
        let mut spec =
            String::from("## Requirements\n\n### Requirement: R\nThe system SHALL r.\n\n");
        let mut tests = String::new();
        for (n, (id, body)) in scenarios.iter().enumerate() {
            spec.push_str(&format!(
                "#### Scenario: {id} Case\n- **WHEN** x\n- **THEN** y\n\n"
            ));
            tests.push_str(&format!(
                "// spec: {id}\n#[test]\nfn t_{n}() {{ assert!(true); }}\n\n"
            ));
            write(r, &format!("lcov/t_{n}.lcov"), body);
        }
        write(r, "openspec/specs/r/spec.md", &spec);
        write(r, "tests/cov.rs", &tests);
        git(r, &["init", "-q", "-b", "main"]);
        git(r, &["add", "-A"]);
        git(r, &["commit", "-qm", "init"]);
        Project {
            dir,
            log,
            _logdir: logdir,
        }
    }

    fn map(&self) -> String {
        let out = Command::cargo_bin("sj")
            .unwrap()
            .args(["coverage", "map", "--root"])
            .arg(self.dir.path())
            .output()
            .unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.status.success(), "{text}");
        text
    }

    fn json(&self) -> serde_json::Value {
        let text =
            std::fs::read_to_string(self.dir.path().join(".specjutsu/coverage-map.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn runs(&self) -> usize {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .count()
    }
}

#[test]
fn drift_002_the_map_holds_each_scenario_s_lines() {
    let p = Project::new(&[
        ("AUTH-003", lcov("src/auth.rs", &[(10, 30)])),
        ("AUTH-004", lcov("src/auth.rs", &[(40, 55)])),
    ]);
    p.map();
    let v = p.json();
    assert_eq!(
        v["scenarios"]["AUTH-003"]["src/auth.rs"],
        serde_json::json!([[10, 30]])
    );
    assert_eq!(
        v["scenarios"]["AUTH-004"]["src/auth.rs"],
        serde_json::json!([[40, 55]])
    );
    assert!(
        v["scenarios"]["AUTH-003"].get("tests/cov.rs").is_none(),
        "a test file is not a source: {v}"
    );
    let head = git(p.dir.path(), &["rev-parse", "HEAD"]);
    assert_eq!(v["commit"], head.trim());
}

#[test]
fn drift_003_lines_most_scenarios_run_are_shared_not_owned() {
    let scenarios: Vec<(String, String)> = (1..=10)
        .map(|n| {
            let own = (100 + n * 10, 105 + n * 10);
            let ranges = if n <= 8 { vec![(1, 9), own] } else { vec![own] };
            (format!("LIB-{n:03}"), lcov("src/lib.rs", &ranges))
        })
        .collect();
    let refs: Vec<(&str, String)> = scenarios
        .iter()
        .map(|(a, b)| (a.as_str(), b.clone()))
        .collect();
    let p = Project::new(&refs);
    let out = p.map();
    assert!(out.contains("9 shared lines dropped"), "{out}");
    let v = p.json();
    for n in 1..=10 {
        let ranges = &v["scenarios"][format!("LIB-{n:03}")]["src/lib.rs"];
        let own = 100 + n * 10;
        assert_eq!(
            ranges,
            &serde_json::json!([[own, own + 5]]),
            "LIB-{n:03}: {v}"
        );
    }
}

#[test]
fn drift_004_an_up_to_date_map_is_not_rebuilt() {
    let p = Project::new(&[
        ("AUTH-003", lcov("src/auth.rs", &[(10, 30)])),
        ("AUTH-004", lcov("src/auth.rs", &[(40, 55)])),
    ]);
    p.map();
    assert_eq!(p.runs(), 2);
    let out = p.map();
    assert!(out.contains("coverage map up to date"), "{out}");
    assert_eq!(p.runs(), 2, "the second run ran tests");
}
