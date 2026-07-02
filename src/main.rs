mod analyze;
mod config;
mod hooks;
mod rules;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use walkdir::WalkDir;

use crate::analyze::{Diagnostic, analyze_text};
use crate::config::Config;
use crate::hooks::{Agent, HookAgent, HookScope};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check Markdown and text files for AI slop patterns.
    Check(CheckArgs),
    /// Create a starter project config.
    Init {
        /// Path to write. Defaults to .slop-lint.toml.
        #[arg(long, default_value = ".slop-lint.toml")]
        path: PathBuf,
    },
    /// Print a PostToolUse hook snippet for agent workflows.
    HookSample,
    /// Install slop-lint into supported agent hook configurations.
    InstallHooks {
        /// Agent to install for. Use auto to detect supported agents.
        #[arg(long, value_enum, default_value_t = HookAgent::Auto)]
        agent: HookAgent,

        /// Install in the current project or user-level config when supported.
        #[arg(long, value_enum, default_value_t = HookScope::Project)]
        scope: HookScope,

        /// Show what would be written without changing files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Show supported agent hook installation status.
    HookStatus {
        /// Agent to inspect. Use auto to inspect detected agents.
        #[arg(long, value_enum, default_value_t = HookAgent::Auto)]
        agent: HookAgent,

        /// Inspect project or user-level config when supported.
        #[arg(long, value_enum, default_value_t = HookScope::Project)]
        scope: HookScope,
    },
    /// Remove slop-lint from supported agent hook configurations.
    UninstallHooks {
        /// Agent to uninstall from. Use auto to uninstall from detected agents.
        #[arg(long, value_enum, default_value_t = HookAgent::Auto)]
        agent: HookAgent,

        /// Remove from project or user-level config when supported.
        #[arg(long, value_enum, default_value_t = HookScope::Project)]
        scope: HookScope,
    },
    /// Agent hook entrypoint. Reads hook JSON from stdin and emits feedback JSON.
    Hook {
        /// Agent protocol shape to target.
        #[arg(long, value_enum, default_value_t = Agent::Generic)]
        agent: Agent,
    },
}

#[derive(Debug, Parser)]
struct CheckArgs {
    /// Files or directories to check.
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Additional config file to merge last.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Exit with status 1 when diagnostics are present.
    #[arg(long)]
    fail_on_warning: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Serialize)]
struct CheckReport {
    diagnostics: Vec<Diagnostic>,
    summary: CheckSummary,
}

#[derive(Debug, Serialize)]
struct CheckSummary {
    files_checked: usize,
    diagnostic_count: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or_else(default_command) {
        Command::Check(args) => run_check(args),
        Command::Init { path } => run_init(&path),
        Command::HookSample => {
            println!("{}", hook_sample());
            Ok(())
        }
        Command::InstallHooks {
            agent,
            scope,
            dry_run,
        } => hooks::install_hooks(agent, scope, dry_run),
        Command::HookStatus { agent, scope } => hooks::hook_status(agent, scope),
        Command::UninstallHooks { agent, scope } => hooks::uninstall_hooks(agent, scope),
        Command::Hook { agent } => hooks::run_hook(agent),
    }
}

fn default_command() -> Command {
    Command::Check(CheckArgs {
        paths: vec![PathBuf::from(".")],
        format: OutputFormat::Text,
        config: None,
        fail_on_warning: false,
    })
}

fn run_check(args: CheckArgs) -> Result<()> {
    let mut config = Config::load_hierarchy(std::env::current_dir()?)?;
    if let Some(path) = args.config.as_ref() {
        config.merge_file(path)?;
    }

    let rules = config.rules()?;
    let mut diagnostics = Vec::new();
    let mut files_checked = 0;

    for path in collect_files(&args.paths, &config)? {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        files_checked += 1;
        diagnostics.extend(analyze_text(&path, &text, &config, &rules));
    }

    let report = CheckReport {
        summary: CheckSummary {
            files_checked,
            diagnostic_count: diagnostics.len(),
        },
        diagnostics,
    };

    match args.format {
        OutputFormat::Text => print_text_report(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }

    if args.fail_on_warning && report.summary.diagnostic_count > 0 {
        bail!("slop-lint found diagnostics");
    }
    Ok(())
}

fn collect_files(paths: &[PathBuf], config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if should_check(path, config) {
                files.push(path.to_path_buf());
            }
            continue;
        }
        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_entry(|entry| !is_ignored_dir(entry.path()))
            {
                let entry = entry?;
                if entry.path().is_file() && should_check(entry.path(), config) {
                    files.push(entry.path().to_path_buf());
                }
            }
            continue;
        }
        bail!("path does not exist: {}", path.display());
    }
    files.sort();
    Ok(files)
}

fn should_check(path: &Path, config: &Config) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    config
        .file_extensions
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(ext))
}

fn is_ignored_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(name, ".git" | "target" | "node_modules" | ".next" | "dist")
}

fn print_text_report(report: &CheckReport) {
    for diagnostic in &report.diagnostics {
        println!(
            "{}:{}:{} {} {} [{}]",
            diagnostic.path.display(),
            diagnostic.line,
            diagnostic.column,
            diagnostic.level,
            diagnostic.message,
            diagnostic.rule_id
        );
        println!("  action: {}", diagnostic.action);
        if let Some(suggestion) = diagnostic.suggestion.as_ref() {
            println!("  suggestion: {suggestion}");
        }
    }

    if report.summary.diagnostic_count == 0 {
        println!(
            "slop-lint: no diagnostics in {} file(s)",
            report.summary.files_checked
        );
    } else {
        println!(
            "slop-lint: {} diagnostic(s) in {} file(s)",
            report.summary.diagnostic_count, report.summary.files_checked
        );
    }
}

fn run_init(path: &Path) -> Result<()> {
    if path.exists() {
        bail!("config already exists: {}", path.display());
    }
    fs::write(path, starter_config())
        .with_context(|| format!("failed to write {}", path.display()))?;
    println!("created {}", path.display());
    Ok(())
}

fn starter_config() -> &'static str {
    r#"# slop-lint project configuration
sensitivity = "high"
mode = "interactive"
uncertain_action = "ask_user"
file_extensions = ["md", "mdx", "txt", "rst", "adoc"]
extends = ["default", "zh"]
ruleset_paths = []

[mute]
rules = []

# Example: mute a rule after the Agent has confirmed the project wants this style.
# rules = ["slop.overconfident-transition"]

# Add project-specific rules with [[rules]], or load external rule files through ruleset_paths.
"#
}

fn hook_sample() -> &'static str {
    r#"Quick hook command:

slop-lint install-hooks

Manual hook command, if you need it:

slop-lint hook --agent generic

Recommended Agent behavior:
- If hook feedback says revise, edit the file directly.
- If hook feedback says ask_user, ask whether to revise or mute the rule in .slop-lint.toml.
- If the user chooses mute, add the rule id to [mute].rules with a short comment.
"#
}
