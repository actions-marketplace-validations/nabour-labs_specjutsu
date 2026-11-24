//! A small line diff: the longest common subsequence of two texts' lines, printed as a
//! unified diff with three lines of context.

/// Lines of context around each change.
const CONTEXT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Same,
    Delete,
    Insert,
}

/// The edit script turning `a` into `b`, one op per line.
fn ops(a: &[&str], b: &[&str]) -> Vec<Op> {
    // The common prefix and suffix need no table.
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..]
        .iter()
        .rev()
        .zip(b[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (a_mid, b_mid) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    let (n, m) = (a_mid.len(), b_mid.len());
    // lcs[i][j]: the LCS length of a_mid[i..] and b_mid[j..].
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a_mid[i] == b_mid[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = vec![Op::Same; pre];
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a_mid[i] == b_mid[j] {
            out.push(Op::Same);
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push(Op::Delete);
            i += 1;
        } else {
            out.push(Op::Insert);
            j += 1;
        }
    }
    out.resize(out.len() + suf, Op::Same);
    out
}

/// The `-s,l` / `+s,l` range of a hunk starting after `before` lines and `len` lines long.
fn range(before: usize, len: usize) -> String {
    let start = if len == 0 { before } else { before + 1 };
    format!("{start},{len}")
}

/// The unified diff of `old` and `new` for `path` (`--- a/<path>`, `+++ b/<path>`, `@@`
/// hunks), or an empty string when the lines are equal. A created file is diffed against "".
pub fn unified(path: &str, old: &str, new: &str) -> String {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let ops = ops(&a, &b);
    let changed: Vec<usize> = (0..ops.len()).filter(|&k| ops[k] != Op::Same).collect();
    if changed.is_empty() {
        return String::new();
    }
    // Group changes whose context would touch or overlap into one hunk: [start, end) of ops.
    let mut groups: Vec<(usize, usize)> = Vec::new();
    for &k in &changed {
        let start = k.saturating_sub(CONTEXT);
        let end = (k + 1 + CONTEXT).min(ops.len());
        match groups.last_mut() {
            Some(last) if start <= last.1 => last.1 = end,
            _ => groups.push((start, end)),
        }
    }
    let mut out = format!("--- a/{path}\n+++ b/{path}\n");
    // Lines of each side before op k.
    let (mut ai, mut bi, mut k) = (0, 0, 0);
    for (start, end) in groups {
        while k < start {
            match ops[k] {
                Op::Same => {
                    ai += 1;
                    bi += 1;
                }
                Op::Delete => ai += 1,
                Op::Insert => bi += 1,
            }
            k += 1;
        }
        let old_len = ops[start..end].iter().filter(|o| **o != Op::Insert).count();
        let new_len = ops[start..end].iter().filter(|o| **o != Op::Delete).count();
        out.push_str(&format!(
            "@@ -{} +{} @@\n",
            range(ai, old_len),
            range(bi, new_len)
        ));
        while k < end {
            match ops[k] {
                Op::Same => {
                    out.push_str(&format!(" {}\n", a[ai]));
                    ai += 1;
                    bi += 1;
                }
                Op::Delete => {
                    out.push_str(&format!("-{}\n", a[ai]));
                    ai += 1;
                }
                Op::Insert => {
                    out.push_str(&format!("+{}\n", b[bi]));
                    bi += 1;
                }
            }
            k += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::unified;

    fn numbered(range: std::ops::RangeInclusive<u32>) -> String {
        range.map(|n| format!("{n}\n")).collect()
    }

    /// `text` with each whole line equal to `from` replaced by `to`.
    fn swap(text: &str, from: &str, to: &str) -> String {
        text.lines()
            .map(|l| format!("{}\n", if l == from { to } else { l }))
            .collect()
    }

    #[test]
    fn identical_input_has_no_hunks() {
        assert_eq!(unified("f", "a\nb\n", "a\nb\n"), "");
        assert_eq!(unified("f", "", ""), "");
    }

    #[test]
    fn pure_insert() {
        assert_eq!(
            unified("f", "a\nb\n", "a\nx\nb\n"),
            "--- a/f\n+++ b/f\n@@ -1,2 +1,3 @@\n a\n+x\n b\n"
        );
    }

    #[test]
    fn pure_delete() {
        assert_eq!(
            unified("f", "a\nx\nb\n", "a\nb\n"),
            "--- a/f\n+++ b/f\n@@ -1,3 +1,2 @@\n a\n-x\n b\n"
        );
    }

    #[test]
    fn a_created_file_is_diffed_against_nothing() {
        assert_eq!(
            unified("new.md", "", "one\ntwo\n"),
            "--- a/new.md\n+++ b/new.md\n@@ -0,0 +1,2 @@\n+one\n+two\n"
        );
    }

    #[test]
    fn a_change_keeps_three_lines_of_context() {
        let old = numbered(1..=10);
        let new = swap(&old, "5", "five");
        assert_eq!(
            unified("f", &old, &new),
            "--- a/f\n+++ b/f\n@@ -2,7 +2,7 @@\n 2\n 3\n 4\n-5\n+five\n 6\n 7\n 8\n"
        );
    }

    #[test]
    fn close_changes_share_a_hunk_and_far_ones_do_not() {
        let old = numbered(1..=20);
        // 3 and 9 have five lines between them: their contexts overlap, so one hunk.
        let near = swap(&swap(&old, "3", "c"), "9", "i");
        let d = unified("f", &old, &near);
        assert_eq!(d.matches("@@ -").count(), 1, "{d}");
        assert!(d.contains("@@ -1,12 +1,12 @@\n"), "{d}");
        // 3 and 17 are far apart: two hunks.
        let far = swap(&swap(&old, "3", "c"), "17", "q");
        let d = unified("f", &old, &far);
        assert_eq!(d.matches("@@ -").count(), 2, "{d}");
        assert!(d.contains("@@ -1,6 +1,6 @@\n"), "{d}");
        assert!(d.contains("@@ -14,7 +14,7 @@\n"), "{d}");
    }
}
