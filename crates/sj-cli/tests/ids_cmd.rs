//! `sj ids` CLI scenarios (specjutsu/changes/scenario-ids).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

const USER_AUTH: &str = "## Requirements\n\n### Requirement: Registration\nThe system SHALL register users.\n\n\
#### Scenario: Valid registration\n- **WHEN** a user registers\n- **THEN** an account exists\n\n\
#### Scenario: Password too short\n- **WHEN** the password has 3 characters\n- **THEN** it is rejected\n";

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn sj(root: &Path) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("ids").arg("--root").arg(root);
    c
}

#[test]
fn ids_001_listing_shows_the_proposed_ids_and_exits_1() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/user-auth/spec.md", USER_AUTH);
    sj(dir.path()).assert().code(1).stdout(
        "2 scenarios without IDs\n\
         USERAUTH-001 Valid registration (openspec/specs/user-auth/spec.md:6)\n\
         USERAUTH-002 Password too short (openspec/specs/user-auth/spec.md:10)\n\
         hint: user-auth would use USERAUTH; sj ids prefixes --write maps it to USER\n",
    );
    assert_eq!(
        read(dir.path(), "openspec/specs/user-auth/spec.md"),
        USER_AUTH
    );
}

#[test]
fn ids_001_one_scenario_is_singular() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "openspec/specs/user-auth/spec.md",
        &USER_AUTH.replace("Scenario: Valid", "Scenario: USERAUTH-001 Valid"),
    );
    sj(dir.path())
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with("1 scenario without an ID\n"));
}

#[test]
fn ids_001_json_lists_the_assignments() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/user-auth/spec.md", USER_AUTH);
    let out = sj(dir.path()).arg("--json").assert().code(1);
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["missing"], 2);
    assert_eq!(v["written"], false);
    assert_eq!(v["assignments"][1]["id"], "USERAUTH-002");
    assert_eq!(
        v["assignments"][1]["file"],
        "openspec/specs/user-auth/spec.md"
    );
    assert_eq!(v["assignments"][1]["line"], 10);
}

#[test]
fn ids_002_write_assigns_in_place_and_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/user-auth/spec.md", USER_AUTH);
    sj(dir.path())
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::ends_with("wrote 2 IDs to 1 file\n"));
    assert_eq!(
        read(dir.path(), "openspec/specs/user-auth/spec.md"),
        USER_AUTH
            .replace("Scenario: Valid", "Scenario: USERAUTH-001 Valid")
            .replace("Scenario: Password", "Scenario: USERAUTH-002 Password")
    );
}

#[test]
fn ids_003_running_twice_is_a_no_op() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/user-auth/spec.md", USER_AUTH);
    sj(dir.path()).arg("--write").assert().success();
    let after = read(dir.path(), "openspec/specs/user-auth/spec.md");
    sj(dir.path())
        .arg("--write")
        .assert()
        .success()
        .stdout("0 scenarios without IDs\n");
    sj(dir.path())
        .assert()
        .success()
        .stdout("0 scenarios without IDs\n");
    assert_eq!(read(dir.path(), "openspec/specs/user-auth/spec.md"), after);
}

#[test]
fn ids_004_prefixes_come_from_specjutsu_toml() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/user-auth/spec.md", USER_AUTH);
    put(
        dir.path(),
        "specjutsu.toml",
        "schema = 1\n\n[ids.prefixes]\nuser-auth = \"AUTH\"\n",
    );
    sj(dir.path())
        .assert()
        .code(1)
        .stdout(predicate::str::contains("AUTH-001 Valid registration"));
}

#[test]
fn ids_005_colliding_prefixes_stop_the_write() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/chat/spec.md", USER_AUTH);
    put(dir.path(), "openspec/specs/c-hat/spec.md", USER_AUTH);
    sj(dir.path())
        .arg("--write")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("`c-hat`").and(predicate::str::contains("`chat`")));
    assert_eq!(read(dir.path(), "openspec/specs/chat/spec.md"), USER_AUTH);
    assert_eq!(read(dir.path(), "openspec/specs/c-hat/spec.md"), USER_AUTH);
}

/// Every file under `root` with its content, to show a command wrote nothing.
fn snapshot(root: &Path) -> Vec<(std::path::PathBuf, String)> {
    let mut out = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                out.push((p.clone(), std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn ids_008_long_capability_names_get_four_letter_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["distribution", "gate", "payment-methods"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    let before = snapshot(dir.path());
    sj(dir.path()).arg("prefixes").assert().success().stdout(
        "[ids.prefixes]\n\
         distribution = \"DIST\"\n\
         gate = \"GATE\"\n\
         payment-methods = \"PAYM\"\n",
    );
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn ids_009_prefixes_already_in_use_are_pinned() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "openspec/specs/user-auth/spec.md",
        &USER_AUTH
            .replace("Scenario: Valid", "Scenario: USERAUTH-001 Valid")
            .replace("Scenario: Password", "Scenario: USERAUTH-002 Password"),
    );
    put(dir.path(), "openspec/specs/billing/spec.md", USER_AUTH);
    put(
        dir.path(),
        "specjutsu.toml",
        "schema = 1\n\n[ids.prefixes]\nbilling = \"BILL\"\n",
    );
    sj(dir.path()).arg("prefixes").assert().success().stdout(
        "[ids.prefixes]\n\
         billing = \"BILL\"\n\
         user-auth = \"USERAUTH\"\n",
    );
}

#[test]
fn ids_010_clashing_suggestions_lengthen_until_distinct() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["transfers", "transactions"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    sj(dir.path()).arg("prefixes").assert().success().stdout(
        "[ids.prefixes]\n\
         transactions = \"TRANSA\"\n\
         transfers = \"TRANSF\"\n",
    );
}

#[test]
fn ids_011_writing_keeps_the_rest_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["billing", "distribution"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    let toml =
        "# project settings\nschema = 1\nmode = \"strict\"\n\n[ids.prefixes]\nbilling = \"BILL\"\n";
    put(dir.path(), "specjutsu.toml", toml);
    sj(dir.path())
        .arg("prefixes")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::ends_with(
            "distribution = \"DIST\"\nadded 1 prefix to specjutsu.toml\n",
        ));
    assert_eq!(
        read(dir.path(), "specjutsu.toml"),
        "# project settings\nschema = 1\nmode = \"strict\"\n\n[ids.prefixes]\nbilling = \"BILL\"\ndistribution = \"DIST\"\n"
    );
    let after = read(dir.path(), "specjutsu.toml");
    sj(dir.path())
        .arg("prefixes")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("added").not());
    assert_eq!(read(dir.path(), "specjutsu.toml"), after);
}

/// Two capability folders, `billing` and `distribution`, under `root`.
fn two_capabilities(root: &Path) {
    for cap in ["billing", "distribution"] {
        put(root, &format!("openspec/specs/{cap}/spec.md"), USER_AUTH);
    }
}

#[test]
fn ids_prefixes_write_without_a_project_file_points_to_sj_init() {
    let dir = tempfile::tempdir().unwrap();
    two_capabilities(dir.path());
    sj(dir.path())
        .arg("prefixes")
        .arg("--write")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("sj init"));
    assert!(!dir.path().join("specjutsu.toml").exists());
}

#[test]
fn ids_prefixes_write_refuses_prefixes_it_cannot_splice() {
    for toml in [
        "schema = 1\n\n[ids]\nprefixes = { billing = \"BILL\" }\n",
        "schema = 1\nids.prefixes.billing = \"BILL\"\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        two_capabilities(dir.path());
        put(dir.path(), "specjutsu.toml", toml);
        sj(dir.path())
            .arg("prefixes")
            .arg("--write")
            .assert()
            .code(2)
            .stderr(predicate::str::contains("distribution = \"DIST\""));
        assert_eq!(read(dir.path(), "specjutsu.toml"), toml);
    }
}

#[test]
fn ids_prefixes_write_finds_a_header_with_spaces() {
    let dir = tempfile::tempdir().unwrap();
    two_capabilities(dir.path());
    put(
        dir.path(),
        "specjutsu.toml",
        "schema = 1\n[ ids.prefixes ]\nbilling = \"BILL\"\n",
    );
    sj(dir.path())
        .arg("prefixes")
        .arg("--write")
        .assert()
        .success();
    assert_eq!(
        read(dir.path(), "specjutsu.toml"),
        "schema = 1\n[ ids.prefixes ]\nbilling = \"BILL\"\ndistribution = \"DIST\"\n"
    );
}

#[test]
fn ids_parent_write_and_json_are_refused_with_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    two_capabilities(dir.path());
    for flag in ["--write", "--json"] {
        sj(dir.path())
            .arg(flag)
            .arg("prefixes")
            .assert()
            .code(2)
            .stderr(predicate::str::contains("sj ids prefixes --write"));
    }
    assert!(!dir.path().join("specjutsu.toml").exists());
}

#[test]
fn ids_012_a_long_derived_prefix_gets_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/distribution/spec.md", USER_AUTH);
    let out = sj(dir.path()).assert().code(1);
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout.lines().last(),
        Some("hint: distribution would use DISTRIBUTION; sj ids prefixes --write maps it to DIST")
    );
}

#[test]
fn ids_013_an_unresolvable_clash_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["bill", "billing"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    let toml = "schema = 1\n\n[ids.prefixes]\nbilling = \"BILL\"\n";
    put(dir.path(), "specjutsu.toml", toml);
    sj(dir.path())
        .arg("prefixes")
        .arg("--write")
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "capabilities `bill` and `billing` both use prefix BILL",
        ));
    assert_eq!(read(dir.path(), "specjutsu.toml"), toml);
}
