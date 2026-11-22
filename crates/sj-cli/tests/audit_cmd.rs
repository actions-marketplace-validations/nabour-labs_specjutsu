//! AUDIT scenarios from specjutsu/changes/audit-checks: `sj audit` on repositories with and
//! without a specjutsu setup.

use std::path::Path;
use std::process::Command as Std;

use assert_cmd::Command;

/// A git repository with a `main` branch; edits after [`Fixture::branch`] form the change.
pub struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    pub fn new(files: &[(&str, &str)]) -> Self {
        let f = Fixture {
            dir: tempfile::tempdir().unwrap(),
        };
        f.git(&["init", "-q", "-b", "main"]);
        for (p, c) in files {
            f.write(p, c);
        }
        f.commit("init");
        f.git(&["checkout", "-q", "-b", "feature"]);
        f
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, rel: &str, content: &str) {
        let p = self.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path().join(rel)).unwrap();
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Std::new("git")
            .arg("-C")
            .arg(self.path())
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

    pub fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", msg]);
    }

    /// Run `sj audit --base main <extra>`; returns (exit code, stdout).
    pub fn audit(&self, extra: &[&str]) -> (Option<i32>, String) {
        let out = Command::cargo_bin("sj")
            .unwrap()
            .arg("audit")
            .arg("--root")
            .arg(self.path())
            .args(["--base", "main"])
            .args(extra)
            .output()
            .unwrap();
        (
            out.status.code(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
        )
    }
}

const CART_TEST: &str = r#"import { total } from "./cart"

it("adds tax", () => {
  expect(total([100])).toBe(108)
})

it("sums items", () => {
  expect(total([1, 2])).toBe(3.24)
})
"#;

#[test]
fn audit_001_audit_a_repository_with_no_specjutsu_setup() {
    let f = Fixture::new(&[
        ("package.json", "{\"name\":\"shop\"}\n"),
        ("src/cart.ts", "export const total = (xs: number[]) => 0\n"),
        ("src/cart.test.ts", CART_TEST),
    ]);
    f.write(
        "src/cart.test.ts",
        &CART_TEST.replace(
            "it(\"adds tax\", () => {\n  expect(total([100])).toBe(108)\n})\n\n",
            "",
        ),
    );
    f.commit("drop the failing test");

    let (code, out) = f.audit(&[]);
    assert!(
        out.contains("removed-test src/cart.test.ts › adds tax"),
        "{out}"
    );
    assert_eq!(code, Some(1), "{out}");
}

const RUST_A: &str = "#[test]\nfn parses_header() {\n    let h = parse(\"GET / HTTP/1.1\");\n    assert_eq!(h.method, \"GET\");\n}\n";

#[test]
fn audit_002_moving_or_renaming_a_test_is_not_a_removal() {
    let f = Fixture::new(&[
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        ("tests/a.rs", RUST_A),
    ]);
    f.remove("tests/a.rs");
    f.write(
        "tests/b.rs",
        &RUST_A.replace("parses_header", "parses_the_header"),
    );
    f.commit("move test");

    let (code, out) = f.audit(&[]);
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_012_a_clean_change_prints_one_line() {
    let f = Fixture::new(&[
        ("package.json", "{\"name\":\"shop\"}\n"),
        ("src/cart.ts", "export const total = (xs: number[]) => 0\n"),
        ("src/cart.test.ts", CART_TEST),
    ]);
    f.write(
        "src/cart.ts",
        "export const total = (xs: number[]) => xs.reduce((a, b) => a + b, 0) * 1.08\n",
    );
    f.write(
        "src/cart.test.ts",
        &format!(
            "{CART_TEST}\nit(\"handles empty\", () => {{\n  expect(total([])).toBe(0)\n  expect(total([0])).toBe(0)\n}})\n"
        ),
    );
    f.commit("feat: tax");

    let (code, out) = f.audit(&[]);
    assert_eq!(out, "audit: clean\n");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_003_a_newly_skipped_test_is_reported() {
    let auth = "it(\"rejects expired token\", () => {\n  expect(check(old)).toBe(401)\n})\n";
    let rust = "#[test]\nfn trace_004_matches_ids() {\n    assert!(true);\n}\n";
    let f = Fixture::new(&[("src/auth.test.ts", auth), ("tests/trace.rs", rust)]);
    f.write("src/auth.test.ts", &auth.replace("it(", "it.skip("));
    f.write(
        "tests/trace.rs",
        &rust.replace("#[test]\n", "#[test]\n#[ignore]\n"),
    );
    f.commit("skip it");

    let (code, out) = f.audit(&[]);
    assert!(
        out.contains("skipped-test src/auth.test.ts › rejects expired token"),
        "{out}"
    );
    assert!(
        out.contains("skipped-test tests/trace.rs › trace_004_matches_ids"),
        "{out}"
    );
    assert_eq!(code, Some(1), "{out}");
}

#[test]
fn audit_004_a_newly_focused_test_is_reported() {
    let mut cart = String::new();
    for name in ["cart", "tax", "shipping", "coupons", "refunds"] {
        cart.push_str(&format!(
            "describe(\"{name}\", () => {{\n  it(\"works\", () => {{\n    expect(1).toBe(1)\n  }})\n  it(\"still works\", () => {{\n    expect(2).toBe(2)\n  }})\n}})\n\n"
        ));
    }
    let f = Fixture::new(&[("src/cart.test.ts", &cart)]);
    f.write(
        "src/cart.test.ts",
        &cart.replacen("describe(\"cart\"", "describe.only(\"cart\"", 1),
    );
    f.commit("focus");

    let (code, out) = f.audit(&[]);
    let focused: Vec<&str> = out.lines().filter(|l| l.contains("focused-test")).collect();
    assert_eq!(focused.len(), 1, "{out}");
    assert!(
        focused[0].ends_with("focused-test src/cart.test.ts › cart"),
        "{out}"
    );
    assert_eq!(code, Some(1));
}

#[test]
fn audit_005_removing_assertions_weakens_a_test() {
    let before = "def test_total_with_discount():\n    cart = Cart([10, 20])\n    assert cart.total() == 30\n    assert cart.discount(0.1) == 27\n    assert cart.items == 2\n    assert cart.currency == \"EUR\"\n    assert not cart.empty\n";
    let after = "def test_total_with_discount():\n    cart = Cart([10, 20])\n    assert cart.total() == 30\n    assert cart.items == 2\n";
    let f = Fixture::new(&[("tests/test_cart.py", before)]);
    f.write("tests/test_cart.py", after);
    f.commit("simplify");

    let (code, out) = f.audit(&[]);
    assert!(
        out.contains(
            "weakened-test tests/test_cart.py › test_total_with_discount (5 → 2 assertions)"
        ),
        "{out}"
    );
    assert_eq!(code, Some(1));
}

#[test]
fn audit_006_a_new_test_without_assertions_is_reported() {
    let f = Fixture::new(&[("tests/cart.rs", "use demo::total;\n")]);
    f.write(
        "tests/cart.rs",
        "use demo::total;\n\n#[test]\nfn computes_total() { let _ = total(&cart); }\n\n#[test]\nfn totals_match() { assert_eq!(total(&cart), 3); }\n",
    );
    f.commit("add tests");

    let (code, out) = f.audit(&[]);
    assert!(
        out.contains("assertion-free-test tests/cart.rs › computes_total"),
        "{out}"
    );
    assert!(!out.contains("totals_match"), "{out}");
    assert_eq!(code, Some(1));
}

#[test]
fn audit_007_a_configured_assertion_helper_counts() {
    let config = "schema = 1\n\n[audit]\nassert_helpers = [\"assert_snapshot\"]\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n[[package.suite]]\nrunner = \"vitest\"\ntests = [\"src/**/*.test.ts\"]\n";
    let f = Fixture::new(&[
        ("specjutsu.toml", config),
        ("src/view.test.ts", "import { render } from \"./view\"\n"),
    ]);
    f.write(
        "src/view.test.ts",
        "import { render } from \"./view\"\n\nit(\"renders\", () => {\n  const out = render()\n  assert_snapshot(out)\n})\n",
    );
    f.commit("snapshot test");

    let (code, out) = f.audit(&[]);
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_014_an_assert_cmd_chain_counts_as_an_assertion() {
    let f = Fixture::new(&[("tests/cli.rs", "use assert_cmd::Command;\n")]);
    f.write(
        "tests/cli.rs",
        "use assert_cmd::Command;\n\n#[test]\nfn runs() { Command::cargo_bin(\"x\").unwrap().assert().success(); }\n",
    );
    f.commit("assert_cmd test");

    let (code, out) = f.audit(&[]);
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_008_editing_the_expected_value_next_to_a_source_change_warns() {
    let test =
        "it(\"adds tax\", () => {\n  const total = cart([100])\n  expect(total).toBe(108)\n})\n";
    let files = [
        (
            "src/cart.ts",
            "export const cart = (xs: number[]) => xs[0] * 1.08\n",
        ),
        ("src/cart.test.ts", test),
    ];
    let f = Fixture::new(&files);
    f.write(
        "src/cart.ts",
        "export const cart = (xs: number[]) => xs[0] * 1.1\n",
    );
    f.write("src/cart.test.ts", &test.replace("108", "110"));
    f.commit("raise tax");

    let (code, out) = f.audit(&[]);
    let line = out
        .lines()
        .find(|l| l.contains("rewritten-expectation"))
        .unwrap_or_else(|| panic!("no warning in:\n{out}"));
    assert!(line.starts_with("warn"), "{line}");
    assert!(
        line.ends_with("rewritten-expectation src/cart.test.ts › adds tax: 108 → 110"),
        "{line}"
    );
    assert_eq!(code, Some(0), "{out}");

    // The same edit without a source change is a spec-driven fix, not a rewrite.
    let g = Fixture::new(&files);
    g.write("src/cart.test.ts", &test.replace("108", "110"));
    g.commit("fix expectation");
    let (_, out) = g.audit(&[]);
    assert!(!out.contains("rewritten-expectation"), "{out}");
}

#[test]
fn audit_011_an_allowed_removal_is_reported_not_failed() {
    let export = "def test_legacy_export():\n    assert export_v1() == b\"\"\n\ndef test_export():\n    assert export() == b\"\"\n";
    let kept = "def test_export():\n    assert export() == b\"\"\n";
    let f = Fixture::new(&[("tests/test_export.py", export)]);
    f.write("tests/test_export.py", kept);
    f.commit("remove legacy export\n\nSpecjutsu-Allow: removed-test tests/test_export.py › test_legacy_export — legacy export removed in #212");

    let (code, out) = f.audit(&[]);
    assert!(
        out.contains("allowed removed-test tests/test_export.py › test_legacy_export (legacy export removed in #212)"),
        "{out}"
    );
    assert!(!out.contains("fail"), "{out}");
    assert_eq!(code, Some(0), "{out}");

    // A trailer without a reason is itself a failure.
    let g = Fixture::new(&[("tests/test_export.py", export)]);
    g.write("tests/test_export.py", kept);
    g.commit("remove legacy export\n\nSpecjutsu-Allow: removed-test tests/test_export.py › test_legacy_export");
    let (code, out) = g.audit(&[]);
    assert!(out.contains("allow without reason"), "{out}");
    assert_eq!(code, Some(1));
}

/// Commit `body` as `tests/cfg.rs` on the change branch and run `sj audit`.
fn audit_rust_test(body: &str) -> (Option<i32>, String) {
    let f = Fixture::new(&[("tests/cfg.rs", "use std::fmt;\n")]);
    f.write("tests/cfg.rs", &format!("use std::fmt;\n\n{body}"));
    f.commit("add test");
    f.audit(&[])
}

#[test]
fn audit_015_a_multi_line_raw_string_fixture_keeps_the_tests_asserts() {
    let (code, out) = audit_rust_test(
        "#[test]\nfn parses() {\n    let j = r#\"{\n  \"k\": \"v \\\" }\"\n}\"#;\n    assert!(j.contains(\"k\"));\n}\n",
    );
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_016_a_string_continued_with_a_backslash_keeps_the_tests_asserts() {
    let (code, out) = audit_rust_test(
        "#[test]\nfn reads() {\n    let t = \"{\\\n    \\\"x\\\": 1\\\n}\";\n    assert!(t.contains(\"x\"));\n}\n",
    );
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_017_char_literals_of_braces_and_quotes_do_not_move_the_body_end() {
    let (code, out) = audit_rust_test(
        "#[test]\nfn chars() {\n    let cs = ['{', '\"', '}'];\n    assert_eq!(cs.len(), 3);\n}\n",
    );
    assert_eq!(out.trim(), "audit: clean", "{out}");
    assert_eq!(code, Some(0));
}

#[test]
fn audit_018_a_fixture_only_test_stays_assertion_free() {
    let (code, out) = audit_rust_test(
        "#[test]\nfn only_fixture() {\n    let _j = r#\"{\n}\"#;\n}\n\n#[test]\nfn checks() { assert!(true); }\n",
    );
    assert!(
        out.contains("assertion-free-test tests/cfg.rs › only_fixture"),
        "{out}"
    );
    assert!(!out.contains("checks"), "{out}");
    assert_eq!(code, Some(1));
}
