//! CONF-011 from openspec/changes/project-config: `sj doctor`.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A project with both spec-root folders and the given packages.
fn project(packages: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "specjutsu.toml",
        &format!("schema = 1\n{packages}"),
    );
    std::fs::create_dir_all(dir.path().join("openspec/specs")).unwrap();
    std::fs::create_dir_all(dir.path().join("openspec/changes")).unwrap();
    dir
}

const WORKER: &str = "\n[[package]]\npath = \"python/worker\"\nsources = []\n\n\
                      [[package.suite]]\nrunner = \"pytest\"\ntests = []\n";

/// `sj doctor` with PATH set to `path` only.
fn doctor(root: &Path, path: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["doctor", "--root"])
        .arg(root)
        .env("PATH", path)
        .assert()
}

/// A fake executable `name` in `dir`.
fn fake_exe(dir: &Path, name: &str) {
    if cfg!(windows) {
        write(dir, &format!("{name}.cmd"), "@echo off\r\n");
    } else {
        write(dir, name, "#!/bin/sh\nexit 0\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let p = dir.join(name);
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

#[test]
fn conf_011_doctor_reports_a_missing_runner_executable() {
    let dir = project(WORKER);
    let empty = tempfile::tempdir().unwrap();
    doctor(dir.path(), empty.path())
        .code(1)
        .stdout(predicate::str::contains(
            "missing runner pytest (python/worker)",
        ));

    let bin = tempfile::tempdir().unwrap();
    fake_exe(bin.path(), "pytest");
    doctor(dir.path(), bin.path())
        .success()
        .stdout("doctor: ok\n");
}

#[test]
fn conf_011_node_runners_are_found_in_node_modules_bin() {
    let dir = project(
        "\n[[package]]\npath = \"apps/api\"\nsources = []\n\n\
         [[package.suite]]\nrunner = \"vitest\"\ntests = []\n",
    );
    let empty = tempfile::tempdir().unwrap();
    doctor(dir.path(), empty.path())
        .code(1)
        .stdout(predicate::str::contains("missing runner vitest (apps/api)"));

    // Hoisted to the workspace root, as pnpm and npm do.
    fake_exe(&dir.path().join("node_modules/.bin"), "vitest");
    doctor(dir.path(), empty.path()).success();
}

#[test]
fn conf_011_doctor_lists_every_problem_and_passes_a_clean_setup() {
    let clean = project("");
    let empty = tempfile::tempdir().unwrap();
    doctor(clean.path(), empty.path())
        .success()
        .stdout("doctor: ok\n");

    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \"packages/ui\"\nsources = []\n",
    );
    doctor(dir.path(), empty.path())
        .code(1)
        .stdout(predicate::str::starts_with("doctor: 3 problems\n"))
        .stdout(predicate::str::contains("no runner: packages/ui"))
        .stdout(predicate::str::contains(
            "missing spec root: specjutsu/specs",
        ))
        .stdout(predicate::str::contains(
            "missing spec root: specjutsu/changes",
        ));
}

#[test]
fn conf_011_a_bad_config_is_a_finding_not_an_environment_error() {
    let dir = project("spec_rot = \"x\"\n");
    let empty = tempfile::tempdir().unwrap();
    doctor(dir.path(), empty.path())
        .code(1)
        .stdout(predicate::str::starts_with("doctor: 1 problem\n"))
        .stdout(predicate::str::contains("spec_rot"));
}
