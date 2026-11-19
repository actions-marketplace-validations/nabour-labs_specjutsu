//! AUDIT scenarios from openspec/changes/audit-checks: `sj audit` on repositories with and
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
