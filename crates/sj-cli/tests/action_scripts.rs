//! CI-007..013 from openspec/changes/gate-action: the action's scripts, run with a stub `gh`.
//! The scripts are bash; these tests run on Linux and macOS only.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn script(name: &str) -> PathBuf {
    repo_root().join("scripts/action").join(name)
}

fn write_exe(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A `gh` that serves `release download --dir D --pattern P` from `$STUB_RELEASE`.
const GH_RELEASE: &str = r#"#!/usr/bin/env bash
set -euo pipefail
if [ "$1 $2" = "release download" ]; then
  shift 2
  dir=. ; pats=()
  while [ $# -gt 0 ]; do
    case "$1" in
      --dir) dir="$2"; shift 2 ;;
      --pattern) pats+=("$2"); shift 2 ;;
      *) shift ;;
    esac
  done
  for p in "${pats[@]}"; do cp "$STUB_RELEASE/$p" "$dir/"; done
  exit 0
fi
echo "unexpected gh call: $*" >&2; exit 9
"#;

fn run(cmd: &mut Command) -> Output {
    cmd.output().unwrap()
}

#[test]
fn ci_007_a_tampered_download_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let target = "x86_64-unknown-linux-gnu";
    let marker = t.join("sj-ran");

    // Release assets: an archive whose sj would leave a marker, and a SHA256SUMS that lies.
    let pkg = t.join("pkg").join(format!("sj-{target}"));
    write_exe(
        &pkg.join("sj"),
        &format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    );
    let release = t.join("release");
    std::fs::create_dir_all(&release).unwrap();
    let asset = format!("sj-{target}.tar.gz");
    assert!(run(Command::new("tar")
        .arg("-czf")
        .arg(release.join(&asset))
        .arg("-C")
        .arg(t.join("pkg"))
        .arg(format!("sj-{target}")))
    .status
    .success());
    std::fs::write(
        release.join("SHA256SUMS"),
        format!("{}  {asset}\n", "0".repeat(64)),
    )
    .unwrap();
    write_exe(&t.join("bin/gh"), GH_RELEASE);

    let out = run(Command::new("bash")
        .arg(script("install.sh"))
        .arg("0.4.0")
        .env(
            "PATH",
            format!(
                "{}:{}",
                t.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("STUB_RELEASE", &release)
        .env("SJ_TARGET", target)
        .env("SJ_INSTALL_DIR", t.join("install"))
        .env("GITHUB_PATH", t.join("github_path")));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "install succeeded: {stderr}");
    assert!(
        stderr.contains(&format!("checksum mismatch for {asset}")),
        "{stderr}"
    );
    assert!(!marker.exists(), "sj ran despite the checksum mismatch");
    assert!(!t.join("install/sj").exists());
}
