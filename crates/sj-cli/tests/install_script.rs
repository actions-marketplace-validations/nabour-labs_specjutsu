//! INST-001..005 from openspec/changes/agent-install: the public `install.sh`, run with a stub
//! `uname` against a local `file://` release folder. The script is POSIX `sh`; these tests run
//! on Linux and macOS only.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../install.sh")
}

fn write_exe(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn sha256(path: &Path) -> String {
    let out = Command::new("sh")
        .arg("-c")
        .arg("if command -v sha256sum >/dev/null; then sha256sum \"$1\"; else shasum -a 256 \"$1\"; fi")
        .arg("sh")
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap()[..64].to_string()
}

/// A scratch setup: a release folder, a stub `uname`, and an empty install folder.
struct Setup {
    tmp: tempfile::TempDir,
}

impl Setup {
    fn new(os: &str, arch: &str) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        write_exe(
            &tmp.path().join("stub/uname"),
            &format!(
                "#!/bin/sh\ncase \"$1\" in -s) echo {os};; -m) echo {arch};; *) echo {os};; esac\n"
            ),
        );
        std::fs::create_dir_all(tmp.path().join("release")).unwrap();
        std::fs::create_dir_all(tmp.path().join("dest")).unwrap();
        Setup { tmp }
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.tmp.path().join(rel)
    }

    /// Publish `sj-<target>.tar.gz` and a `SHA256SUMS` line for it (`sum` overrides the sum).
    fn publish(&self, target: &str, sum: Option<&str>) {
        let name = format!("sj-{target}");
        let pkg = self.path("pkg");
        write_exe(
            &pkg.join(&name).join("sj"),
            &format!("#!/bin/sh\necho 'sj 9.9.9 ({target})'\n"),
        );
        let asset = self.path("release").join(format!("{name}.tar.gz"));
        assert!(Command::new("tar")
            .arg("-czf")
            .arg(&asset)
            .arg("-C")
            .arg(&pkg)
            .arg(&name)
            .status()
            .unwrap()
            .success());
        let sum = sum.map(str::to_string).unwrap_or_else(|| sha256(&asset));
        let mut sums = std::fs::read_to_string(self.path("release/SHA256SUMS")).unwrap_or_default();
        sums.push_str(&format!("{sum}  {name}.tar.gz\n"));
        std::fs::write(self.path("release/SHA256SUMS"), sums).unwrap();
    }

    /// Run the script; `on_path` puts the install folder on PATH too.
    fn run(&self, on_path: bool) -> Output {
        let mut path = format!("{}:", self.path("stub").display());
        if on_path {
            path.push_str(&format!("{}:", self.path("dest").display()));
        }
        path.push_str(&std::env::var("PATH").unwrap());
        Command::new("sh")
            .arg(script())
            .env("PATH", path)
            .env("HOME", self.path("home"))
            .env("SJ_INSTALL_DIR", self.path("dest"))
            .env(
                "SJ_DOWNLOAD_URL",
                format!("file://{}", self.path("release").display()),
            )
            .env_remove("SJ_VERSION")
            .output()
            .unwrap()
    }

    fn installed(&self) -> bool {
        self.path("dest/sj").exists()
    }
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn inst_001_install_on_apple_silicon() {
    let s = Setup::new("Darwin", "arm64");
    s.publish("aarch64-apple-darwin", None);
    let out = s.run(false);
    let t = text(&out);
    assert!(out.status.success(), "{t}");
    let mode = std::fs::metadata(s.path("dest/sj"))
        .unwrap()
        .permissions()
        .mode();
    assert!(mode & 0o111 != 0, "not executable: {mode:o}");
    assert!(t.contains("sj 9.9.9 (aarch64-apple-darwin)"), "{t}");
}

#[test]
fn inst_002_linux_gets_the_static_build() {
    for (arch, target) in [
        ("x86_64", "x86_64-unknown-linux-musl"),
        ("aarch64", "aarch64-unknown-linux-gnu"),
        ("arm64", "aarch64-unknown-linux-gnu"),
    ] {
        let s = Setup::new("Linux", arch);
        s.publish(target, None);
        let out = s.run(false);
        let t = text(&out);
        assert!(out.status.success(), "{arch}: {t}");
        assert!(t.contains(&format!("sj 9.9.9 ({target})")), "{arch}: {t}");
    }
}

#[test]
fn inst_003_a_checksum_mismatch_installs_nothing() {
    let s = Setup::new("Darwin", "arm64");
    s.publish("aarch64-apple-darwin", Some(&"0".repeat(64)));
    let out = s.run(false);
    let t = text(&out);
    assert_eq!(out.status.code(), Some(1), "{t}");
    assert!(t.contains("sj-aarch64-apple-darwin.tar.gz"), "{t}");
    assert!(t.contains("checksum"), "{t}");
    assert!(!s.installed());
}

#[test]
fn inst_003_a_missing_checksum_line_installs_nothing() {
    let s = Setup::new("Darwin", "arm64");
    s.publish("x86_64-apple-darwin", None);
    let other = s.path("release/sj-x86_64-apple-darwin.tar.gz");
    std::fs::copy(&other, s.path("release/sj-aarch64-apple-darwin.tar.gz")).unwrap();
    let out = s.run(false);
    let t = text(&out);
    assert_eq!(out.status.code(), Some(1), "{t}");
    assert!(t.contains("checksum"), "{t}");
    assert!(!s.installed());
}

#[test]
fn inst_004_an_unsupported_platform_is_named() {
    let s = Setup::new("FreeBSD", "x86_64");
    let out = s.run(false);
    let t = text(&out);
    assert_eq!(out.status.code(), Some(1), "{t}");
    assert!(t.contains("FreeBSD x86_64"), "{t}");
    assert!(t.contains("cargo install specjutsu"), "{t}");
    assert!(!s.installed());
}

#[test]
fn inst_005_a_path_hint_when_the_folder_is_not_on_path() {
    let s = Setup::new("Darwin", "arm64");
    s.publish("aarch64-apple-darwin", None);
    let off = text(&s.run(false));
    let want = format!("add {} to your PATH", s.path("dest").display());
    assert!(off.contains(&want), "{off}");
    let on = s.run(true);
    let on_text = text(&on);
    assert!(on.status.success(), "{on_text}");
    assert!(!on_text.contains("to your PATH"), "{on_text}");
}
