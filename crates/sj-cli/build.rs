//! Embed the short git commit in `sj --version` when built from a checkout.

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn rerun_on(git_path: &str) {
    if let Some(p) = git(&["rev-parse", "--git-path", git_path]) {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(p);
        println!("cargo:rerun-if-changed={}", p.display());
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let version = env!("CARGO_PKG_VERSION");
    let full = match git(&["rev-parse", "--short", "HEAD"]) {
        Some(commit) => {
            rerun_on("HEAD");
            if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"]) {
                rerun_on(&r);
            }
            format!("{version} ({commit})")
        }
        None => version.to_string(),
    };
    println!("cargo:rustc-env=SJ_VERSION={full}");
}
