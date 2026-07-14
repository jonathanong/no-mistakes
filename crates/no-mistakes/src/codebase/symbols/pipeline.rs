pub fn collect_entries(args: &SymbolsArgs) -> Result<(Vec<FileEntry>, Vec<String>)> {
    collect_entries_with_timings(args, None)
}

pub(crate) fn collect_entries_with_prepared_facts(
    args: &SymbolsArgs,
    root: &Path,
    tsconfig: &TsConfig,
    visible_files: &std::collections::HashSet<PathBuf>,
    facts: &crate::codebase::check_facts::CheckFactMap,
    supplemental: &crate::codebase::check_facts::CheckFactMap,
) -> Result<(Vec<FileEntry>, Vec<String>)> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let abs_files = resolve_input_files(&args.files, root, &cwd);
    let kind_filter = build_kind_filter(&args.kinds);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(tsconfig)
        .with_visible(visible_files);
    let entries = abs_files
        .iter()
        .map(|path| {
            let file = facts
                .ts
                .get(path)
                .or_else(|| supplemental.ts.get(path))
                .with_context(|| format!("reading {}", path.display()))?;
            if let Some(error) = &file.parse_error {
                anyhow::bail!("extracting symbols from {}: {error}", path.display());
            }
            let symbols = file
                .symbols
                .clone()
                .context("shared analyzeProject facts are missing symbols")?;
            build_entry_from_symbols(
                path,
                root,
                &resolver,
                symbols,
                args.include,
                kind_filter.as_ref(),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let root_strs = args.files.iter().map(|file| file.display().to_string()).collect();
    Ok((entries, root_strs))
}

fn collect_entries_with_timings(
    args: &SymbolsArgs,
    mut timings: Option<&mut crate::codebase::timing::PhaseTimings>,
) -> Result<(Vec<FileEntry>, Vec<String>)> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = resolve_root(args.root.as_deref(), &cwd);
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        args.tsconfig.as_deref(),
        &root,
        &visible_paths,
    )?;
    let visible_files = visible_paths
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect();
    let abs_files = resolve_input_files(&args.files, &root, &cwd);
    if let Some(timings) = &mut timings {
        timings.mark("search");
    }

    let kind_filter = build_kind_filter(&args.kinds);
    if let Some(timings) = &mut timings {
        timings.mark("ingest");
    }

    let entries: Vec<FileEntry> = abs_files
        .par_iter()
        .map(|abs| {
            build_entry(
                abs,
                &root,
                &tsconfig,
                &visible_files,
                args.include,
                kind_filter.as_ref(),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    if let Some(timings) = &mut timings {
        timings.mark("parse+analysis");
    }

    let root_strs: Vec<String> = args.files.iter().map(|f| f.display().to_string()).collect();
    Ok((entries, root_strs))
}

pub fn run(args: SymbolsArgs) -> Result<()> {
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    if args.mode == SymbolsMode::SignatureImpact {
        let report = impact::collect_report(&args)?;
        timings.mark("parse+analysis");
        let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
        let stdout = io::stdout();
        let mut out = stdout.lock();
        impact::write_report(&report, format, &mut out)?;
        timings.mark("output");
        if args.timings {
            timings.print_stderr();
        }
        return Ok(());
    }
    let (entries, root_strs) = collect_entries_with_timings(&args, Some(&mut timings))?;

    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());

    let stdout = io::stdout();
    let mut out = stdout.lock();
    match format {
        Format::Json => output::write_json(&root_strs, &entries, &mut out)?,
        Format::Md => output::write_md(&root_strs, &entries, &mut out)?,
        Format::Yml => output::write_yml(&root_strs, &entries, &mut out)?,
        Format::Paths => output::write_paths(&entries, &mut out)?,
        Format::Human => output::write_human(&root_strs, &entries, &mut out)?,
    }
    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }
    Ok(())
}

pub fn run_json(args: SymbolsArgs) -> Result<String> {
    if args.mode == SymbolsMode::SignatureImpact {
        return impact::report_json(args);
    }
    let (entries, root_strs) = collect_entries(&args)?;
    let mut out = Vec::new();
    output::write_json(&root_strs, &entries, &mut out)?;
    String::from_utf8(out).context("symbols JSON output must be UTF-8")
}

fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(f) = format {
        f
    } else if stdout_is_terminal {
        Format::Human
    } else {
        Format::Json
    }
}
