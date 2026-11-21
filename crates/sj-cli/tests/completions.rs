//! `sj completions <shell>` (openspec/changes/cli-conveniences): scripts generated from the CLI.

use assert_cmd::Command;

fn completions(dir: &std::path::Path, shell: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sj")
        .unwrap()
        .current_dir(dir)
        .args(["completions", shell])
        .assert()
}

fn stdout(a: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(a.get_output().stdout.clone()).unwrap()
}

// spec: CONF-023
#[test]
fn conf_023_completion_script_for_zsh() {
    let dir = tempfile::tempdir().unwrap();
    let a = completions(dir.path(), "zsh").success();
    let out = stdout(&a);
    assert!(out.starts_with("#compdef sj"), "{out}");
    for sub in ["trace", "gate", "spec"] {
        assert!(out.contains(sub), "missing {sub}");
    }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

// spec: CONF-024
#[test]
fn conf_024_every_supported_shell_gets_a_script() {
    let dir = tempfile::tempdir().unwrap();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let a = completions(dir.path(), shell).success();
        let out = stdout(&a);
        assert!(!out.is_empty(), "{shell}: empty");
        assert!(out.contains("verify"), "{shell}: no verify");
    }
}

// spec: CONF-025
#[test]
fn conf_025_unknown_shell_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let a = completions(dir.path(), "tcsh").code(2);
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    assert!(err.contains("bash, zsh, fish, powershell, elvish"), "{err}");
}
