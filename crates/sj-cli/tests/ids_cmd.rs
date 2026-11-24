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

/// IDS-013 with a configured mapping whose capability has no folder: the clash still counts.
#[test]
fn ids_013_a_clash_with_a_mapping_without_a_folder_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/bill/spec.md", USER_AUTH);
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

#[test]
fn ids_prefixes_takes_root_after_the_subcommand() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/gate/spec.md", USER_AUTH);
    Command::cargo_bin("sj")
        .unwrap()
        .args(["ids", "prefixes", "--root"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout("[ids.prefixes]\ngate = \"GATE\"\n");
}

/// Two capabilities, `staff` and `billing`, whose scenarios have no IDs.
fn staff_and_billing(root: &Path) {
    for cap in ["staff", "billing"] {
        put(root, &format!("openspec/specs/{cap}/spec.md"), USER_AUTH);
    }
}

#[test]
fn ids_014_writing_one_capability_leaves_the_others_alone() {
    let dir = tempfile::tempdir().unwrap();
    staff_and_billing(dir.path());
    let billing = std::fs::read(dir.path().join("openspec/specs/billing/spec.md")).unwrap();
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .success()
        .stdout(
            "2 scenarios without IDs\n\
             STAFF-001 Valid registration (openspec/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/specs/staff/spec.md:10)\n\
             wrote 2 IDs to 1 file\n",
        );
    assert_eq!(
        read(dir.path(), "openspec/specs/staff/spec.md"),
        USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-001 Valid")
            .replace("Scenario: Password", "Scenario: STAFF-002 Password")
    );
    assert_eq!(
        std::fs::read(dir.path().join("openspec/specs/billing/spec.md")).unwrap(),
        billing
    );
}

/// IDS-014 with IDS-006 and IDS-007: numbers continue past IDs outside the target, and the
/// targeted capability's delta copies in open changes are included.
#[test]
fn ids_014_a_target_keeps_numbering_and_delta_copies() {
    let dir = tempfile::tempdir().unwrap();
    staff_and_billing(dir.path());
    put(
        dir.path(),
        "openspec/changes/archive/2026-01-01-old/specs/staff/spec.md",
        &USER_AUTH.replace("Scenario: Valid", "Scenario: STAFF-007 Valid"),
    );
    put(
        dir.path(),
        "openspec/changes/add/specs/staff/spec.md",
        USER_AUTH,
    );
    put(
        dir.path(),
        "openspec/changes/add/specs/billing/spec.md",
        USER_AUTH,
    );
    sj(dir.path()).arg("staff").assert().code(1).stdout(
        "4 scenarios without IDs\n\
         STAFF-008 Valid registration (openspec/specs/staff/spec.md:6)\n\
         STAFF-009 Password too short (openspec/specs/staff/spec.md:10)\n\
         STAFF-008 Valid registration (openspec/changes/add/specs/staff/spec.md:6)\n\
         STAFF-009 Password too short (openspec/changes/add/specs/staff/spec.md:10)\n",
    );
}

#[test]
fn ids_015_a_spec_path_limits_the_listing_the_same_way() {
    let dir = tempfile::tempdir().unwrap();
    staff_and_billing(dir.path());
    let before = snapshot(dir.path());
    sj(dir.path())
        .arg("openspec/specs/staff/spec.md")
        .assert()
        .code(1)
        .stdout(
            "2 scenarios without IDs\n\
             STAFF-001 Valid registration (openspec/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/specs/staff/spec.md:10)\n",
        );
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn ids_016_an_unknown_target_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    staff_and_billing(dir.path());
    let before = snapshot(dir.path());
    sj(dir.path())
        .args(["--write", "staf"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("`staf`").and(predicate::str::contains(
                "known capabilities: billing, staff",
            )),
        );
    assert_eq!(snapshot(dir.path()), before);
}

/// IDS-015 with a delta file as the target: it selects the capability, `specs/` file included.
#[test]
fn ids_015_a_delta_file_target_selects_its_capability() {
    let dir = tempfile::tempdir().unwrap();
    staff_and_billing(dir.path());
    put(
        dir.path(),
        "openspec/changes/add/specs/staff/spec.md",
        USER_AUTH,
    );
    let before = snapshot(dir.path());
    sj(dir.path())
        .arg("openspec/changes/add/specs/staff/spec.md")
        .assert()
        .code(1)
        .stdout(
            "4 scenarios without IDs\n\
             STAFF-001 Valid registration (openspec/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/specs/staff/spec.md:10)\n\
             STAFF-001 Valid registration (openspec/changes/add/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/changes/add/specs/staff/spec.md:10)\n",
        );
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn ids_017_a_target_still_stops_on_colliding_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["staff", "st-aff"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    let before = snapshot(dir.path());
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("`st-aff`").and(predicate::str::contains("`staff`")));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn ids_021_an_unrelated_collision_does_not_stop_a_target() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["staff", "billing", "bill-ing"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .code(0)
        .stdout(
            "2 scenarios without IDs\n\
             STAFF-001 Valid registration (openspec/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/specs/staff/spec.md:10)\n\
             wrote 2 IDs to 1 file\n",
        );
    assert_eq!(
        read(dir.path(), "openspec/specs/staff/spec.md"),
        USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-001 Valid")
            .replace("Scenario: Password", "Scenario: STAFF-002 Password")
    );
    for cap in ["billing", "bill-ing"] {
        assert_eq!(
            read(dir.path(), &format!("openspec/specs/{cap}/spec.md")),
            USER_AUTH,
            "{cap}"
        );
    }
}

// Regression: IDS-021 on the listing path, which `sj ids` without `--write` takes. No ID: the
// scenario names `--write`.
#[test]
fn regression_listing_a_target_ignores_an_unrelated_collision() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["staff", "billing", "bill-ing"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    sj(dir.path())
        .arg("staff")
        .assert()
        .code(1)
        .stdout(
            "2 scenarios without IDs\n\
             STAFF-001 Valid registration (openspec/specs/staff/spec.md:6)\n\
             STAFF-002 Password too short (openspec/specs/staff/spec.md:10)\n",
        )
        .stderr("");
    let out = sj(dir.path()).args(["--json", "staff"]).assert().code(1);
    assert_eq!(String::from_utf8_lossy(&out.get_output().stderr), "");
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["missing"], 2);
}

// Regression: a target with nothing to number lists nothing, as `--write` writes nothing,
// even when an unselected capability needing IDs collides with its prefix. Listing `st-aff`,
// which would number, reports that collision as a line.
#[test]
fn regression_a_fully_numbered_target_lists_nothing_beside_a_colliding_capability() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "openspec/specs/staff/spec.md",
        &USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-001 Valid")
            .replace("Scenario: Password", "Scenario: STAFF-002 Password"),
    );
    put(dir.path(), "openspec/specs/st-aff/spec.md", USER_AUTH);
    let before = snapshot(dir.path());
    sj(dir.path())
        .arg("staff")
        .assert()
        .success()
        .stdout("0 scenarios without IDs\n")
        .stderr("");
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .success()
        .stdout("0 scenarios without IDs\n");
    assert_eq!(snapshot(dir.path()), before);
    sj(dir.path())
        .arg("st-aff")
        .assert()
        .code(1)
        .stdout(predicate::str::ends_with(
            "\nnote: capabilities `st-aff` and `staff` both use prefix STAFF; \
             map one in [ids.prefixes] in specjutsu.toml\n",
        ))
        .stderr("");
}

// Regression: within IDS-021's rule, a capability outside the targets with no usable prefix
// doesn't stop the run either. No ID: the scenario covers collisions only.
#[test]
fn regression_an_unselected_capability_without_a_prefix_does_not_stop_a_target() {
    let dir = tempfile::tempdir().unwrap();
    for cap in ["staff", "123"] {
        put(
            dir.path(),
            &format!("openspec/specs/{cap}/spec.md"),
            USER_AUTH,
        );
    }
    sj(dir.path()).args(["--write", "staff"]).assert().code(0);
    assert_eq!(
        read(dir.path(), "openspec/specs/staff/spec.md"),
        USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-001 Valid")
            .replace("Scenario: Password", "Scenario: STAFF-002 Password")
    );
    assert_eq!(read(dir.path(), "openspec/specs/123/spec.md"), USER_AUTH);
}

const ROLE_SPEC: &str = "## Requirements\n\n### Requirement: Role changes are audited\nThe system SHALL audit role changes.\n\n\
#### Scenario: Role change is recorded\n- **WHEN** a role changes\n- **THEN** it is recorded\n\n\
#### Scenario: STAFF-099 Role change is visible\n- **WHEN** a role changes\n- **THEN** it is shown\n";

const ROLE_DELTA: &str = "## MODIFIED Requirements\n\n### Requirement: Role changes are audited\nThe system SHALL audit role changes.\n\n\
#### Scenario: STAFF-057 Role change is recorded\n- **WHEN** a role changes\n- **THEN** it is recorded with its author\n";

#[test]
fn ids_018_the_original_takes_the_deltas_id() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        ROLE_DELTA,
    );
    let delta = std::fs::read(
        dir.path()
            .join("openspec/changes/audit/specs/staff/spec.md"),
    )
    .unwrap();
    sj(dir.path()).args(["--write", "staff"]).assert().success();
    assert_eq!(
        read(dir.path(), "openspec/specs/staff/spec.md"),
        ROLE_SPEC.replace(
            "#### Scenario: Role change is recorded",
            "#### Scenario: STAFF-057 Role change is recorded"
        )
    );
    assert_eq!(
        std::fs::read(
            dir.path()
                .join("openspec/changes/audit/specs/staff/spec.md")
        )
        .unwrap(),
        delta
    );
}

/// IDS-018 when the delta's ID is already used by another scenario: a new number, and a line
/// naming both scenarios.
#[test]
fn ids_018_a_taken_id_gets_a_new_number_and_a_note() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        ROLE_DELTA,
    );
    put(
        dir.path(),
        "openspec/specs/billing/spec.md",
        &USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-057 Valid")
            .replace("Scenario: Password", "Scenario: BILLING-001 Password"),
    );
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "note: STAFF-057 Role change is recorded (openspec/changes/audit/specs/staff/spec.md:6) \
             is taken by STAFF-057 Valid registration (openspec/specs/billing/spec.md:6); \
             Role change is recorded (openspec/specs/staff/spec.md:6) gets STAFF-100; \
             give the delta copies STAFF-100\n",
        ));
    assert!(read(dir.path(), "openspec/specs/staff/spec.md")
        .contains("#### Scenario: STAFF-100 Role change is recorded"));
}

#[test]
fn ids_019_copies_that_disagree_get_a_new_number_and_a_note() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/c1/specs/staff/spec.md",
        ROLE_DELTA,
    );
    put(
        dir.path(),
        "openspec/changes/c2/specs/staff/spec.md",
        &ROLE_DELTA.replace("STAFF-057", "STAFF-058"),
    );
    let deltas = |root: &Path| {
        ["c1", "c2"].map(|c| {
            std::fs::read(root.join(format!("openspec/changes/{c}/specs/staff/spec.md"))).unwrap()
        })
    };
    let before = deltas(dir.path());
    let out = sj(dir.path()).args(["--json", "staff"]).assert().code(1);
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let note = &v["assignments"][0]["note"];
    assert_eq!(note["kind"], "disagree");
    assert_eq!(note["copies"][0]["id"], "STAFF-057");
    assert_eq!(
        note["copies"][0]["file"],
        "openspec/changes/c1/specs/staff/spec.md"
    );
    assert_eq!(note["copies"][1]["id"], "STAFF-058");
    sj(dir.path())
        .args(["--write", "staff"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "note: copies disagree: \
             STAFF-057 Role change is recorded (openspec/changes/c1/specs/staff/spec.md:6), \
             STAFF-058 Role change is recorded (openspec/changes/c2/specs/staff/spec.md:6); \
             Role change is recorded (openspec/specs/staff/spec.md:6) gets STAFF-100; \
             give the delta copies STAFF-100\n",
        ));
    assert!(read(dir.path(), "openspec/specs/staff/spec.md")
        .contains("#### Scenario: STAFF-100 Role change is recorded"));
    assert_eq!(deltas(dir.path()), before);
}

/// IDS-018's JSON carries the taken note too.
#[test]
fn ids_018_json_carries_the_taken_note() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        ROLE_DELTA,
    );
    put(
        dir.path(),
        "openspec/specs/billing/spec.md",
        &USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-057 Valid")
            .replace("Scenario: Password", "Scenario: BILLING-001 Password"),
    );
    let out = sj(dir.path()).args(["--json", "staff"]).assert().code(1);
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let note = &v["assignments"][0]["note"];
    assert_eq!(v["assignments"][0]["id"], "STAFF-100");
    assert_eq!(note["kind"], "taken");
    assert_eq!(
        note["copy"]["file"],
        "openspec/changes/audit/specs/staff/spec.md"
    );
    assert_eq!(note["by"]["name"], "Valid registration");
    assert_eq!(note["by"]["file"], "openspec/specs/billing/spec.md");
}

/// IDS-018: a `## ` line inside fenced code does not end the MODIFIED section.
#[test]
fn ids_018_a_fenced_heading_does_not_hide_the_copy() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        &ROLE_DELTA.replace(
            "The system SHALL audit role changes.\n",
            "The system SHALL audit role changes.\n\n```\n## ADDED Requirements\n```\n",
        ),
    );
    sj(dir.path()).args(["--write", "staff"]).assert().success();
    assert!(read(dir.path(), "openspec/specs/staff/spec.md")
        .contains("#### Scenario: STAFF-057 Role change is recorded"));
}

#[test]
fn ids_020_a_renamed_requirement_matches_by_its_old_name() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        "## RENAMED Requirements\n\
         - FROM: `### Requirement: Role changes are audited`\n\
         - TO: `### Requirement: Role changes are logged`\n\n\
         ## MODIFIED Requirements\n\n\
         ### Requirement: Role changes are logged\n\
         The system SHALL log role changes.\n\n\
         #### Scenario: STAFF-057 Role change is recorded\n\
         - **WHEN** a role changes\n\
         - **THEN** it is recorded with its author\n",
    );
    sj(dir.path()).args(["--write", "staff"]).assert().success();
    assert!(read(dir.path(), "openspec/specs/staff/spec.md")
        .contains("#### Scenario: STAFF-057 Role change is recorded"));
}

#[test]
fn ids_022_a_copy_under_a_renamed_requirement_gets_the_originals_id() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "openspec/specs/staff/spec.md", ROLE_SPEC);
    put(
        dir.path(),
        "openspec/changes/audit/specs/staff/spec.md",
        "## RENAMED Requirements\n\
         - FROM: `### Requirement: Role changes are audited`\n\
         - TO: `### Requirement: Role changes are logged`\n\n\
         ## MODIFIED Requirements\n\n\
         ### Requirement: Role changes are logged\n\
         The system SHALL log role changes.\n\n\
         #### Scenario: Role change is recorded\n\
         - **WHEN** a role changes\n\
         - **THEN** it is recorded with its author\n",
    );
    sj(dir.path()).args(["--write", "staff"]).assert().success();
    assert!(read(dir.path(), "openspec/specs/staff/spec.md")
        .contains("#### Scenario: STAFF-100 Role change is recorded"));
    assert!(
        read(dir.path(), "openspec/changes/audit/specs/staff/spec.md")
            .contains("#### Scenario: STAFF-100 Role change is recorded")
    );
}

#[test]
fn ids_023_a_numbered_capabilitys_prefix_is_taken() {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "openspec/specs/staff/spec.md",
        &USER_AUTH
            .replace("Scenario: Valid", "Scenario: STAFF-001 Valid")
            .replace("Scenario: Password", "Scenario: STAFF-002 Password"),
    );
    put(dir.path(), "openspec/specs/st-aff/spec.md", USER_AUTH);
    let before = snapshot(dir.path());
    for args in [&["--write"][..], &["--write", "st-aff"][..]] {
        sj(dir.path())
            .args(args)
            .assert()
            .code(2)
            .stderr(predicate::str::contains("`st-aff`").and(predicate::str::contains("`staff`")));
        assert_eq!(snapshot(dir.path()), before);
    }
}
