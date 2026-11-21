//! RUN scenarios covered by the cargo runner (specjutsu/changes/test-ledger).

use sj_runners::cargo::{cargo_test_detailed, parse_build_errors, parse_failures};

const FAILING: &str = "\
running 2 tests
test tests::adds ... ok
test gate_004_untested ... FAILED

failures:

---- gate_004_untested stdout ----
thread 'gate_004_untested' panicked at tests/gate.rs:9:5:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    gate_004_untested

test result: FAILED. 1 passed; 1 failed; 0 ignored
";

#[test]
fn run_006_failure_sections_are_parsed_per_test() {
    let f = parse_failures(FAILING);
    assert_eq!(f.len(), 1);
    let text = &f["gate_004_untested"];
    assert!(text.starts_with("thread 'gate_004_untested' panicked"));
    assert!(text.contains("right: 2"));
    assert!(
        !text.contains("failures:"),
        "section must stop at the summary: {text}"
    );
    assert!(!text.ends_with('\n'));
}

#[test]
fn run_008_build_errors_keep_error_and_location_lines() {
    let stderr = "   Compiling demo v0.1.0\nerror[E0425]: cannot find function `nope` in this scope\n --> tests/t.rs:3:5\n  |\n3 |     nope();\n  |     ^^^^ not found\n\nerror: could not compile `demo` (test \"t\") due to 1 previous error\n";
    let lines = parse_build_errors(stderr, 40);
    assert_eq!(
        lines,
        vec![
            "error[E0425]: cannot find function `nope` in this scope",
            " --> tests/t.rs:3:5",
            "error: could not compile `demo` (test \"t\") due to 1 previous error",
        ]
    );
    assert_eq!(parse_build_errors(stderr, 1).len(), 1);
}

#[test]
fn run_008_compile_failure_is_a_build_failure_not_an_empty_run() {
    let dir = tempfile::tempdir().unwrap();
    let w = |rel: &str, s: &str| {
        let p = dir.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, s).unwrap();
    };
    w(
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    w("src/lib.rs", "");
    w("tests/t.rs", "#[test]\nfn t() {\n    nope();\n}\n");
    let d = cargo_test_detailed(dir.path(), &["--workspace"], &[]).unwrap();
    assert!(d.build_failed);
    assert!(
        d.build_errors.iter().any(|l| l.starts_with("error")),
        "{:?}",
        d.build_errors
    );
    assert!(d.run.results.is_empty());
    assert!(!d.run.success);
}
