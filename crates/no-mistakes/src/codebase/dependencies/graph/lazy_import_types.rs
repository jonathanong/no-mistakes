#[derive(Clone, Copy)]
pub struct LazyImportFacts<'a> {
    prepared: Option<&'a dyn TsFactLookup>,
    live_cache: Option<&'a dashmap::DashMap<PathBuf, std::sync::Arc<TsFileFacts>>>,
    collect_plan: TsFactPlan,
    context: &'a TsFactContext,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    retain_collected: bool,
}

impl<'a> LazyImportFacts<'a> {
    pub fn new(
        prepared: Option<&'a dyn TsFactLookup>,
        collect_plan: TsFactPlan,
        context: &'a TsFactContext,
    ) -> Self {
        Self {
            prepared,
            live_cache: None,
            collect_plan,
            context,
            sources: None,
            retain_collected: false,
        }
    }

    pub fn with_live_cache(
        mut self,
        live_cache: &'a dashmap::DashMap<PathBuf, std::sync::Arc<TsFileFacts>>,
    ) -> Self {
        self.live_cache = Some(live_cache);
        self
    }

    pub fn with_source_store(
        mut self,
        sources: &'a crate::codebase::ts_source::SourceStore,
    ) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn retain_collected(mut self) -> Self {
        self.retain_collected = true;
        self
    }
}

pub struct LazyImportBuild<'a> {
    pub roots: &'a [NodeId],
    pub tsconfig: &'a TsConfig,
    pub tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    pub max_depth: Option<usize>,
    pub graph_files: &'a GraphFiles,
    pub resolution_visible: Option<&'a dyn crate::codebase::ts_resolver::VisiblePathLookup>,
    pub allowed: Option<&'a HashSet<EdgeKind>>,
    pub facts: LazyImportFacts<'a>,
    pub workspace: &'a crate::codebase::workspace_resolution::IndexedWorkspaceMap,
    pub import_resolution_cache:
        Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
}

pub struct LazyImportWalk {
    pub entries: Vec<NodeEntry>,
    pub facts: Vec<(PathBuf, TsFileFacts)>,
    pub(crate) edges: Vec<CanonicalEdge<NodeId, EdgeKind>>,
    pub nodes: Vec<NodeId>,
}

struct ExpandedImportNode {
    node: NodeId,
    neighbors: Vec<(NodeId, EdgeKind)>,
    collected: Option<(PathBuf, TsFileFacts)>,
}

struct LazyVisit {
    result_order: Option<usize>,
    depth: usize,
    via: Vec<EdgeKind>,
}
