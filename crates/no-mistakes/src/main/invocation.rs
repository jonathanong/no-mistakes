use super::{run_command, Cli};
use anyhow::Result;
use no_mistakes::cli::init_rayon_threads;
use no_mistakes::diagnostics::{InvocationGuard, ProgressGuard, TimingKind};
use no_mistakes::invocation::InvocationGuard as ExecutionGuard;
use std::{process::ExitCode, sync::Arc};

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
    let mut result = (|| {
        let prepare = || ExecutionGuard::acquire_for_cli(cli.invocation.options());
        execution_guard = Some(match &observer {
            Some(observer) => observer.trace("invocation.prepare", TimingKind::Serial, prepare),
            None => prepare(),
        }?);
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
        if observer.progress_enabled() {
            if let Err(error) = &result {
                observer.emit_progress_output(format!("error: {error:#}\n"));
                result = Ok(ExitCode::from(
                    no_mistakes::invocation::timeout_exit_code(error).unwrap_or(2),
                ));
            }
        }
    }
    drop(execution_guard);
    result
}
