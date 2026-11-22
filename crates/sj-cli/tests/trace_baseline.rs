//! TRACE-017 from specjutsu/changes/brownfield-adoption: `sj trace --write-baseline`.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const SPEC: &str = "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
                    #### Scenario: AUTH-005 Refresh token\n- **WHEN** x\n- **THEN** y\n\n\
                    #### Scenario: AUTH-002 Log in\n- **WHEN** x\n- **THEN** y\n";

#[test]
fn trace_017_write_baseline_lists_every_gap_and_orphan() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", "schema = 1\nspec_root = \"specjutsu\"\n");
    write(
        r,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(r, "src/lib.rs", "");
    write(r, "specjutsu/specs/auth/spec.md", SPEC);
    write(r, "tests/auth.rs", "#[test]\nfn auth_009_old() {}\n");

    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--write-baseline", "--root"])
        .arg(r)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let baseline = std::fs::read_to_string(r.join("specjutsu/trace-baseline.txt")).unwrap();
    assert_eq!(
        baseline,
        "gap AUTH-002\ngap AUTH-005\norphan AUTH-009 tests/auth.rs auth_009_old\n"
    );
}
