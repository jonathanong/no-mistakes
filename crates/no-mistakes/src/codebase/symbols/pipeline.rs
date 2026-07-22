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
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<(Vec<FileEntry>, Vec<String>)> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let abs_files = resolve_input_files(&args.files, root, &cwd);
    let kind_filter = build_kind_filter(&args.kinds);
    let visible_paths = visible_files.iter().cloned().collect::<Vec<_>>();
    let catalog = match args.tsconfig.as_deref() {
        None => crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
            root,
            &[root.to_path_buf()],
            &visible_paths,
        ),
        Some(path) => {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::TsConfigCatalog::forced(
                root,
                tsconfig.clone(),
                Some(crate::codebase::ts_resolver::normalize_path(&path)),
            )
        }
    };
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        &catalog,
        visible_files,
        session,
    );
    // Keep resolver targets in the same lexical namespace as the frozen facts.
    // In particular, a tsconfig alias below a symlinked root can resolve to its
    // real target even though the report's files are keyed by the symlink path.
    let remapper = crate::codebase::ts_source::FrozenPathRemapper::from_paths(
        visible_files.iter().cloned(),
    );
    let entries = abs_files
        .iter()
        .map(|path| {
            let file = facts
                .ts
                .get(path)
                .filter(|facts| facts.symbols.is_some())
                .or_else(|| supplemental.ts.get(path).filter(|facts| facts.symbols.is_some()))
                .or_else(|| facts.ts.get(path))
                .or_else(|| supplemental.ts.get(path))
                .with_context(|| format!("reading {}", path.display()))?;
            let symbols = file
                .legacy_symbol_parse_error
                .is_none()
                .then(|| file.legacy_symbols.clone().or_else(|| file.symbols.clone()))
                .flatten()
                .with_context(|| match file
                    .legacy_symbol_parse_error
                    .as_ref()
                    .or(file.parse_error.as_ref())
                {
                    Some(error) => format!("extracting symbols from {}: {error}", path.display()),
                    None => "shared analyzeProject facts are missing symbols".to_string(),
                })?;
            build_entry_from_symbols(
                path,
                root,
                &resolver,
                &remapper,
                symbols.as_ref().clone(),
                args.include,
                kind_filter.as_ref(),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let root_strs = args
        .files
        .iter()
        .map(|file| file.display().to_string())
        .collect();
    Ok((entries, root_strs))
}

fn collect_entries_with_timings(
    args: &SymbolsArgs,
    mut timings: Option<&mut crate::codebase::timing::PhaseTimings>,
) -> Result<(Vec<FileEntry>, Vec<String>)> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = resolve_root(args.root.as_deref(), &cwd);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let visible_snapshot = session.visible_paths(&root);
    let visible_paths = visible_snapshot.paths_for(&root);
    let tsconfig = session
        .tsconfig(&root, args.tsconfig.as_deref())
        .or_else(|error| {
            if args.tsconfig.is_some() {
                Err(error)
            } else {
                Ok(std::sync::Arc::new(TsConfig {
                    dir: root.clone(),
                    paths: Vec::new(),
                    paths_dir: root.clone(),
                    base_url: None,
                }))
            }
        })?;
    let visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<std::collections::HashSet<_>>();
    let abs_files = resolve_input_files(&args.files, &root, &cwd);
    if let Some(timings) = &mut timings {
        timings.mark("search");
    }

    let kind_filter = build_kind_filter(&args.kinds);
    if let Some(timings) = &mut timings {
        timings.mark("ingest");
    }

    let symbol_paths = crate::codebase::ts_source::deduplicate_analysis_paths(abs_files.iter());
    let symbols = symbol_paths
        .par_iter()
        .map(|path| {
            let normalized = crate::codebase::ts_resolver::normalize_path(path);
            let source = session
                .read_source(&normalized)
                .with_context(|| format!("reading {}", normalized.display()))?;
            let symbols = session
                .with_legacy_symbols_program(
                    &normalized,
                    &source,
                    |program, source, _diagnostic| {
                        crate::codebase::ts_symbols::extract_symbols_from_program(program, source)
                    },
                )
                .with_context(|| format!("extracting symbols from {}", normalized.display()))?;
            Ok((normalized, symbols))
        })
        .collect::<Result<std::collections::HashMap<_, _>>>()?;
    let catalog = match args.tsconfig.as_deref() {
        None => crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
            &root,
            std::slice::from_ref(&root),
            &visible_paths,
        ),
        Some(path) => {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                (*tsconfig).clone(),
                Some(crate::codebase::ts_resolver::normalize_path(&path)),
            )
        }
    };
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        &catalog,
        &visible_files,
        &session,
    );
    let remapper = crate::codebase::ts_source::FrozenPathRemapper::from_paths(
        visible_paths.iter().cloned(),
    );
    let entries: Vec<FileEntry> = abs_files
        .par_iter()
        .map(|abs| {
            let symbols = symbols
                .get(&crate::codebase::ts_resolver::normalize_path(abs))
                .with_context(|| format!("reading {}", abs.display()))?;
            build_entry_from_symbols(
                abs,
                &root,
                &resolver,
                &remapper,
                symbols.clone(),
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
