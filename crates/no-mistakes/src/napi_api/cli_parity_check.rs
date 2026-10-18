pub(crate) fn check_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<ProjectOptions>(options)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let results = if options.include_sql_verdicts {
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
    .map_err(to_napi_error)?;
    to_pretty_json(&crate::check_runner::json_value(&results))
}
