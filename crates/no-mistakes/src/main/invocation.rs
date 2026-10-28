use super::{run_command, Cli, Command};
use anyhow::Result;
use no_mistakes::cli::init_rayon_threads;
use no_mistakes::diagnostics::{InvocationGuard, ProgressGuard, TimingKind};
use no_mistakes::invocation::{CliErrorOutput, InvocationGuard as ExecutionGuard};
use std::{process::ExitCode, sync::Arc};

/// Use validated command arguments, never values that only resemble output flags.
pub(super) fn wants_json(matches: &clap::ArgMatches) -> bool {
    let json = matches.try_get_one::<bool>("json").ok().flatten() == Some(&true);
    let format = matches
        .try_get_raw("format")
        .ok()
        .flatten()
        .and_then(|mut values| values.next())
        .is_some_and(|format| format == "json");
    json || format
        || matches
            .subcommand()
            .is_some_and(|(_, child)| wants_json(child))
}

/// Print the error, plus a JSON error object when JSON output was requested, and pick the exit code.
pub(super) fn report_error(error: &anyhow::Error, output: &CliErrorOutput) -> ExitCode {
    let message = format!("error: {error:#}\n");
    eprint!("{message}");
    if let Some(report) = output.report_or_wait(error) {
        println!("{report}");
        output.complete();
    }
    ExitCode::from(no_mistakes::invocation::exit_code(error))
}

#[cfg(all(test, unix))]
pub(super) fn run(cli: Cli) -> Result<ExitCode> {
    run_with_output(cli, CliErrorOutput::new(false))
}

pub(super) fn run_with_output(cli: Cli, output: CliErrorOutput) -> Result<ExitCode> {
    let observer = cli.diagnostics.observer();
    let _diagnostics_guard = observer
        .as_ref()
        .map(|observer| InvocationGuard::install(Arc::clone(observer)));
    let progress_guard = observer
        .as_ref()
        .zip(cli.diagnostics.progress_interval())
        .map(|(observer, interval)| ProgressGuard::install(Arc::clone(observer), interval));
    let mut execution_guard = None;
    let result = (|| {
        let prepare = || {
            ExecutionGuard::acquire_for_cli_with_output(cli.invocation.options(), output.clone())
        };
        // `lock status` must stay answerable while another invocation holds the lock.
        if !matches!(cli.command, Command::Lock(_)) {
            execution_guard = Some(match &observer {
                Some(observer) => observer.trace("invocation.prepare", TimingKind::Serial, prepare),
                None => prepare(),
            }?);
        }
        init_rayon_threads(cli.jobs);
        no_mistakes::invocation::check_timeout()?;
        let dispatch = || no_mistakes::ast::with_request_parse_cache(|| run_command(cli.command));
        let result = match &observer {
            Some(observer) => observer.trace("invocation.dispatch", TimingKind::Serial, dispatch),
            None => dispatch(),
        };
        no_mistakes::invocation::check_timeout()?;
        result
    })();
    drop(progress_guard);
    if let Some(observer) = observer {
        match &result {
            Ok(_) => observer.increment("output.renders", 1),
            Err(_) => observer.increment("output.errors", 1),
        }
        observer.render_progress_stderr();
        observer.render_stderr();
    }
    drop(execution_guard);
    result
}

#[cfg(all(test, unix))]
#[path = "invocation/tests.rs"]
mod tests;
