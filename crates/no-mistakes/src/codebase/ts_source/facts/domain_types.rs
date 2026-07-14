use globset::GlobSet;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TsFactContext {
    pub root: PathBuf,
    pub backend_route_extractors: Vec<BackendRouteExtractor>,
    pub queue_factory_specifier: Option<String>,
    pub queue_factory_function: Option<String>,
    pub queue_factory_glob: Option<GlobSet>,
    pub queue_project_factory_names: Vec<String>,
    pub http_prefixes: Vec<String>,
    pub effect_functions: HashMap<String, Option<String>>,
    pub visible_files: Option<Arc<HashSet<PathBuf>>>,
    pub(crate) server_route_filter: Option<ServerRouteFactFilter>,
}

#[derive(Clone)]
pub(crate) struct ServerRouteFactFilter {
    glob: GlobSet,
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
}

impl std::fmt::Debug for ServerRouteFactFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerRouteFactFilter")
            .field("test_filter", &self.test_filter.is_some())
            .finish_non_exhaustive()
    }
}

impl ServerRouteFactFilter {
    pub(crate) fn new(
        glob: GlobSet,
        test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
    ) -> Self {
        Self { glob, test_filter }
    }

    fn is_match(&self, root: &Path, path: &Path) -> bool {
        path.strip_prefix(root)
            .is_ok_and(|rel| self.glob.is_match(rel))
            && !self
                .test_filter
                .as_ref()
                .is_some_and(|filter| filter.is_match(root, path))
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectCallFact {
    pub line: usize,
    pub callee: String,
    pub category: Option<String>,
    pub caller: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RscEnvironmentFact {
    Server,
    Client,
    Unknown,
}

impl TsFactContext {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            ..Self::default()
        }
    }

    pub(crate) fn set_server_route_filter(
        &mut self,
        glob: GlobSet,
        test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
    ) {
        self.server_route_filter = Some(ServerRouteFactFilter::new(glob, test_filter));
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

    pub fn set_visible_files(&mut self, files: impl IntoIterator<Item = PathBuf>) {
        self.visible_files = Some(Arc::new(
            files
                .into_iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
                .collect(),
        ));
    }

    pub(crate) fn include(&mut self, other: Self) {
        for extractor in other.backend_route_extractors {
            self.add_backend_route_extractor(
                extractor.register_object,
                extractor.pattern,
                extractor.glob,
            );
        }
        self.queue_factory_specifier = self
            .queue_factory_specifier
            .take()
            .or(other.queue_factory_specifier);
        self.queue_factory_function = self
            .queue_factory_function
            .take()
            .or(other.queue_factory_function);
        self.queue_factory_glob = self.queue_factory_glob.take().or(other.queue_factory_glob);
        self.queue_project_factory_names
            .extend(other.queue_project_factory_names);
        self.queue_project_factory_names.sort();
        self.queue_project_factory_names.dedup();
        self.http_prefixes.extend(other.http_prefixes);
        self.http_prefixes.sort();
        self.http_prefixes.dedup();
        self.effect_functions.extend(other.effect_functions);
        self.server_route_filter = self
            .server_route_filter
            .take()
            .or(other.server_route_filter);
        let mut visible = self
            .visible_files
            .take()
            .map(|files| files.iter().cloned().collect::<HashSet<_>>())
            .unwrap_or_default();
        if let Some(other_visible) = other.visible_files {
            visible.extend(other_visible.iter().cloned());
        }
        if !visible.is_empty() {
            self.visible_files = Some(Arc::new(visible));
        }
    }

    pub(crate) fn matches_server_route(&self, path: &Path) -> bool {
        self.server_route_filter
            .as_ref()
            .is_none_or(|filter| filter.is_match(&self.root, path))
    }

    pub fn matches_queue_factory(&self, path: &Path) -> bool {
        self.queue_factory_glob
            .as_ref()
            .map(|glob| self.matches_glob(path, glob))
            .unwrap_or(true)
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
            effect_functions: HashMap::new(),
            visible_files: None,
            server_route_filter: None,
        }
    }
}
