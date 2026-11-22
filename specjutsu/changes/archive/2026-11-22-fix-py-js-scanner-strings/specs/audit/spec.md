## ADDED Requirements

### Requirement: Python and JS test bodies are read past multi-line strings
The Python scanner SHALL end a test at the first line indented no deeper than its `def` that is
not inside a triple-quoted string. The JS/TS scanner SHALL count the newline a `\` continues a
quoted string over, so test line numbers after it stay right.

#### Scenario: AUDIT-019 A column-0 triple-quoted fixture keeps a pytest test's asserts
- **WHEN** the change adds `def test_parses():` to `tests/test_cfg.py` whose body holds `data = """` and then `[tool]`, `name = "x"` and `"""` at column 0, then `    assert "x" in data`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-020 A backslash-continued JS string does not shift later tests
- **WHEN** the change adds to `src/a.test.js` `it("first", …)` holding `const s = "a\` and `b"` on two lines with `expect(s).toBe("ab")`, then `it("second", () => {` with `const y = 2` and `expect(y).toBe(2) })`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-021 [keep] A pytest test still ends at the next top-level def
- **WHEN** the change adds `def test_a():` with body `x = 1`, then `def test_b():` with body `assert True`, to `tests/test_cfg.py`
- **THEN** `sj audit` reports `assertion-free-test tests/test_cfg.py › test_a` and no finding for `test_b`
