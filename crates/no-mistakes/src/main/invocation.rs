use super::{run_command, Cli, Command};
use anyhow::Result;
use no_mistakes::cli::init_rayon_threads;
use no_mistakes::diagnostics::{InvocationGuard, ProgressGuard, TimingKind};
use no_mistakes::invocation::InvocationGuard as ExecutionGuard;
use std::{process::ExitCode, sync::Arc};

/// Whether argv asks for JSON output (`--format json`, `--format=json`, `--json`).
pub(super) fn wants_json(args: impl IntoIterator<Item = String>) -> bool {
    let args: Vec<String> = args.into_iter().collect();
    args.iter()
        .any(|arg| arg == "--json" || arg == "--format=json")
        || args
            .windows(2)
            .any(|pair| pair[0] == "--format" && pair[1] == "json")
}

/// Print the error, plus a JSON error object when JSON output was requested, and pick the exit code.
pub(super) fn report_error(error: &anyhow::Error) -> ExitCode {
    eprintln!("error: {error:#}");
    if wants_json(std::env::args()) {
        if let Some(report) = no_mistakes::invocation::error_report(error) {
            println!("{report}");
        }
    }
    ExitCode::from(no_mistakes::invocation::exit_code(error))
}

pub(super) fn run(cli: Cli) -> Result<ExitCode> {
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
        let prepare = || ExecutionGuard::acquire_for_cli(cli.invocation.options());
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
