//! AUDIT-009 and AUDIT-010 from specjutsu/changes/audit-checks: `sj audit --registry` against
//! a local fixture registry.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command as Std;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use assert_cmd::Command;

/// Serves `routes` (path → status and body) on 127.0.0.1 and counts requests; any other path
/// is a 404.
struct Registry {
    url: String,
    hits: Arc<AtomicUsize>,
}

impl Registry {
    fn start(routes: &[(&str, u16, &str)]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let routes: HashMap<String, (u16, String)> = routes
            .iter()
            .map(|(p, s, b)| (p.to_string(), (*s, b.to_string())))
            .collect();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                counter.fetch_add(1, Ordering::SeqCst);
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                reader.read_line(&mut request).unwrap_or_default();
                let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
                    line.clear();
                }
                let (status, body) = routes
                    .get(&path)
                    .cloned()
                    .unwrap_or((404, "{}".to_string()));
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        Registry { url, hits }
    }
}

fn git(dir: &Path, args: &[&str]) {
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
}

/// A repository whose change rewrites `manifest` from `before` to `after`.
fn change(manifest: &str, before: &str, after: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    git(d, &["init", "-q", "-b", "main"]);
    std::fs::write(d.join(manifest), before).unwrap();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-qm", "init"]);
    git(d, &["checkout", "-qb", "feature"]);
    std::fs::write(d.join(manifest), after).unwrap();
    git(d, &["commit", "-qam", "add dependency"]);
    dir
}

fn audit(dir: &Path, env: &[(&str, &str)], registry: bool) -> (Option<i32>, String) {
    let mut cmd = Command::cargo_bin("sj").unwrap();
    cmd.arg("audit")
        .arg("--root")
        .arg(dir)
        .args(["--base", "main"]);
    if registry {
        cmd.arg("--registry");
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    (
        out.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn audit_009_a_dependency_that_does_not_exist_fails() {
    let reg = Registry::start(&[(
        "/react",
        200,
        r#"{"time":{"created":"2013-05-24T16:13:25.000Z"}}"#,
    )]);
    let dir = change(
        "package.json",
        "{\n  \"dependencies\": { \"react\": \"^18.0.0\" }\n}\n",
        "{\n  \"dependencies\": { \"react\": \"^18.0.0\", \"left-padx\": \"^1.0.0\" }\n}\n",
    );
    let env = [("SJ_REGISTRY_NPM", reg.url.as_str())];

    // Without --registry, nothing is looked up.
    let (_, out) = audit(dir.path(), &env, false);
    assert_eq!(reg.hits.load(Ordering::SeqCst), 0, "{out}");

    let (code, out) = audit(dir.path(), &env, true);
    assert!(out.contains("unknown-package npm left-padx"), "{out}");
    assert!(
        !out.contains("react"),
        "unchanged dependency was reported: {out}"
    );
    assert_eq!(code, Some(1), "{out}");
}

#[test]
fn audit_010_a_brand_new_dependency_warns() {
    let reg = Registry::start(&[(
        "/pypi/fastjsonx/json",
        200,
        r#"{"releases":{"0.1":[{"upload_time_iso_8601":"2026-11-09T10:00:00.000000Z"}],"0.2":[{"upload_time_iso_8601":"2026-11-15T10:00:00.000000Z"}]}}"#,
    )]);
    let dir = change(
        "pyproject.toml",
        "[project]\nname = \"svc\"\ndependencies = [\"requests>=2\"]\n",
        "[project]\nname = \"svc\"\ndependencies = [\"requests>=2\", \"fastjsonx>=0.1\"]\n",
    );
    let (code, out) = audit(
        dir.path(),
        &[
            ("SJ_REGISTRY_PYPI", reg.url.as_str()),
            ("SJ_TODAY", "2026-11-18"),
        ],
        true,
    );
    assert!(
        out.contains("warn    new-package pypi fastjsonx (first published 9 days ago)"),
        "{out}"
    );
    assert_eq!(code, Some(0), "{out}");

    // An unreachable registry warns and the audit continues.
    let (code, out) = audit(
        dir.path(),
        &[
            ("SJ_REGISTRY_PYPI", "http://127.0.0.1:9"),
            ("SJ_TODAY", "2026-11-18"),
        ],
        true,
    );
    assert!(out.contains("registry unreachable: pypi"), "{out}");
    assert_eq!(code, Some(0), "{out}");
}
