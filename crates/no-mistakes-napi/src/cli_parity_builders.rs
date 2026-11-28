pub(crate) fn build_plan_args(options: TestsPlanOptions) -> AnyhowResult<no_mistakes::tests::PlanArgs> {
    validate_head_requires_base(options.base.as_deref(), options.head.as_deref())?;
    let framework = options
        .framework
        .as_deref()
        .map(parse_test_framework)
        .transpose()?;
    if options.direct_test_owner && framework.is_none() {
        bail!(
            "directTestOwner requires framework (for example, framework: \"vitest\") because direct ownership requires framework-specific test ownership"
        );
    }
    if options.direct_test_owner && !options.entrypoints.is_empty() {
        bail!(
            "directTestOwner conflicts with entrypoints: direct-owner selection only follows changed files and one reverse canonical graph edge; use testsImpact for explicit entrypoint traversal"
        );
    }
    if options.direct_test_owner
        && (options.limit_percent.is_some()
            || options.limit_files.is_some()
            || options.global_config_fallback.is_some())
    {
        bail!(
            "directTestOwner conflicts with limitPercent, limitFiles, and globalConfigFallback; remove those policy overrides because direct ownership bypasses configured plan policy"
        );
    }

    if options.direct_test_owner && options.changes.is_some() {
        bail!("directTestOwner conflicts with changes");
    }
    let (entrypoints, entrypoint_symbols) = entrypoint_parts(options.entrypoints);

    Ok(no_mistakes::tests::PlanArgs {
        framework,
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        base: options.base,
        head: options.head,
        from_git_diff: options.from_git_diff,
        changed_file: strings_to_paths(options.changed_files),
        changed_files: options.changed_files_file.map(PathBuf::from),
        changes: options.changes.map(PathBuf::from),
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
        direct_test_owner: options.direct_test_owner,
        format: Some(no_mistakes::tests::PlanFormat::Json),
        json: true,
        include_comment: options.include_comment,
        include_glob: options.include_glob,
    })
}

pub(crate) fn parse_test_framework(value: &str) -> AnyhowResult<no_mistakes::tests::TestFramework> {
    match value {
        "dotnet" => Ok(no_mistakes::tests::TestFramework::Dotnet),
        "playwright" => Ok(no_mistakes::tests::TestFramework::Playwright),
        "vitest" => Ok(no_mistakes::tests::TestFramework::Vitest),
        "swift" => Ok(no_mistakes::tests::TestFramework::Swift),
        "python" => Ok(no_mistakes::tests::TestFramework::Python),
        "go" => Ok(no_mistakes::tests::TestFramework::Go),
        "cargo" => Ok(no_mistakes::tests::TestFramework::Cargo),
        "rails" => Ok(no_mistakes::tests::TestFramework::Rails),
        "php" => Ok(no_mistakes::tests::TestFramework::Php),
        "java" => Ok(no_mistakes::tests::TestFramework::Java),
        "kotlin" => Ok(no_mistakes::tests::TestFramework::Kotlin),
        "elixir" => Ok(no_mistakes::tests::TestFramework::Elixir),
        "dart" => Ok(no_mistakes::tests::TestFramework::Dart),
        "jest" => Ok(no_mistakes::tests::TestFramework::Jest),
        _ => bail!("unknown test framework: {value}"),
    }
}

pub(crate) fn build_why_args(options: TestsWhyOptions) -> AnyhowResult<no_mistakes::tests::WhyArgs> {
    let test = options.test.context("test is required")?;
    Ok(no_mistakes::tests::WhyArgs {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        test: PathBuf::from(test),
        changed: options.changed.map(PathBuf::from),
        plan: options.plan.map(PathBuf::from),
        format: no_mistakes::tests::WhyFormat::Json,
    })
}

pub(crate) fn build_impact_args(
    options: TestsImpactOptions,
) -> AnyhowResult<no_mistakes::tests::ImpactArgs> {
    if options.entrypoints.is_empty() {
        bail!("entrypoints is required and must not be empty");
    }
    let (entrypoints, entrypoint_symbols) = entrypoint_parts(options.entrypoints);

    Ok(no_mistakes::tests::ImpactArgs {
        entrypoints,
        entrypoint_symbols,
        include_symbols: options.include_symbols,
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(no_mistakes::tests::ImpactFormat::Json),
        json: true,
    })
}

fn strings_to_paths(values: Vec<String>) -> Vec<PathBuf> {
    values.into_iter().map(PathBuf::from).collect()
}

fn validate_head_requires_base(base: Option<&str>, head: Option<&str>) -> AnyhowResult<()> {
    anyhow::ensure!(head.is_none() || base.is_some(), "head requires base");
    Ok(())
}

pub(crate) fn build_impacted_checks_args(
    options: ImpactedChecksOptions,
) -> AnyhowResult<no_mistakes::impacted_checks::ImpactedChecksArgs> {
    validate_head_requires_base(options.base.as_deref(), options.head.as_deref())?;
    let diff_sources = [
        options.diff.is_some(),
        options.diff_file.is_some(),
        options.diff_stdin,
        options.diff_command.is_some(),
    ];
    anyhow::ensure!(
        diff_sources.into_iter().filter(|present| *present).count() <= 1,
        "diff, diffFile, diffStdin, and diffCommand are mutually exclusive"
    );
    Ok(no_mistakes::impacted_checks::ImpactedChecksArgs {
        files: Vec::new(),
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
        diff: options.diff_file.map(PathBuf::from),
        diff_stdin: options.diff_stdin,
        diff_command: options.diff_command,
        diff_content: options.diff,
        format: None,
        json: false,
        generic_only: options.generic_only,
        // N-API timings are collected into the structured response by the
        // binding; they must never print CLI progress to the Node process.
        timings: false,
        diagnose_empty: false,
    })
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
