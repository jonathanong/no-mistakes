struct EffectiveScope {
    key: String,
    root: PathBuf,
    tsconfig: Option<PathBuf>,
    config: Option<PathBuf>,
}

fn effective_scope(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> Result<EffectiveScope> {
    let root =
        super::options::resolve_root(string_option(request, "root")?.or(options.root.as_deref()))?;
    let tsconfig = effective_path(
        &root,
        string_option(request, "tsconfig")?.or(options.tsconfig.as_deref()),
    );
    let config = effective_path(
        &root,
        string_option(request, "config")?.or(options.config.as_deref()),
    );
    let key = serde_json::to_string(&(
        root.to_string_lossy(),
        tsconfig.as_ref().map(|path| path.to_string_lossy()),
        config.as_ref().map(|path| path.to_string_lossy()),
    ))?;
    Ok(EffectiveScope {
        key,
        root,
        tsconfig,
        config,
    })
}

fn string_option<'a>(request: &'a AnalyzeReportRequest, name: &str) -> Result<Option<&'a str>> {
    request
        .options
        .get(name)
        .map(|value| {
            value
                .as_str()
                .with_context(|| format!("{name} must be a string"))
        })
        .transpose()
}

fn effective_path(root: &Path, value: Option<&str>) -> Option<PathBuf> {
    value.map(|value| {
        let path = PathBuf::from(value);
        let path = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        crate::codebase::ts_resolver::normalize_path(&path)
    })
}
