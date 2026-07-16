//! `no-mistakes impacted-checks <file...>` — the minimal local validation
//! commands to run for a set of changed files.
//!
//! Test commands are derived from the same configured-plan engine as
//! `tests plan`, with one shared file discovery and dependency graph across
//! frameworks so emitted runner invocations keep exact parity without repeated
//! repository analysis.
//! Generic checks (lint, typecheck, …) come from the `checks:` config block,
//! keyed by file globs.

use crate::cli::{resolve_format, Format};
use crate::tests::Warning;
use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

mod frameworks;
mod generate;
pub(crate) mod timing;
pub use generate::generate_impacted_checks;
pub(crate) use generate::generate_impacted_checks_with_timing;

#[derive(Args)]
pub struct ImpactedChecksArgs {
    /// Changed file path(s), relative to `--root` (or absolute).
    #[arg(value_name = "FILE")]
    pub(crate) files: Vec<PathBuf>,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub(crate) root: PathBuf,
    /// Path to config file.
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,
    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    pub(crate) tsconfig: Option<PathBuf>,
    /// Git base commit/branch to diff against.
    #[arg(long)]
    pub(crate) base: Option<String>,
    /// Git head commit/branch (defaults to HEAD).
    #[arg(long, requires = "base")]
    pub(crate) head: Option<String>,
    /// Specific changed file path. Can be repeated.
    #[arg(long = "changed-file")]
    pub(crate) changed_file: Vec<PathBuf>,
    /// Path to a file listing changed files (one per line).
    #[arg(long = "changed-files")]
    pub(crate) changed_files: Option<PathBuf>,
    /// Path to a unified diff file.
    #[arg(long)]
    pub(crate) diff: Option<PathBuf>,
    /// Inline unified diff content (programmatic/N-API only).
    #[arg(skip)]
    pub(crate) diff_content: Option<String>,
    /// Output format: json, md, yml, paths, human.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<Format>,
    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub(crate) json: bool,
    /// Legacy programmatic timing switch. CLI timing flags are root-global.
    #[arg(skip)]
    pub(crate) timings: bool,
}

/// The elapsed time for one `impacted-checks` analysis phase.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ImpactedChecksTiming {
    /// Stable phase identifier.
    pub phase: String,
    /// Fractional milliseconds elapsed in the phase.
    pub duration_ms: f64,
}

/// A single validation command to run.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckCommand {
    /// Runner or configured check name (e.g. `vitest`, `eslint`).
    pub name: String,
    /// What kind of check this is.
    pub kind: CheckKind,
    /// Full command argv.
    pub command: Vec<String>,
    /// Changed files that triggered this command.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// The category of a [`CheckCommand`].
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    /// A test command derived from the test-plan engine.
    Test,
    /// A configured generic check (lint, typecheck, …).
    Generic,
}

/// The result of an impacted-checks query.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImpactedChecksReport {
    /// The changed files considered (repo-relative, slash-normalized).
    pub changed_files: Vec<String>,
    /// Validation commands to run, deduped and sorted.
    pub checks: Vec<CheckCommand>,
    /// Warnings propagated from the test-plan engine.
    pub warnings: Vec<Warning>,
    /// True when a full-suite fallback was triggered (e.g. global config change).
    pub fallback_triggered: bool,
}

pub fn run(args: ImpactedChecksArgs) -> Result<ExitCode> {
    let format = resolve_format(args.json, args.format, std::io::stdout().is_terminal());
    let mut timings = timing::TimingTracker::new(args.timings, false);
    let report = match generate_impacted_checks_with_timing(&args, &mut timings) {
        Ok((report, _)) => report,
        Err(error) => {
            timings.fail_total();
            return Err(error);
        }
    };
    timings.finish_total();
    let stdout = std::io::stdout();
    publish_rendered_with_deadline_check(
        &report,
        format,
        &mut stdout.lock(),
        crate::invocation::check_timeout,
    )?;
    Ok(ExitCode::SUCCESS)
}

const _: fn(ImpactedChecksArgs) -> Result<ExitCode> = run;

fn render(report: &ImpactedChecksReport, format: Format) -> Result<String> {
    Ok(match format {
        Format::Json => format!("{}\n", serde_json::to_string_pretty(report)?),
        Format::Yml => serde_yaml::to_string(report)?,
        Format::Paths => report
            .checks
            .iter()
            .map(|check| format!("{}\n", shell_join(&check.command)))
            .collect(),
        Format::Md => render_text(report, "- "),
        Format::Human => render_text(report, ""),
    })
}

fn publish_rendered_with_deadline_check(
    report: &ImpactedChecksReport,
    format: Format,
    output: &mut impl Write,
    mut check_deadline: impl FnMut() -> Result<()>,
) -> Result<()> {
    check_deadline()?;
    let rendered = render(report, format)?;
    check_deadline()?;
    output.write_all(rendered.as_bytes())?;
    Ok(())
}

/// Join command tokens with POSIX shell quoting so the `paths` output is safe to
/// `eval` even when a file path contains spaces or shell metacharacters.
fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|token| shell_quote(token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(token: &str) -> String {
    let safe = !token.is_empty()
        && token.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '_' | '-' | '.' | '/' | ':' | '=' | '@' | ',' | '+')
        });
    if safe {
        token.to_string()
    } else {
        format!("'{}'", token.replace('\'', "'\\''"))
    }
}

fn render_text(report: &ImpactedChecksReport, bullet: &str) -> String {
    let mut out = String::new();
    if report.checks.is_empty() {
        out.push_str("No checks for the changed files.\n");
    }
    for check in &report.checks {
        out.push_str(&format!("{bullet}{}\n", check.command.join(" ")));
    }
    for warning in &report.warnings {
        out.push_str(&format!("warning: {}: {}\n", warning.file, warning.message));
    }
    if report.fallback_triggered {
        out.push_str("note: full-suite fallback triggered\n");
    }
    out
}

#[cfg(test)]
mod tests;
