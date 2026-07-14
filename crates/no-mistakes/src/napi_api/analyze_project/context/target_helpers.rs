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
