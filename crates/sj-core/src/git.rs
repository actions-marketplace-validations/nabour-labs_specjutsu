//! Thin wrapper over the `git` CLI.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// A commit's subject and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub subject: String,
    pub body: String,
}

pub struct Git {
    root: PathBuf,
}

impl Git {
    pub fn new(root: &Path) -> Self {
        Git {
            root: root.to_path_buf(),
        }
    }

    /// Run git with `args` in the repo root and return stdout.
    pub fn run(&self, args: &[&str]) -> Result<String> {
        self.run_env(args, &[])
    }

    /// Like `run`, with extra environment variables.
    pub fn run_env(&self, args: &[&str], env: &[(&str, &OsStr)]) -> Result<String> {
        let out = Command::new("git")
            .args(args)
            .envs(env.iter().copied())
            .current_dir(&self.root)
            .output()
            .context("running git (is it installed and on PATH?)")?;
        if !out.status.success() {
            bail!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Like `run`, writing `input` to git's stdin: for path lists too long for argv.
    pub fn run_stdin(&self, args: &[&str], input: &str) -> Result<String> {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("running git (is it installed and on PATH?)")?;
        let mut stdin = child.stdin.take().context("opening git's stdin")?;
        let input = input.to_string();
        // Written from a thread so a large input can't deadlock against a full stdout pipe.
        let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
        let out = child.wait_with_output().context("running git")?;
        writer
            .join()
            .map_err(|_| anyhow::anyhow!("writing to git's stdin panicked"))?
            .context("writing to git's stdin")?;
        if !out.status.success() {
            bail!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Resolve a ref to a commit SHA; errors name the ref and suggest a full fetch.
    pub fn resolve(&self, rev: &str) -> Result<String> {
        let spec = format!("{rev}^{{commit}}");
        match self.run(&["rev-parse", "--verify", "--quiet", &spec]) {
            Ok(sha) if !sha.trim().is_empty() => Ok(sha.trim().to_string()),
            _ => bail!(
                "base ref `{rev}` not found in this repository; fetch it first \
                 (in GitHub Actions use actions/checkout with `fetch-depth: 0`)"
            ),
        }
    }

    pub fn merge_base(&self, a: &str, b: &str) -> Result<String> {
        Ok(self.run(&["merge-base", a, b])?.trim().to_string())
    }

    /// Files that differ between `base` and the working tree, plus untracked files (repo-relative).
    pub fn changed_files(&self, base: &str) -> Result<Vec<String>> {
        let mut files: Vec<String> = self
            .run(&["diff", "--name-only", "--no-renames", base])?
            .lines()
            .chain(
                self.run(&["ls-files", "--others", "--exclude-standard"])?
                    .lines(),
            )
            .map(str::to_string)
            .collect();
        files.sort();
        files.dedup();
        Ok(files)
    }

    /// `git diff -U0` hunks of `path` from `from` to `to` (the working tree when `None`), as
    /// `(old_start, old_len, new_start, new_len)`.
    pub fn hunks(&self, from: &str, to: Option<&str>, path: &str) -> Result<Vec<Hunk>> {
        let mut args = vec!["diff", "-U0", "--no-color", "--no-renames", from];
        if let Some(t) = to {
            args.push(t);
        }
        args.extend(["--", path]);
        Ok(parse_hunks(&self.run(&args)?))
    }

    /// Every file path in the tree of `rev`.
    pub fn ls_tree(&self, rev: &str) -> Result<Vec<String>> {
        Ok(self
            .run(&["ls-tree", "-r", "--name-only", rev])?
            .lines()
            .map(str::to_string)
            .collect())
    }

    /// Contents of `path` at `rev`.
    pub fn show(&self, rev: &str, path: &str) -> Result<String> {
        self.run(&["show", &format!("{rev}:{path}")])
    }

    /// Commits in `range`, oldest first.
    pub fn log(&self, range: &str) -> Result<Vec<Commit>> {
        let raw = self.run(&["log", "--reverse", "--format=%s%x00%b%x1e", range])?;
        Ok(raw
            .split('\x1e')
            .filter(|c| !c.trim().is_empty())
            .map(|c| {
                let (subject, body) = c.split_once('\0').unwrap_or((c, ""));
                Commit {
                    subject: subject.trim().to_string(),
                    body: body.trim().to_string(),
                }
            })
            .collect())
    }

    pub fn worktree_add(&self, path: &Path, rev: &str) -> Result<()> {
        let p = path.to_string_lossy();
        self.run(&["worktree", "add", "--quiet", "--detach", &p, rev])
            .map(|_| ())
    }

    pub fn worktree_remove(&self, path: &Path) -> Result<()> {
        let p = path.to_string_lossy();
        self.run(&["worktree", "remove", "--force", &p])?;
        self.run(&["worktree", "prune"]).map(|_| ())
    }

    /// Whether the root is inside a git work tree.
    pub fn is_repo(&self) -> bool {
        self.run(&["rev-parse", "--is-inside-work-tree"])
            .is_ok_and(|o| o.trim() == "true")
    }

    /// Tree hash of the working tree as it is now: tracked, modified and untracked-but-not-ignored
    /// files. Uses a throwaway copy of the index, so the user's index and files are never touched.
    pub fn worktree_tree_hash(&self) -> Result<String> {
        let real = self.run(&["rev-parse", "--git-path", "index"])?;
        let real = self.root.join(real.trim());
        let dir = tempfile::tempdir().context("creating a temporary index directory")?;
        let tmp = dir.path().join("index");
        if real.is_file() {
            std::fs::copy(&real, &tmp).context("copying the git index")?;
        }
        let env = [("GIT_INDEX_FILE", tmp.as_os_str())];
        self.run_env(&["add", "-A", "--", ":/"], &env)?;
        Ok(self.run_env(&["write-tree"], &env)?.trim().to_string())
    }
}

/// One `@@ -a,b +c,d @@` hunk header: `old_len` 0 is a pure insertion after `old_start`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hunk {
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
}

impl Hunk {
    /// Base lines the hunk modifies or deletes.
    pub fn old_lines(&self) -> std::ops::Range<usize> {
        self.old_start..self.old_start + self.old_len
    }
}

/// Parse the hunk headers of a unified diff.
pub fn parse_hunks(diff: &str) -> Vec<Hunk> {
    let side = |s: &str| -> Option<(usize, usize)> {
        let (start, len) = match s.split_once(',') {
            Some((a, b)) => (a.parse().ok()?, b.parse().ok()?),
            None => (s.parse().ok()?, 1),
        };
        Some((start, len))
    };
    diff.lines()
        .filter_map(|l| l.strip_prefix("@@ -"))
        .filter_map(|rest| {
            let (old, rest) = rest.split_once(" +")?;
            let new = rest.split_once(' ').map(|(n, _)| n).unwrap_or(rest);
            let (old_start, old_len) = side(old)?;
            let (new_start, new_len) = side(new)?;
            Some(Hunk {
                old_start,
                old_len,
                new_start,
                new_len,
            })
        })
        .collect()
}
