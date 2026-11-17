use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// specjutsu — make AI coding agents prove their work.
#[derive(Parser)]
#[command(name = "sj", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Map spec scenario IDs to tests; list gaps and orphans.
    Trace {
        /// Repository root (defaults to the current directory).
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Spec root relative to the repository root (overrides specjutsu.toml; default openspec).
        #[arg(long)]
        spec_root: Option<PathBuf>,
        /// Print JSON instead of the compact report.
        #[arg(long)]
        json: bool,
        /// Exit 1 when there are gaps or orphans.
        #[arg(long)]
        strict: bool,
    },
    /// CI check: every changed scenario tested, red on base, green on head.
    Gate {
        /// Base ref to compare against, e.g. origin/main.
        #[arg(long, required_unless_present = "since")]
        base: Option<String>,
        /// Only check commit history: red commit before green, since this commit.
        #[arg(long, conflicts_with = "base")]
        since: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        /// Warn instead of failing when a test cannot be overlaid on base.
        #[arg(long)]
        allow_unverifiable: bool,
        #[arg(long)]
        json: bool,
    },
    /// Run the tests (all, one package, or one scenario's); print failures only; record the run.
    Test {
        /// A scenario ID (e.g. GATE-004) or a Cargo package name. Omit to run everything.
        scope: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
        /// Do not write the run to the evidence ledger.
        #[arg(long)]
        no_ledger: bool,
    },
    /// Start a project: write specjutsu.toml and create the spec folders.
    Init {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Replace an existing specjutsu.toml.
        #[arg(long)]
        force: bool,
        /// Install agent adapters (comma-separated; known: claude).
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
        /// Remove agent adapters (comma-separated; known: claude).
        #[arg(long, value_delimiter = ',', conflicts_with = "agents")]
        remove_agents: Vec<String>,
    },
    /// Hook entrypoint for agent tools: `sj hook claude session-start|pre-edit`.
    Hook {
        /// The agent tool (claude).
        agent: String,
        /// The hook event: session-start or pre-edit.
        event: String,
        /// Project root (default: $CLAUDE_PROJECT_DIR, else the current directory).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Regenerate installed agent adapters; hand-edited files are kept unless --force.
    Update {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Replace generated files even when they were edited by hand.
        #[arg(long)]
        force: bool,
    },
    /// Detect an existing project's packages and runners and write specjutsu.toml.
    Adopt {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Print the config instead of writing it.
        #[arg(long)]
        dry_run: bool,
        /// Replace an existing specjutsu.toml.
        #[arg(long)]
        force: bool,
    },
    /// Check the setup: project file, packages, runner executables and spec folders.
    Doctor {
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Create, list, validate, check and archive spec changes.
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
    },
    /// List scenarios without IDs and the IDs they would get; --write assigns them in place.
    Ids {
        /// Insert the IDs into the spec files.
        #[arg(long)]
        write: bool,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Where each open change stands: tasks, scenario evidence and the next task.
    Status {
        /// Only this change (default: every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// The next task's context: its scenarios, changed files and test commands.
    Next {
        /// Only this change (default: the first open change with an open task).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// List open [P] tasks in batches that share no test file.
        #[arg(long)]
        parallel: bool,
        /// Base ref for changed files (default: origin/HEAD, origin/main, main, ...).
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Local check before a PR: the gate's checks without red-on-base.
    Verify {
        /// Only this change (default: every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// Base ref (default: origin/HEAD, origin/main, origin/master, main, master).
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Check off tasks in a change's tasks.md.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Per-scenario evidence from recorded runs: latest result and red-before-green.
    Ledger {
        /// Report only this scenario ID.
        id: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(clap::Args)]
struct SpecArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Spec root relative to the repository root (overrides specjutsu.toml; default openspec).
    #[arg(long)]
    spec_root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum SpecCommand {
    /// Scaffold changes/<change>/ with proposal, design, tasks and specs/.
    New {
        /// Kebab-case change name.
        change: String,
        #[command(flatten)]
        args: SpecArgs,
    },
    /// Open changes with task progress and scenario counts.
    List {
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Check spec format, delta structure and that every task names its scenarios.
    Validate {
        /// Only this change (default: specs/ and every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// Fail on warnings too.
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        json: bool,
    },
    /// Scenarios no task covers, tasks naming unknown IDs, and open questions.
    Check {
        change: String,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Apply the change's deltas to specs/ and move it to changes/archive/.
    Archive {
        change: String,
        #[command(flatten)]
        args: SpecArgs,
        /// Archive despite validation errors and open tasks.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum TaskCommand {
    /// Check a task's box; strict mode requires red-before-green for its scenarios.
    Done {
        change: String,
        /// The task's number, e.g. 2.1.
        task: String,
        #[command(flatten)]
        args: SpecArgs,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("sj: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::Gate {
            base,
            since,
            root,
            spec_root,
            allow_unverifiable,
            json,
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            let report = match (base, since) {
                (_, Some(since)) => sj_core::gate::history(&root, &since)?,
                (Some(base), None) => sj_core::gate::run(&sj_core::gate::Options {
                    root,
                    spec_root,
                    base,
                    allow_unverifiable,
                })?,
                (None, None) => anyhow::bail!("pass --base <ref> or --since <commit>"),
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.summary);
                for f in &report.failures {
                    println!("fail    {f}");
                }
                for w in &report.warnings {
                    println!("warn    {w}");
                }
                for e in &report.exemptions {
                    println!("exempt  {e}");
                }
            }
            Ok(if report.pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Test {
            scope,
            root,
            spec_root,
            json,
            no_ledger,
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            test_cmd(scope, root, spec_root, json, no_ledger)
        }
        Command::Init {
            root,
            force,
            agents,
            remove_agents,
        } => {
            if !remove_agents.is_empty() {
                return remove_agents_cmd(&root, &remove_agents);
            }
            if agents.is_empty() {
                return init_cmd(&root, force);
            }
            check_agents(&agents)?;
            if force || !root.join(sj_core::config::FILE).exists() {
                init_cmd(&root, force)?;
            }
            agents_cmd(&root, false, false)
        }
        Command::Hook { agent, event, root } => Ok(hook_cmd(&agent, &event, root)),
        Command::Update { root, force } => {
            if !sj_adapters::claude::is_installed(&root) {
                println!("no adapters installed");
                return Ok(ExitCode::SUCCESS);
            }
            agents_cmd(&root, force, true)
        }
        Command::Doctor { root } => {
            let problems = sj_core::doctor::check(&root);
            if problems.is_empty() {
                println!("doctor: ok");
                return Ok(ExitCode::SUCCESS);
            }
            println!("doctor: {}", plural(problems.len(), "problem"));
            for p in &problems {
                println!("{p}");
            }
            Ok(ExitCode::from(1))
        }
        Command::Adopt {
            root,
            dry_run,
            force,
        } => adopt_cmd(&root, dry_run, force),
        Command::Spec { command } => spec_cmd(command),
        Command::Ids {
            write,
            root,
            spec_root,
            json,
        } => ids_cmd(write, &root, spec_root, json),
        Command::Status { change, args, json } => status_cmd(change, &args, json),
        Command::Task {
            command: TaskCommand::Done { change, task, args },
        } => task_done_cmd(&change, &task, &args),
        Command::Next {
            change,
            args,
            parallel,
            base,
            json,
        } => next_cmd(change, &args, parallel, base, json),
        Command::Verify {
            change,
            args,
            base,
            json,
        } => verify_cmd(change, &args, base, json),
        Command::Ledger { id, root, json } => {
            sj_core::config::load(&root)?;
            ledger_cmd(id, root, json)
        }
        Command::Trace {
            root,
            spec_root,
            json,
            strict,
        } => {
            let config = sj_core::config::load(&root)?;
            let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
            let scenarios = sj_core::spec::load_scenarios(&root.join(&spec_root))?;
            let scan = sj_core::testscan::scan_project(&root, &config)?;
            let mut report = sj_core::trace::build_report(&scenarios, &scan.tests);
            report.warnings = scan
                .unsuited
                .iter()
                .map(|p| format!("no test suite: {p}"))
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_compact(&report, &root);
            }
            Ok(if strict && !report.is_clean() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn init_cmd(root: &Path, force: bool) -> Result<ExitCode> {
    use sj_core::config::{Config, FILE};

    let file = root.join(FILE);
    if file.exists() && !force {
        anyhow::bail!("{FILE} already exists; pass --force to replace it");
    }
    let config = Config::default_for(root);
    std::fs::write(&file, config.to_toml())?;
    let spec_root = root.join(&config.spec_root);
    for dir in ["specs", "changes"] {
        std::fs::create_dir_all(spec_root.join(dir))?;
    }
    println!(
        "init: wrote {FILE} (spec root {}, mode {}, {})",
        config.spec_root,
        config.mode.as_str(),
        plural(config.packages.len(), "package")
    );
    Ok(ExitCode::SUCCESS)
}

/// Agent hooks never fail the tool call on an internal error: they warn and allow.
fn hook_cmd(agent: &str, event: &str, root: Option<PathBuf>) -> ExitCode {
    use sj_core::hook::{self, Decision};
    let root = root
        .or_else(|| std::env::var_os("CLAUDE_PROJECT_DIR").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    if agent != "claude" {
        eprintln!("specjutsu: hook error: unknown agent `{agent}`");
        return ExitCode::SUCCESS;
    }
    match event {
        "session-start" => match hook::session_start(&root) {
            Ok(text) => print!("{text}"),
            Err(e) => println!("specjutsu: {e:#}"),
        },
        "pre-edit" => {
            let mut payload = String::new();
            let decided = std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload)
                .map_err(anyhow::Error::from)
                .and_then(|_| hook::pre_edit(&root, &payload));
            match decided {
                Ok(Decision::Allow) => {}
                Ok(Decision::Warn(reasons)) => {
                    for r in reasons {
                        println!("specjutsu: {r}");
                    }
                }
                Ok(Decision::Block(reasons)) => {
                    for r in reasons {
                        eprintln!("{r}");
                    }
                    return ExitCode::from(2);
                }
                Err(e) => eprintln!("specjutsu: hook error: {e:#}"),
            }
        }
        _ => eprintln!("specjutsu: hook error: unknown event `{event}`"),
    }
    ExitCode::SUCCESS
}

const AGENTS: [&str; 1] = ["claude"];

fn check_agents(agents: &[String]) -> Result<()> {
    for a in agents {
        if !AGENTS.contains(&a.as_str()) {
            anyhow::bail!("unknown agent `{a}` (known: {})", AGENTS.join(", "));
        }
    }
    Ok(())
}

/// Install or refresh the Claude Code adapter and print what changed.
fn agents_cmd(root: &Path, force: bool, update: bool) -> Result<ExitCode> {
    use sj_adapters::claude::{self, Change};
    let changes = claude::install(root, force)?;
    if changes.is_empty() {
        println!("adapters up to date");
    }
    for c in &changes {
        println!("{c}");
    }
    let kept = changes.iter().any(|c| matches!(c, Change::Kept(_)));
    Ok(if kept && update {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn remove_agents_cmd(root: &Path, agents: &[String]) -> Result<ExitCode> {
    check_agents(agents)?;
    let changes = sj_adapters::claude::remove(root)?;
    if changes.is_empty() {
        println!("no adapter files to remove");
    }
    for c in &changes {
        println!("{c}");
    }
    Ok(ExitCode::SUCCESS)
}

fn adopt_cmd(root: &Path, dry_run: bool, force: bool) -> Result<ExitCode> {
    use sj_core::config::FILE;

    let file = root.join(FILE);
    if !dry_run && file.exists() && !force {
        anyhow::bail!("{FILE} already exists; pass --force to replace it, or --dry-run to compare");
    }
    let detected = sj_core::adopt::detect(root)?;
    let config = &detected.config;
    if dry_run {
        print!("{}", config.to_toml());
    } else {
        std::fs::write(&file, config.to_toml())?;
        println!(
            "adopt: wrote {FILE} (spec root {}, {})",
            config.spec_root,
            plural(config.packages.len(), "package")
        );
        for p in &config.packages {
            let runners: Vec<_> = p.suites.iter().map(|s| s.runner.as_str()).collect();
            let runners = if runners.is_empty() {
                "-".to_string()
            } else {
                runners.join(", ")
            };
            println!("package {}  {runners}", p.path);
        }
    }
    for n in &detected.notes {
        eprintln!("note    {n}");
    }
    Ok(ExitCode::SUCCESS)
}

/// Load the project file (failing on a bad one) and pick the spec root: the flag, then the file.
fn spec_root_for(root: &Path, flag: Option<PathBuf>) -> Result<PathBuf> {
    let config = sj_core::config::load(root)?;
    Ok(flag.unwrap_or_else(|| PathBuf::from(config.spec_root)))
}

fn test_cmd(
    scope: Option<String>,
    root: PathBuf,
    spec_root: PathBuf,
    json: bool,
    no_ledger: bool,
) -> Result<ExitCode> {
    use sj_core::ledger::{self, Recorded};
    use sj_core::run::{run_tests, Scope};

    let run = run_tests(&root, &spec_root, &Scope::parse(scope.as_deref()))?;
    if !no_ledger {
        match ledger::record(&root, &run) {
            Ok(Recorded::Records(_)) => {}
            Ok(Recorded::NotGit) => eprintln!("warn    not a git repository; run not recorded"),
            Err(e) => eprintln!("warn    run not recorded: {e:#}"),
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&run)?);
    } else if run.build_failed {
        if run.suites.len() > 1 {
            let names: Vec<&str> = run.suites.iter().map(|s| s.name.as_str()).collect();
            println!("test: fail — build failed ({})", names.join(", "));
        } else {
            println!("test: fail — build failed");
        }
        for l in &run.build_errors {
            println!("        {l}");
        }
    } else {
        let mut counts = Vec::new();
        if run.failed > 0 {
            counts.push(format!("{} failed", run.failed));
        }
        counts.push(format!("{} passed", run.passed));
        if run.ignored > 0 {
            counts.push(format!("{} ignored", run.ignored));
        }
        let verdict = if run.pass { "pass" } else { "fail" };
        let failed: Vec<&str> = run
            .suites
            .iter()
            .filter(|s| !s.pass)
            .map(|s| s.name.as_str())
            .collect();
        let suffix = if run.suites.len() > 1 && !failed.is_empty() {
            format!(" (failed: {})", failed.join(", "))
        } else {
            String::new()
        };
        println!("test: {verdict} — {}{suffix}", counts.join(", "));
        for t in run
            .tests
            .iter()
            .filter(|t| t.result == sj_core::run::Outcome::Failed)
        {
            if t.ids.is_empty() {
                println!("fail    {}", t.path);
            } else {
                println!("fail    {}  {}", t.path, t.ids.join(", "));
            }
            if let Some(ex) = &t.excerpt {
                for l in &ex.lines {
                    println!("        {l}");
                }
                if ex.omitted > 0 {
                    println!("        … {} more", plural(ex.omitted, "line"));
                }
            }
        }
        for s in run.suites.iter().filter(|s| !s.build_errors.is_empty()) {
            println!("build failed ({})", s.name);
            for l in &s.build_errors {
                println!("        {l}");
            }
        }
    }
    Ok(if run.pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn ledger_cmd(id: Option<String>, root: PathBuf, json: bool) -> Result<ExitCode> {
    let loaded = sj_core::ledger::load(&root)?;
    if loaded.skipped > 0 {
        eprintln!(
            "warn    skipped {} unreadable ledger {}",
            loaded.skipped,
            if loaded.skipped == 1 { "line" } else { "lines" }
        );
    }
    let mut report = sj_core::ledger::report(&loaded.records);
    if let Some(id) = &id {
        report.retain(|s| &s.id == id);
    }
    if json {
        let v = serde_json::json!({ "scenarios": report, "skipped": loaded.skipped });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else if let (Some(id), true) = (&id, report.is_empty()) {
        println!("ledger: no records for {id}");
    } else {
        let proven = report.iter().filter(|s| s.proven).count();
        println!(
            "ledger: {}, {proven} proven red-before-green",
            plural(report.len(), "scenario")
        );
        for s in &report {
            let tag = if s.proven { "proven" } else { "open" };
            println!("{tag:<8}{}  latest {}", s.id, s.latest.as_str());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn spec_paths(args: &SpecArgs) -> Result<(PathBuf, PathBuf)> {
    let spec_root = spec_root_for(&args.root, args.spec_root.clone())?;
    Ok((args.root.clone(), args.root.join(spec_root)))
}

fn location(p: &sj_core::authoring::validate::Problem, root: &Path) -> String {
    if p.line == 0 {
        format!("{} ({})", p.message, rel(&p.file, root))
    } else {
        format!("{} ({}:{})", p.message, rel(&p.file, root), p.line)
    }
}

fn spec_cmd(command: SpecCommand) -> Result<ExitCode> {
    use sj_core::authoring::{self, archive, check, validate};
    match command {
        SpecCommand::New { change, args } => {
            let (root, spec_root) = spec_paths(&args)?;
            let dir = authoring::new_change(&spec_root, &change)?;
            println!("created {}", rel(&dir, &root));
            for (prefix, n) in sj_core::ids::highest_numbers(&spec_root)? {
                println!("next {prefix}-{:03}", n + 1);
            }
            Ok(ExitCode::SUCCESS)
        }
        SpecCommand::List { args, json } => {
            let (_, spec_root) = spec_paths(&args)?;
            let rows = authoring::list(&spec_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if rows.is_empty() {
                println!("no open changes");
            } else {
                let width = rows.iter().map(|r| r.name.len()).max().unwrap_or(0);
                for r in &rows {
                    println!(
                        "{:<width$}  {}/{} tasks  {}",
                        r.name,
                        r.tasks_done,
                        r.tasks_total,
                        plural(r.scenarios, "scenario")
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        SpecCommand::Validate {
            change,
            args,
            strict,
            json,
        } => {
            let (root, spec_root) = spec_paths(&args)?;
            let r = validate::validate(&spec_root, change.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                if r.errors.is_empty() && r.warnings.is_empty() {
                    println!("validate: ok");
                } else {
                    println!(
                        "validate: {}, {}",
                        plural(r.errors.len(), "error"),
                        plural(r.warnings.len(), "warning")
                    );
                }
                for p in &r.errors {
                    println!("fail    {}", location(p, &root));
                }
                for p in &r.warnings {
                    println!("warn    {}", location(p, &root));
                }
            }
            let failed = !r.errors.is_empty() || (strict && !r.warnings.is_empty());
            Ok(if failed {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        SpecCommand::Check { change, args, json } => {
            let (_, spec_root) = spec_paths(&args)?;
            let r = check::check(&spec_root, &change)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                if r.is_clean() {
                    println!("check: ok");
                } else {
                    println!("check: {}", plural(r.problems(), "problem"));
                }
                for id in &r.uncovered {
                    println!("uncovered {id}");
                }
                for u in &r.unknown {
                    println!(
                        "unknown {} ({}:{})",
                        u.id,
                        rel(&u.file, Path::new("")),
                        u.line
                    );
                }
                for c in &r.clarify {
                    println!(
                        "clarify {} ({}:{})",
                        c.question,
                        rel(&c.file, Path::new("")),
                        c.line
                    );
                }
            }
            Ok(if r.is_clean() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        SpecCommand::Archive {
            change,
            args,
            force,
        } => {
            let (root, spec_root) = spec_paths(&args)?;
            let opts = archive::Options {
                force,
                date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            };
            match archive::archive(&spec_root, &change, &opts)? {
                archive::Outcome::Refused(reasons) => {
                    println!("archive: refused");
                    for p in &reasons {
                        println!("fail    {}", location(p, &root));
                    }
                    Ok(ExitCode::from(1))
                }
                archive::Outcome::Archived { specs, dest } => {
                    for s in &specs {
                        let file = rel(&s.file, &root);
                        if s.created {
                            println!("created {file} (+{})", s.added);
                        } else {
                            println!(
                                "updated {file} (+{} ~{} -{} →{})",
                                s.added, s.modified, s.removed, s.renamed
                            );
                        }
                    }
                    println!("archived {change} → {}", rel(&dest, &root));
                    Ok(ExitCode::SUCCESS)
                }
            }
        }
    }
}

fn status_cmd(change: Option<String>, args: &SpecArgs, json: bool) -> Result<ExitCode> {
    use sj_core::agent_loop::status;
    let (root, spec_root) = spec_paths(args)?;
    let rows = status::status(&root, &spec_root, change.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else if rows.is_empty() {
        println!("no open changes");
    } else {
        for r in &rows {
            for l in status::render(r) {
                println!("{l}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn next_cmd(
    change: Option<String>,
    args: &SpecArgs,
    parallel: bool,
    base: Option<String>,
    json: bool,
) -> Result<ExitCode> {
    use sj_core::agent_loop::next;
    let (root, spec_root) = spec_paths(args)?;
    if parallel {
        let Some(name) = next::parallel_change(&spec_root, change.as_deref())? else {
            println!("next: no open tasks");
            return Ok(ExitCode::SUCCESS);
        };
        let batches = next::parallel(&root, &spec_root, &name)?;
        if json {
            let v = serde_json::json!({ "change": name, "batches": batches });
            println!("{}", serde_json::to_string_pretty(&v)?);
        } else if batches.is_empty() {
            println!("next: no parallel tasks");
        } else {
            for (i, b) in batches.iter().enumerate() {
                println!("batch {}", i + 1);
                for t in b {
                    println!("  {t}");
                }
            }
        }
        return Ok(ExitCode::SUCCESS);
    }
    match next::next(&root, &spec_root, change.as_deref(), base.as_deref())? {
        None if json => println!("null"),
        None => println!("next: no open tasks"),
        Some(n) if json => println!("{}", serde_json::to_string_pretty(&n)?),
        Some(n) => print!("{}", next::render(&n)),
    }
    Ok(ExitCode::SUCCESS)
}

fn verify_cmd(
    change: Option<String>,
    args: &SpecArgs,
    base: Option<String>,
    json: bool,
) -> Result<ExitCode> {
    use sj_core::agent_loop::verify;
    let spec_root = spec_root_for(&args.root, args.spec_root.clone())?;
    let r = verify::verify(&verify::Options {
        root: args.root.clone(),
        spec_root,
        change,
        base,
    })?;
    for w in &r.warnings {
        eprintln!("warn    {w}");
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
    } else if r.ok {
        println!("verify: ok");
    } else {
        println!("verify: {}", plural(r.problems.len(), "problem"));
        for p in &r.problems {
            println!("{}", p.message);
        }
    }
    Ok(if r.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn task_done_cmd(change: &str, task: &str, args: &SpecArgs) -> Result<ExitCode> {
    use sj_core::agent_loop::task::{self, Outcome};
    let config = sj_core::config::load(&args.root)?;
    let (root, spec_root) = spec_paths(args)?;
    match task::done(&spec_root, &root, change, task, config.mode)? {
        Outcome::AlreadyDone { text } => println!("task: already done {text}"),
        Outcome::Done { text, warnings } => {
            println!("task: done {text}");
            for u in &warnings {
                println!("warn    unproven {} ({})", u.id, u.why);
            }
        }
        Outcome::Refused { unproven } => {
            println!("task: refused — {} unproven", unproven.len());
            for u in &unproven {
                println!("unproven {} ({})", u.id, u.why);
            }
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn ids_cmd(write: bool, root: &Path, spec_root: Option<PathBuf>, json: bool) -> Result<ExitCode> {
    let config = sj_core::config::load(root)?;
    let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
    let plan = sj_core::ids::plan(&root.join(&spec_root), &config.id_prefixes)?;
    let files = if write {
        sj_core::ids::write(&plan)?
    } else {
        0
    };
    if json {
        let assignments: Vec<serde_json::Value> = plan
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "name": a.name,
                    "file": rel(&a.file, root),
                    "line": a.line,
                })
            })
            .collect();
        let v = serde_json::json!({
            "missing": plan.len(),
            "written": write && !plan.is_empty(),
            "assignments": assignments,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        if plan.len() == 1 {
            println!("1 scenario without an ID");
        } else {
            println!("{} scenarios without IDs", plan.len());
        }
        for a in &plan {
            println!("{} {} ({}:{})", a.id, a.name, rel(&a.file, root), a.line);
        }
        if write && !plan.is_empty() {
            println!(
                "wrote {} to {}",
                if plan.len() == 1 {
                    "1 ID".to_string()
                } else {
                    format!("{} IDs", plan.len())
                },
                plural(files, "file")
            );
        }
    }
    Ok(if write || plan.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn print_compact(r: &sj_core::trace::Report, root: &Path) {
    println!(
        "trace: {} mapped, {}, {}, {} without ID",
        r.mapped.len(),
        plural(r.gaps.len(), "gap"),
        plural(r.orphans.len(), "orphan"),
        r.unidentified.len()
    );
    for w in &r.warnings {
        println!("warn    {w}");
    }
    for g in &r.gaps {
        println!(
            "gap     {} {}  ({}:{})",
            g.id.as_deref().unwrap_or_default(),
            g.name,
            rel(&g.file, root),
            g.line
        );
    }
    for o in &r.orphans {
        println!(
            "orphan  {} {}  ({}:{})",
            o.id,
            o.test.name,
            rel(&o.test.file, root),
            o.test.line
        );
    }
    for u in &r.unidentified {
        println!("no-id   {}  ({}:{})", u.name, rel(&u.file, root), u.line);
    }
}
