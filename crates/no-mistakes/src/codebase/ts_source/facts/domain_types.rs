use globset::GlobSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TsFactContext {
    pub root: PathBuf,
    pub backend_route_extractors: Vec<BackendRouteExtractor>,
    pub queue_factory_specifier: Option<String>,
    pub queue_factory_function: Option<String>,
    pub queue_factory_glob: Option<GlobSet>,
    pub queue_project_factory_names: Vec<String>,
    pub http_prefixes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BackendRouteExtractor {
    pub register_object: String,
    pub pattern: String,
    pub glob: GlobSet,
}

#[derive(Debug, Clone, Default)]
pub struct BackendRouteFact {
    pub register_object: String,
    pub route: String,
    pub line: u32,
}

impl TsFactContext {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            ..Self::default()
        }
    }

    pub fn add_backend_route_extractor(
        &mut self,
        register_object: String,
        pattern: String,
        glob: GlobSet,
    ) {
        if self.backend_route_extractors.iter().any(|extractor| {
            extractor.register_object == register_object && extractor.pattern == pattern
        }) {
            return;
        }
        self.backend_route_extractors.push(BackendRouteExtractor {
            register_object,
            pattern,
            glob,
        });
    }

    pub fn matches_queue_factory(&self, path: &Path) -> bool {
        self.matches_optional_glob(path, &self.queue_factory_glob)
    }

    fn matches_optional_glob(&self, path: &Path, glob: &Option<GlobSet>) -> bool {
        let Some(glob) = glob else {
            return false;
        };
        self.matches_glob(path, glob)
    }

    pub fn matches_glob(&self, path: &Path, glob: &GlobSet) -> bool {
        path.strip_prefix(&self.root)
            .map(|rel| glob.is_match(rel))
            .unwrap_or(false)
    }
}

impl Default for TsFactContext {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            backend_route_extractors: Vec::new(),
            queue_factory_specifier: None,
            queue_factory_function: None,
            queue_factory_glob: None,
            queue_project_factory_names: Vec::new(),
            http_prefixes: Vec::new(),
        }
    }
}
