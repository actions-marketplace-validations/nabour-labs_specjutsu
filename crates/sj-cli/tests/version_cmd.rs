use std::process::Command as Std;

use assert_cmd::Command;

#[test]
fn conf_030_build_from_checkout_shows_commit() {
    let head = Std::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git runs");
    assert!(head.status.success(), "the test runs inside a git checkout");
    let commit = String::from_utf8(head.stdout).unwrap().trim().to_string();
    let expected = format!("sj {} ({commit})\n", env!("CARGO_PKG_VERSION"));

    Command::cargo_bin("sj")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(expected);
}
