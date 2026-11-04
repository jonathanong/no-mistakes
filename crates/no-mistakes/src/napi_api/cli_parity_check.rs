pub(crate) fn check_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<ProjectOptions>(options)?;
    let observer = options.timings.then(|| {
        crate::diagnostics::current()
            .unwrap_or_else(|| crate::diagnostics::InvocationObserver::new(false))
    });
    let _guard = observer.as_ref().map(|observer| {
        crate::diagnostics::InvocationGuard::install(std::sync::Arc::clone(observer))
    });
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let results = if options.effort.is_none() {
        if options.include_sql_verdicts {
            crate::check_runner::run_all_with_sql_verdicts(
                root,
                options.config.map(PathBuf::from),
                options.tsconfig.map(PathBuf::from),
                options.include_suppressed,
                options.include_runner_config_deadlines,
                true,
            )
        } else {
            crate::check_runner::run_all_with_evidence(
                root,
                options.config.map(PathBuf::from),
                options.tsconfig.map(PathBuf::from),
                options.include_suppressed,
                options.include_runner_config_deadlines,
            )
        }
    } else {
        crate::check_runner::run_all_with_options(
            root,
            options.config.map(PathBuf::from),
            options.tsconfig.map(PathBuf::from),
            options.include_suppressed,
            options.include_runner_config_deadlines,
            crate::check_runner::AnalysisOptions {
                effort: options.effort,
                include_sql_verdicts: options.include_sql_verdicts,
            },
        )
    }
    .map_err(to_napi_error)?;
    let mut report = crate::check_runner::json_value(&results);
    if let Some(observer) = observer {
        report["timings"] = serde_json::Value::Array(
            observer
                .snapshot()
                .timings
                .into_iter()
                .map(|entry| {
                    serde_json::json!({
                        "label": entry.label,
                        "durationMs": entry.duration.as_secs_f64() * 1000.0,
                        "parallel": entry.kind == crate::diagnostics::TimingKind::Parallel,
                    })
                })
                .collect(),
        );
    }
    to_pretty_json(&report)
}
