pub(crate) fn build_plan_args(
    options: TestsPlanOptions,
) -> AnyhowResult<crate::tests::PlanArgs> {
    let framework = options
        .framework
        .as_deref()
        .map(parse_test_framework)
        .transpose()?;

    let (entrypoints, entrypoint_symbols) = entrypoint_parts(options.entrypoints);

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
        entrypoints,
        entrypoint_symbols,
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

pub(crate) fn parse_test_framework(value: &str) -> AnyhowResult<crate::tests::TestFramework> {
    match value {
        "dotnet" => Ok(crate::tests::TestFramework::Dotnet),
        "playwright" => Ok(crate::tests::TestFramework::Playwright),
        "vitest" => Ok(crate::tests::TestFramework::Vitest),
        "swift" => Ok(crate::tests::TestFramework::Swift),
        _ => bail!("unknown test framework: {value}"),
    }
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
    let (entrypoints, entrypoint_symbols) = entrypoint_parts(options.entrypoints);

    Ok(crate::tests::ImpactArgs {
        entrypoints,
        entrypoint_symbols,
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

pub(crate) fn build_impacted_checks_args(
    options: ImpactedChecksOptions,
) -> crate::impacted_checks::ImpactedChecksArgs {
    crate::impacted_checks::ImpactedChecksArgs {
        files: Vec::new(),
        root: options.root.map(PathBuf::from).unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        base: options.base,
        head: options.head,
        changed_file: strings_to_paths(options.changed_files),
        changed_files: options.changed_files_file.map(PathBuf::from),
        // Match `testsPlan`: the N-API `diff` option is inline diff content.
        diff: None,
        diff_content: options.diff,
        format: None,
        json: false,
    }
}

fn entrypoint_parts(
    values: Vec<super::options::EntrypointOption>,
) -> (Vec<String>, Vec<Option<String>>) {
    let mut entrypoints = Vec::new();
    let mut symbols = Vec::new();
    values.into_iter().for_each(|entrypoint| match entrypoint {
        super::options::EntrypointOption::Path(path) => {
            entrypoints.push(path);
            symbols.push(None);
        }
        super::options::EntrypointOption::Symbol(option) => {
            entrypoints.push(option.file);
            symbols.push(Some(option.symbol.unwrap_or_default()));
        }
    });
    (entrypoints, symbols)
}
