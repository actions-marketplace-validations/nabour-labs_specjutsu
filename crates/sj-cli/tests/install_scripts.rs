//! DIST-001..009 from specjutsu/changes/install-channels: the public install scripts and the
//! Homebrew formula script, run against a release served from a local folder.
//! install.sh and homebrew.sh are POSIX shell and run on Linux and macOS only. install.ps1 runs
//! wherever `pwsh` is installed, which includes every GitHub-hosted runner.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[cfg(unix)]
mod sh {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    pub fn write_exe(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn sha256(path: &Path) -> String {
        let out = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .or_else(|_| Command::new("sha256sum").arg(path).output())
            .unwrap();
        stdout(&out).split_whitespace().next().unwrap().to_string()
    }

    /// Package an `sj` that prints `sj <tag>` as `sj-<target>.tar.gz` into `release/<sub>`, and
    /// write `SHA256SUMS` beside it (with a wrong checksum when `tamper`).
    pub fn release(t: &Path, sub: &str, target: &str, tag: &str, tamper: bool) {
        let pkg = t.join("pkg").join(format!("sj-{target}"));
        write_exe(&pkg.join("sj"), &format!("#!/bin/sh\necho 'sj {tag}'\n"));
        let dir = t.join("release").join(sub);
        std::fs::create_dir_all(&dir).unwrap();
        let asset = format!("sj-{target}.tar.gz");
        assert!(Command::new("tar")
            .arg("-czf")
            .arg(dir.join(&asset))
            .arg("-C")
            .arg(t.join("pkg"))
            .arg(format!("sj-{target}"))
            .status()
            .unwrap()
            .success());
        let sum = if tamper {
            "0".repeat(64)
        } else {
            sha256(&dir.join(&asset))
        };
        std::fs::write(dir.join("SHA256SUMS"), format!("{sum}  {asset}\n")).unwrap();
    }

    /// Run install.sh with a stub `uname`, the release in `<t>/release`, HOME at `<t>/home` and
    /// the install folder `<t>/bin`, which is not on PATH.
    pub fn install(t: &Path, os: &str, arch: &str, version: Option<&str>) -> Output {
        write_exe(
            &t.join("stub/uname"),
            "#!/bin/sh\ncase \"$1\" in -s) echo \"$STUB_OS\" ;; -m) echo \"$STUB_ARCH\" ;; *) echo \"$STUB_OS\" ;; esac\n",
        );
        std::fs::create_dir_all(t.join("home")).unwrap();
        let mut cmd = Command::new("sh");
        cmd.arg(repo_root().join("scripts/install.sh"))
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    t.join("stub").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("STUB_OS", os)
            .env("STUB_ARCH", arch)
            .env("HOME", t.join("home"))
            .env(
                "SJ_DOWNLOAD_BASE",
                format!("file://{}", t.join("release").display()),
            )
            .env("SJ_INSTALL_DIR", t.join("bin"))
            .env_remove("SJ_VERSION");
        if let Some(v) = version {
            cmd.env("SJ_VERSION", v);
        }
        cmd.output().unwrap()
    }
}

#[cfg(unix)]
#[test]
fn dist_001_the_latest_release_is_installed_for_the_detected_platform() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    sh::release(
        t,
        "latest/download",
        "aarch64-apple-darwin",
        "latest",
        false,
    );
    let out = sh::install(t, "Darwin", "arm64", None);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let sj = t.join("bin/sj");
    assert!(sj.exists());
    use std::os::unix::fs::PermissionsExt;
    assert!(std::fs::metadata(&sj).unwrap().permissions().mode() & 0o111 != 0);
    assert!(
        stdout(&out).contains(&format!("installed sj to {}", sj.display())),
        "{}",
        stdout(&out)
    );
}

#[cfg(unix)]
#[test]
fn dist_002_sj_version_pins_the_release() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    sh::release(
        t,
        "download/v0.4.0",
        "x86_64-unknown-linux-musl",
        "0.4.0",
        false,
    );
    let out = sh::install(t, "Linux", "x86_64", Some("0.4.0"));
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let ran = Command::new(t.join("bin/sj")).output().unwrap();
    assert_eq!(stdout(&ran).trim(), "sj 0.4.0");
}

#[cfg(unix)]
#[test]
fn dist_003_a_tampered_download_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    sh::release(t, "latest/download", "aarch64-apple-darwin", "latest", true);
    let out = sh::install(t, "Darwin", "arm64", None);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("checksum mismatch for sj-aarch64-apple-darwin.tar.gz"),
        "{}",
        stderr(&out)
    );
    assert!(!t.join("bin/sj").exists());
}

#[cfg(unix)]
#[test]
fn dist_004_an_unsupported_platform_is_named() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    // A curl that records any call: nothing may be downloaded.
    sh::write_exe(
        &t.join("stub/curl"),
        &format!(
            "#!/bin/sh\ntouch '{}'\nexit 7\n",
            t.join("curl-ran").display()
        ),
    );
    let out = sh::install(t, "FreeBSD", "amd64", None);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("no sj release for FreeBSD/amd64"),
        "{}",
        stderr(&out)
    );
    assert!(!t.join("curl-ran").exists(), "install.sh downloaded");
}

#[cfg(unix)]
#[test]
fn dist_005_a_folder_missing_from_path_gets_a_hint() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    sh::release(
        t,
        "latest/download",
        "aarch64-apple-darwin",
        "latest",
        false,
    );
    let out = sh::install(t, "Darwin", "arm64", None);
    assert!(out.status.success(), "{}", stderr(&out));
    let hint = format!("export PATH=\"{}:$PATH\"", t.join("bin").display());
    assert!(stdout(&out).contains(&hint), "{}", stdout(&out));
    // HOME is untouched: no shell profile was written.
    assert_eq!(std::fs::read_dir(t.join("home")).unwrap().count(), 0);
}

// ---- install.ps1 ----

fn pwsh(args: &[&str], envs: &[(&str, &Path)], extra: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new("pwsh");
    // cargo passes build.rs's SJ_VERSION (the `sj --version` string) to tests; clear it so
    // the installer fetches `latest`, as install.sh's tests do.
    cmd.args(["-NoLogo", "-NoProfile", "-NonInteractive"])
        .args(args)
        .env_remove("SJ_VERSION");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    for (k, v) in extra {
        cmd.env(k, v);
    }
    cmd.output().unwrap_or_else(|e| {
        panic!("pwsh not found ({e}); install PowerShell 7 to run the install.ps1 tests")
    })
}

/// Zip an `sj.exe` as `sj-x86_64-pc-windows-msvc.zip` into `<t>/release/latest/download` with
/// its `SHA256SUMS`, then run install.ps1 against it.
fn ps1_install(t: &Path, tamper: bool) -> Output {
    let target = "x86_64-pc-windows-msvc";
    let pkg = t.join("pkg").join(format!("sj-{target}"));
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("sj.exe"), b"not really an exe").unwrap();
    let dir = t.join("release/latest/download");
    std::fs::create_dir_all(&dir).unwrap();
    let asset = format!("sj-{target}.zip");
    let zip = dir.join(&asset);
    let out = pwsh(
        &[
            "-Command",
            "Compress-Archive -Path $env:PKG -DestinationPath $env:ZIP; (Get-FileHash -Algorithm SHA256 $env:ZIP).Hash.ToLower()",
        ],
        &[("PKG", &pkg), ("ZIP", &zip)],
        &[],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let sum = if tamper {
        "0".repeat(64)
    } else {
        stdout(&out).trim().to_string()
    };
    std::fs::write(dir.join("SHA256SUMS"), format!("{sum}  {asset}\n")).unwrap();

    pwsh(
        &[
            "-File",
            repo_root().join("scripts/install.ps1").to_str().unwrap(),
        ],
        &[
            ("SJ_DOWNLOAD_BASE", &t.join("release")),
            ("SJ_INSTALL_DIR", &t.join("bin")),
        ],
        &[("SJ_TARGET", target), ("SJ_NO_MODIFY_PATH", "1")],
    )
}

#[test]
fn dist_006_the_windows_archive_is_installed_after_verification() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let out = ps1_install(t, false);
    let all = format!("{}{}", stdout(&out), stderr(&out));
    assert!(out.status.success(), "{all}");
    assert!(t.join("bin").join("sj.exe").exists());
    assert!(all.contains("installed sj to"), "{all}");
    assert!(
        all.contains(&format!("add {} to your PATH", t.join("bin").display())),
        "{all}"
    );
}

#[test]
fn dist_007_a_tampered_windows_download_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let out = ps1_install(t, true);
    let all = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(out.status.code(), Some(1), "{all}");
    assert!(
        all.contains("checksum mismatch for sj-x86_64-pc-windows-msvc.zip"),
        "{all}"
    );
    assert!(!t.join("bin").join("sj.exe").exists());
}

// ---- Homebrew formula ----

#[cfg(unix)]
const ALL_ASSETS: [&str; 6] = [
    "sj-aarch64-apple-darwin.tar.gz",
    "sj-x86_64-apple-darwin.tar.gz",
    "sj-aarch64-unknown-linux-gnu.tar.gz",
    "sj-x86_64-unknown-linux-gnu.tar.gz",
    "sj-x86_64-unknown-linux-musl.tar.gz",
    "sj-x86_64-pc-windows-msvc.zip",
];

/// A distinct fake checksum per asset: its index repeated.
#[cfg(unix)]
fn fake_sum(i: usize) -> String {
    format!("{i}").repeat(64)
}

#[cfg(unix)]
fn homebrew(t: &Path, skip: Option<&str>) -> Output {
    let sums: String = ALL_ASSETS
        .iter()
        .enumerate()
        .filter(|(_, a)| Some(**a) != skip)
        .map(|(i, a)| format!("{}  {a}\n", fake_sum(i)))
        .collect();
    let file = t.join("SHA256SUMS");
    std::fs::write(&file, sums).unwrap();
    Command::new("sh")
        .arg(repo_root().join("scripts/release/homebrew.sh"))
        .arg("0.5.0")
        .arg(&file)
        .output()
        .unwrap()
}

#[cfg(unix)]
#[test]
fn dist_008_the_formula_pins_every_archive_with_its_checksum() {
    let tmp = tempfile::tempdir().unwrap();
    let out = homebrew(tmp.path(), None);
    let f = stdout(&out);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(f.contains("class Specjutsu < Formula"), "{f}");
    assert!(f.contains("version \"0.5.0\""), "{f}");
    assert!(f.contains("bin.install \"sj\""), "{f}");
    for (i, asset) in ALL_ASSETS.iter().enumerate() {
        let url = format!(
            "url \"https://github.com/nabour-labs/specjutsu/releases/download/v0.5.0/{asset}\""
        );
        let wanted = matches!(
            *asset,
            "sj-aarch64-apple-darwin.tar.gz"
                | "sj-x86_64-apple-darwin.tar.gz"
                | "sj-aarch64-unknown-linux-gnu.tar.gz"
                | "sj-x86_64-unknown-linux-musl.tar.gz"
        );
        if !wanted {
            assert!(!f.contains(&url), "unexpected {asset} in\n{f}");
            continue;
        }
        let at = f.find(&url).unwrap_or_else(|| panic!("no {url} in\n{f}"));
        let next = f[at..].lines().nth(1).unwrap().trim();
        assert_eq!(next, format!("sha256 \"{}\"", fake_sum(i)), "{f}");
    }
}

#[cfg(unix)]
#[test]
fn dist_009_a_missing_archive_stops_the_formula() {
    let tmp = tempfile::tempdir().unwrap();
    let out = homebrew(tmp.path(), Some("sj-x86_64-apple-darwin.tar.gz"));
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("SHA256SUMS has no sj-x86_64-apple-darwin.tar.gz"),
        "{}",
        stderr(&out)
    );
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
}
