pub fn run(args: TraverseArgs, direction: Direction) -> Result<()> {
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let stdout = io::stdout();
    let stdout_is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();

    run_with_cwd_and_writer(args, direction, cwd_early, stdout_is_terminal, &mut out)
}

fn run_with_cwd_and_writer(
    args: TraverseArgs,
    direction: Direction,
    cwd_early: PathBuf,
    stdout_is_terminal: bool,
    out: &mut dyn Write,
) -> Result<()> {
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let root = resolve_root(&args, &cwd_early);
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let tsconfig = resolve_tsconfig(&args, &root)?;
    let entrypoints = resolve_entrypoints(&args.files, &root, &cwd_early);

    let root_strs: Vec<String> = args.files.iter().map(|f| f.display().to_string()).collect();

    timings.mark("search");

    // Check for #symbol used in Deps direction (unsupported).
    validate_direction(&direction, &entrypoints)?;

    let allowed = relationship_filter(&args.relationships);
    let build_plan = graph::GraphBuildPlan::from_allowed(allowed.as_ref());
    let graph_files = graph::GraphFiles::discover(&root);
    let ctx = TraversalCtx {
        root: &root,
        tsconfig: &tsconfig,
        graph_files: &graph_files,
        build_plan,
        allowed: allowed.as_ref(),
    };
    let roots: Vec<NodeId> = entrypoints
        .iter()
        .map(|e| NodeId::File(e.file.clone()))
        .collect();
    let import_only = relationships_are_import_only(&args.relationships);

    timings.mark("ingest");

    let entries = get_entries(
        direction,
        &roots,
        &entrypoints,
        args.depth,
        import_only,
        &ctx,
    );

    timings.mark("parse");

    // Build combined filter from --filter and --test globs.
    let mut all_filters = args.filters.clone();
    for framework in &args.tests {
        all_filters.extend(test_globs(framework));
    }
    let filter = graph::build_filter(&all_filters)?;
    let entries = graph::apply_filter(entries, filter.as_ref(), &root);

    timings.mark("analysis");

    // Resolve output format.
    let format = resolve_format(args.json, args.format, stdout_is_terminal);

    write_entries(format, &root_strs, &entries, &root, out)?;

    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }

    Ok(())
}
