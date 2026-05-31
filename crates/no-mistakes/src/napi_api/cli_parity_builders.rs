pub(crate) fn build_plan_args(
    options: TestsPlanOptions,
) -> AnyhowResult<crate::tests::PlanArgs> {
    let framework = match options.framework.as_deref() {
        Some("playwright") => Some(crate::tests::TestFramework::Playwright),
        Some("vitest") => Some(crate::tests::TestFramework::Vitest),
        Some(value) => bail!("unknown test framework: {value}"),
        None => None,
    };

    Ok(crate::tests::PlanArgs {
        framework,
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        base: options.base,
        head: options.head,
        changed_file: strings_to_paths(options.changed_files),
        changed_files: options.changed_files_file.map(PathBuf::from),
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: options
            .entrypoints
            .into_iter()
            .map(|entrypoint| entrypoint.into_cli_string())
            .collect(),
        include_symbols: options.include_symbols,
        diff_content: options.diff,
        environment: options
            .environment
            .unwrap_or_else(|| "pre-push".to_string()),
        limit_percent: options.limit_percent,
        limit_files: options.limit_files,
        global_config_fallback: options.global_config_fallback,
        format: Some(crate::tests::PlanFormat::Json),
        json: true,
    })
}

pub(crate) fn build_why_args(
    options: TestsWhyOptions,
) -> AnyhowResult<crate::tests::WhyArgs> {
    let test = options.test.context("test is required")?;
    Ok(crate::tests::WhyArgs {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        test: PathBuf::from(test),
        changed: options.changed.map(PathBuf::from),
        plan: options.plan.map(PathBuf::from),
        format: crate::tests::WhyFormat::Json,
    })
}

pub(crate) fn build_impact_args(
    options: TestsImpactOptions,
) -> AnyhowResult<crate::tests::ImpactArgs> {
    if options.entrypoints.is_empty() {
        bail!("entrypoints is required and must not be empty");
    }
    Ok(crate::tests::ImpactArgs {
        entrypoints: options
            .entrypoints
            .into_iter()
            .map(|entrypoint| entrypoint.into_cli_string())
            .collect(),
        include_symbols: options.include_symbols,
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(crate::tests::PlanFormat::Json),
        json: true,
    })
}

fn strings_to_paths(values: Vec<String>) -> Vec<PathBuf> {
    values.into_iter().map(PathBuf::from).collect()
}
