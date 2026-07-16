#[doc(hidden)]
pub fn prepare_analysis(
    root: &Path,
    tsconfig_path: Option<&Path>,
) -> anyhow::Result<PreparedServerAnalysis> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new_observed(
        &root,
        session.observer().cloned(),
    );
    let visible_paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let tsconfig = resolve_tsconfig(&root, tsconfig_path, &visible_paths, &sources)?;
    let config =
        crate::config::v2::load_v2_config_from_source_store(&root, None, &visible_paths, &sources)
            .ok();
    let extra_skip = config
        .as_ref()
        .map(|config| config.filesystem.skip_directories.as_slice())
        .unwrap_or(&[]);
    let source_files = discover_source_files_from_visible(&root, extra_skip, &visible_paths);
    let mut fact_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    if let Some(config) = &config {
        configure_fact_context(&mut fact_context, &root, config);
    }
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            &source_files,
            crate::codebase::ts_source::facts::TsFactPlan {
                route_refs: true,
                server_routes: true,
                ..Default::default()
            },
            &fact_context,
            &sources,
        );
    crate::invocation::check_timeout()?;
    Ok(PreparedServerAnalysis {
        root,
        source_files: std::sync::Arc::new(source_files),
        tsconfig,
        config,
        facts,
        session,
    })
}

#[doc(hidden)]
pub fn prepare_analysis_with_shared_facts(
    root: &Path,
    tsconfig: &TsConfig,
    config: &crate::config::v2::NoMistakesConfig,
    source_files: &[PathBuf],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> PreparedServerAnalysis {
    prepare_analysis_with_shared_facts_and_session(
        root,
        tsconfig,
        config,
        source_files,
        shared,
        crate::codebase::analysis_session::AnalysisSession::disabled(),
    )
}

#[doc(hidden)]
pub fn prepare_analysis_with_shared_facts_and_session(
    root: &Path,
    tsconfig: &TsConfig,
    config: &crate::config::v2::NoMistakesConfig,
    source_files: &[PathBuf],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
) -> PreparedServerAnalysis {
    let facts = crate::codebase::ts_source::facts::TsFactMap::from_shared_iter_with_plan(
        source_files.iter().filter_map(|path| {
            shared
                .ts
                .get(path)
                .map(|facts| (path.clone(), facts.ts.clone()))
        }),
        shared.graph_plan(),
    );
    PreparedServerAnalysis {
        root: root.to_path_buf(),
        source_files: std::sync::Arc::new(source_files.to_vec()),
        tsconfig: tsconfig.clone(),
        config: Some(config.clone()),
        facts,
        session,
    }
}

fn resolve_tsconfig(
    root: &Path,
    explicit: Option<&Path>,
    visible_paths: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<TsConfig> {
    let explicit_path = explicit.is_some();
    let path = match explicit {
        Some(path) if path.is_absolute() => Some(path.to_path_buf()),
        Some(path) => Some(root.join(path)),
        None => find_tsconfig_from_visible(root, visible_paths),
    };
    match path {
        Some(path) if explicit_path => {
            crate::codebase::ts_resolver::load_tsconfig_from_source_store(&path, sources)
                .context(format!("loading tsconfig {}", path.display()))
        }
        Some(path) => Ok(
            crate::codebase::ts_resolver::load_tsconfig_from_source_store(&path, sources)
                .unwrap_or_else(|_| empty_tsconfig(root)),
        ),
        None => Ok(empty_tsconfig(root)),
    }
}

fn empty_tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths_dir: root.to_path_buf(),
        paths: Vec::new(),
        base_url: None,
    }
}
