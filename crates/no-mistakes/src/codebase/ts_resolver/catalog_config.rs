type ConfigPathMappings = (Vec<(String, Vec<String>)>, PathBuf);

#[derive(Clone)]
struct EffectiveConfig {
    dir: PathBuf,
    paths: Option<ConfigPathMappings>,
    base_url: Option<PathBuf>,
    files: Option<BTreeSet<PathBuf>>,
    includes: Option<Vec<PatternInput>>,
    excludes: Option<Vec<PatternInput>>,
    allow_js: Option<bool>,
    out_dir: Option<PathBuf>,
    module_resolution: Option<String>,
    references: Vec<PathBuf>,
    identity: BTreeSet<PathBuf>,
}

#[derive(Clone)]
struct PatternInput {
    base: PathBuf,
    value: String,
}

impl EffectiveConfig {
    fn new(path: PathBuf, dir: PathBuf) -> Self {
        let mut identity = BTreeSet::new();
        identity.insert(path);
        Self {
            dir,
            paths: None,
            base_url: None,
            files: None,
            includes: None,
            excludes: None,
            allow_js: None,
            out_dir: None,
            module_resolution: None,
            references: Vec::new(),
            identity,
        }
    }

    fn inherit(&mut self, base: Self) {
        if base.paths.is_some() { self.paths = base.paths; }
        if base.base_url.is_some() { self.base_url = base.base_url; }
        if base.files.is_some() { self.files = base.files; }
        if base.includes.is_some() { self.includes = base.includes; }
        if base.excludes.is_some() { self.excludes = base.excludes; }
        if base.allow_js.is_some() { self.allow_js = base.allow_js; }
        if base.out_dir.is_some() { self.out_dir = base.out_dir; }
        if base.module_resolution.is_some() { self.module_resolution = base.module_resolution; }
        self.identity.extend(base.identity);
        // TypeScript does not inherit `references` through `extends`.
    }

    fn tsconfig(&self) -> TsConfig {
        let (paths, paths_dir) = self.paths.clone().unwrap_or_else(|| (Vec::new(), self.dir.clone()));
        TsConfig {
            dir: self.dir.clone(),
            paths,
            paths_dir,
            base_url: self.base_url.clone(),
        }
    }

    fn matcher(&self) -> ConfigMatcher {
        let excludes = self.excludes.clone().unwrap_or_else(|| default_excludes(&self.dir));
        ConfigMatcher {
            dir: self.dir.clone(),
            real_dir: real_path(&self.dir).unwrap_or_else(|| self.dir.clone()),
            files: self.files.clone(),
            includes: self.includes.as_ref().map(|rules| {
                rules.iter().filter_map(|rule| GlobRule::new(&rule.base, &rule.value)).collect()
            }),
            excludes: excludes.into_iter().filter_map(|rule| GlobRule::new(&rule.base, &rule.value)).collect(),
            out_dir: self.out_dir.clone(),
            allow_js: self.allow_js.unwrap_or(false),
        }
    }
}

include!("catalog_config_apply.rs");
include!("catalog_config_values.rs");
