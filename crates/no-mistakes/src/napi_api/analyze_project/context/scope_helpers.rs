#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct EffectiveScopeKey {
    root: PathBuf,
    tsconfig: Option<PathBuf>,
    automatic_tsconfig: bool,
    config: Option<PathBuf>,
}

struct EffectiveScope {
    key: EffectiveScopeKey,
    root: PathBuf,
    tsconfig: Option<PathBuf>,
    automatic_tsconfig: bool,
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
            automatic_tsconfig: self.automatic_tsconfig,
            config: self.config.clone(),
        };
        Ok(self)
    }
}

fn effective_scope(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> Result<EffectiveScope> {
    let root = super::options::resolve_root(
        string_option(request, "root")?.or(options.root.as_deref()),
    )?;
    let inherited_root = super::options::resolve_root(options.root.as_deref())?;
    let request_tsconfig = string_option(request, "tsconfig")?;
    let automatic_tsconfig = request_tsconfig.is_none() && options.tsconfig.is_none();
    let tsconfig = match request_tsconfig {
        Some(path) => effective_path(&root, Some(path)),
        None => effective_path(&inherited_root, options.tsconfig.as_deref()),
    };
    let config = match string_option(request, "config")? {
        Some(path) => effective_path(&root, Some(path)),
        None => effective_path(&inherited_root, options.config.as_deref()),
    };
    let key = EffectiveScopeKey {
        root: root.clone(),
        tsconfig: tsconfig.clone(),
        automatic_tsconfig,
        config: config.clone(),
    };
    Ok(EffectiveScope {
        key,
        root,
        tsconfig,
        automatic_tsconfig,
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
