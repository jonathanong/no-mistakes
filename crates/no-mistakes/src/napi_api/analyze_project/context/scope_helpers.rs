#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct EffectiveScopeKey {
    root: PathBuf,
    tsconfig: Option<PathBuf>,
    config: Option<PathBuf>,
}

struct EffectiveScope {
    key: EffectiveScopeKey,
    root: PathBuf,
    tsconfig: Option<PathBuf>,
    config: Option<PathBuf>,
}

impl EffectiveScope {
    fn normalize_automatic_paths(
        mut self,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> Result<Self> {
        let paths = visible_paths.paths_for(&self.root);
        if self.tsconfig.is_none() {
            self.tsconfig =
                crate::codebase::ts_resolver::find_tsconfig_from_visible(&self.root, &paths);
        }
        if self.config.is_none() {
            self.config = crate::config::find_automatic_config_path_from_visible(
                &self.root,
                &[".no-mistakes"],
                &paths,
            )?;
        }
        self.key = EffectiveScopeKey {
            root: self.root.clone(),
            tsconfig: self.tsconfig.clone(),
            config: self.config.clone(),
        };
        Ok(self)
    }
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
    let key = EffectiveScopeKey {
        root: root.clone(),
        tsconfig: tsconfig.clone(),
        config: config.clone(),
    };
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

fn traversal_report_keys(
    options: &AnalyzeProjectOptions,
) -> Result<(
    std::collections::HashSet<String>,
    std::collections::HashSet<String>,
)> {
    let mut queue = std::collections::HashSet::new();
    let mut server = std::collections::HashSet::new();
    for request in &options.reports {
        if !matches!(
            request.report_type.as_str(),
            "queueEdges" | "queueRelated" | "serverRouteEdges" | "serverRouteRelated"
        ) {
            continue;
        }
        let raw = project_options(request, options)?;
        let parsed: ProjectOptions = serde_json::from_str(&raw)?;
        if matches!(request.report_type.as_str(), "queueEdges" | "queueRelated") {
            queue.insert(canonical_filter_key(&parsed.filters)?);
        } else {
            server.insert(canonical_filter_key(&server_filters(
                &request.report_type,
                &parsed,
            ))?);
        }
    }
    Ok((queue, server))
}
