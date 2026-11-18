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

/// A `gh api` that keeps comments as files in `$STUB_STATE/comments/<id>`, logs every call to
/// `$STUB_STATE/calls`, and refuses writes with HTTP 403 when `STUB_READONLY=1`.
const GH_API: &str = r#"#!/usr/bin/env bash
set -euo pipefail
state="$STUB_STATE"; mkdir -p "$state/comments"
echo "$*" >> "$state/calls"
[ "$1" = api ] || { echo "unexpected gh call: $*" >&2; exit 9; }
shift
method=GET; path=""; body=""
while [ $# -gt 0 ]; do
  case "$1" in
    -X) method="$2"; shift 2 ;;
    -F|-f) case "$2" in body=@*) body="${2#body=@}" ;; esac; shift 2 ;;
    --paginate) shift ;;
    --jq) shift 2 ;;
    *) path="$1"; shift ;;
  esac
done
if [ "$method" != GET ] && [ "${STUB_READONLY:-}" = 1 ]; then
  echo "gh: Resource not accessible by integration (HTTP 403)" >&2; exit 1
fi
case "$method $path" in
  "GET repos/"*/issues/*/comments)
    for f in "$state"/comments/*; do
      [ -e "$f" ] || continue
      if [ "$(head -n1 "$f")" = "<!-- specjutsu:gate -->" ]; then basename "$f"; fi
    done ;;
  "POST repos/"*/issues/*/comments)
    n=$(( $(ls "$state/comments" | wc -l) + 1 )); cp "$body" "$state/comments/$n" ;;
  "PATCH repos/"*/issues/comments/*)
    cp "$body" "$state/comments/${path##*/}" ;;
  *) echo "unexpected gh api: $method $path" >&2; exit 9 ;;
esac
"#;

/// An `sj` that prints `$STUB_BODY` and exits `$STUB_GATE_EXIT`.
const SJ_STUB: &str = r#"#!/usr/bin/env bash
printf '%s\n' "$STUB_BODY"
exit "${STUB_GATE_EXIT:-0}"
"#;

struct Action {
    dir: tempfile::TempDir,
}

impl Action {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        write_exe(&dir.path().join("bin/gh"), GH_API);
        write_exe(&dir.path().join("bin/sj"), SJ_STUB);
        std::fs::create_dir_all(dir.path().join("state/comments")).unwrap();
        Action { dir }
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn summary(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("summary.md")).unwrap_or_default()
    }

    fn calls(&self) -> String {
        std::fs::read_to_string(self.state().join("calls")).unwrap_or_default()
    }

    /// Bodies of the comments on the PR, in id order.
    fn comments(&self) -> Vec<String> {
        let mut ids: Vec<u32> = std::fs::read_dir(self.state().join("comments"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().parse().unwrap())
            .collect();
        ids.sort();
        ids.iter()
            .map(|id| std::fs::read_to_string(self.state().join(format!("comments/{id}"))).unwrap())
            .collect()
    }

    fn gate_comments(&self) -> Vec<String> {
        self.comments()
            .into_iter()
            .filter(|c| c.starts_with("<!-- specjutsu:gate -->"))
            .collect()
    }

    fn run(&self, body: &str, gate_exit: i32, env: &[(&str, &str)]) -> Output {
        let bin = self.dir.path().join("bin");
        let mut cmd = Command::new("bash");
        cmd.arg(script("comment.sh"))
            .env(
                "PATH",
                format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
            )
            .env("STUB_STATE", self.state())
            .env("STUB_BODY", body)
            .env("STUB_GATE_EXIT", gate_exit.to_string())
            .env("GITHUB_STEP_SUMMARY", self.dir.path().join("summary.md"))
            .env("GITHUB_REPOSITORY", "acme/shop")
            .env("SJ_BASE", "origin/main")
            .env("SJ_PR", "7")
            .env("SJ_COMMENT", "true")
            .env("SJ_FAIL_ON", "gate");
        for (k, v) in env {
            cmd.env(k, v);
        }
        run(&mut cmd)
    }
}

const PASS: &str = "<!-- specjutsu:gate -->\n### ✅ gate: pass — 2 scenarios proven";
const FAIL: &str = "<!-- specjutsu:gate -->\n### ❌ gate: fail — 1 problem";

#[test]
fn ci_008_the_first_run_creates_the_comment() {
    let a = Action::new();
    std::fs::write(a.state().join("comments/1"), "LGTM\n").unwrap();
    let out = a.run(PASS, 0, &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(a.gate_comments(), vec![format!("{PASS}\n")]);
    assert_eq!(a.comments().len(), 2, "the unrelated comment is kept");
    assert!(a.summary().contains(PASS), "{}", a.summary());
}

#[test]
fn ci_009_later_runs_update_the_same_comment() {
    let a = Action::new();
    a.run(FAIL, 1, &[]);
    let out = a.run(PASS, 0, &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(a.gate_comments(), vec![format!("{PASS}\n")]);
    assert!(
        a.calls()
            .contains("-X PATCH repos/acme/shop/issues/comments/1"),
        "{}",
        a.calls()
    );
}

#[test]
fn ci_010_comments_can_be_turned_off() {
    let a = Action::new();
    let out = a.run(PASS, 0, &[("SJ_COMMENT", "false")]);
    assert!(out.status.success());
    assert_eq!(a.calls(), "", "gh was called");
    assert!(a.summary().contains(PASS));
}

#[test]
fn ci_011_a_read_only_token_skips_the_comment_not_the_gate() {
    let a = Action::new();
    let out = a.run(FAIL, 1, &[("STUB_READONLY", "1")]);
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1), "{all}");
    assert!(
        all.contains("cannot comment on this pull request (read-only token); see the job summary"),
        "{all}"
    );
    assert!(a.summary().contains(FAIL));
    assert!(a.gate_comments().is_empty());
}

#[test]
fn ci_012_fail_on_never_reports_without_blocking() {
    let a = Action::new();
    let out = a.run(FAIL, 1, &[("SJ_FAIL_ON", "never")]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(a.gate_comments(), vec![format!("{FAIL}\n")]);
    assert!(a.summary().contains(FAIL));

    let env_error = a.run("", 2, &[("SJ_FAIL_ON", "never")]);
    assert_eq!(env_error.status.code(), Some(2));
}

/// An `sj badge --out <path>` that writes `$STUB_BADGE` to the path.
const SJ_BADGE_STUB: &str = r#"#!/usr/bin/env bash
while [ $# -gt 0 ]; do [ "$1" = --out ] && out="$2"; shift; done
printf '%s\n' "$STUB_BADGE" > "$out"
echo "badge: written"
"#;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn ci_013_the_badge_branch_is_updated_only_when_the_badge_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let origin = t.join("origin.git");
    let work = t.join("work");
    git(
        t,
        &[
            "init",
            "-q",
            "--bare",
            "-b",
            "main",
            origin.to_str().unwrap(),
        ],
    );
    git(
        t,
        &[
            "clone",
            "-q",
            origin.to_str().unwrap(),
            work.to_str().unwrap(),
        ],
    );
    std::fs::write(work.join("README.md"), "# shop\n").unwrap();
    git(&work, &["add", "."]);
    git(&work, &["commit", "-qm", "init"]);
    git(&work, &["push", "-q", "origin", "HEAD:main"]);
    write_exe(&t.join("bin/sj"), SJ_BADGE_STUB);

    let badge =
        r#"{"schemaVersion":1,"label":"specs","message":"45/50 tested","color":"brightgreen"}"#;
    for _ in 0..2 {
        let out = run(Command::new("bash")
            .arg(script("badge.sh"))
            .current_dir(&work)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    t.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("STUB_BADGE", badge));
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    assert_eq!(
        git(&origin, &["rev-list", "--count", "sj-badge"]).trim(),
        "1"
    );
    assert_eq!(
        git(&origin, &["show", "sj-badge:badge.json"]).trim_end(),
        badge
    );
    assert_eq!(
        git(&origin, &["ls-tree", "--name-only", "sj-badge"]).trim(),
        "badge.json"
    );
    assert_eq!(
        git(&work, &["status", "--porcelain"]),
        "",
        "work tree was touched"
    );
}
