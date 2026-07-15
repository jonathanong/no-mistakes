fn symbol_target_files(options: &AnalyzeProjectOptions, root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for request in &options.reports {
        if request.report_type != "symbols" {
            continue;
        }
        let raw = super::symbols_options(request, options)?;
        let parsed: crate::napi_api::options::SymbolOptions = serde_json::from_str(&raw)?;
        files.extend(parsed.files.into_iter().map(|file| {
            let file = PathBuf::from(file);
            let path = if file.is_absolute() {
                file
            } else {
                root.join(file)
            };
            crate::codebase::ts_resolver::normalize_path(&path)
        }));
    }
    Ok(files)
}

fn legacy_symbol_target_files(
    options: &AnalyzeProjectOptions,
    root: &Path,
) -> Result<std::collections::HashSet<PathBuf>> {
    let mut files = std::collections::HashSet::new();
    for request in options
        .reports
        .iter()
        .filter(|request| request.report_type == "symbols")
    {
        let raw = super::symbols_options(request, options)?;
        let parsed: crate::napi_api::options::SymbolOptions = serde_json::from_str(&raw)?;
        let args = crate::napi_api::codebase::build_symbols_args(parsed)?;
        if args.mode == crate::codebase::symbols::SymbolsMode::SignatureImpact {
            continue;
        }
        files.extend(args.files.into_iter().map(|file| authoritative_path(root, file)));
    }
    Ok(files)
}

fn prepare_import_usage_views(
    options: &AnalyzeProjectOptions,
    root: &Path,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<(
    HashMap<String, crate::codebase::import_usages::PreparedImportUsages>,
    Vec<PathBuf>,
)> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let mut views = HashMap::new();
    let mut files = Vec::new();
    for request in &options.reports {
        if request.report_type != "importUsages" {
            continue;
        }
        let key = super::import_usages_options(request, options)?;
        if views.contains_key(&key) {
            continue;
        }
        let parsed = serde_json::from_str(key.as_str())?;
        let args = crate::napi_api::codebase::build_import_usages_args(parsed);
        let prepared =
            crate::codebase::import_usages::prepare_file_universe(&args, root, &cwd, session)?;
        files.extend(prepared.files().iter().cloned());
        views.insert(key, prepared);
    }
    files.sort();
    files.dedup();
    Ok((views, files))
}

fn authoritative_report_files(
    options: &AnalyzeProjectOptions,
    root: &Path,
) -> Result<Vec<PathBuf>> {
    let mut files = symbol_target_files(options, root)?;
    for request in &options.reports {
        if super::graph_direction(&request.report_type).is_some() {
            files.extend(
                super::traverse_args(request, options)?
                    .files
                    .into_iter()
                    .map(|path| authoritative_path(root, path)),
            );
        } else if request.report_type == "effects" {
            if let Some(entry) = super::options::effects_options(request, options)?.entry {
                files.push(authoritative_path(root, PathBuf::from(entry)));
            }
        } else if request.report_type == "rscCallers" {
            if let Some(component) =
                super::options::rsc_callers_options(request, options)?.component
            {
                files.push(authoritative_path(root, PathBuf::from(component)));
            }
        }
    }
    files.retain(|path| path.is_file());
    files.sort();
    files.dedup();
    Ok(files)
}

fn authoritative_path(root: &Path, path: PathBuf) -> PathBuf {
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    crate::codebase::ts_resolver::normalize_path(&path)
}

fn has_server_report(options: &AnalyzeProjectOptions) -> bool {
    options
        .reports
        .iter()
        .any(|request| super::is_server_report(&request.report_type))
}
