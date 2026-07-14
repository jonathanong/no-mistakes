#[derive(Clone, Copy)]
pub(crate) struct LazyImportFacts<'a> {
    prepared: Option<&'a dyn TsFactLookup>,
    collect_plan: TsFactPlan,
    context: &'a TsFactContext,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    retain_collected: bool,
}

impl<'a> LazyImportFacts<'a> {
    pub(crate) fn new(
        prepared: Option<&'a dyn TsFactLookup>,
        collect_plan: TsFactPlan,
        context: &'a TsFactContext,
    ) -> Self {
        Self {
            prepared,
            collect_plan,
            context,
            sources: None,
            retain_collected: false,
        }
    }

    pub(crate) fn with_source_store(
        mut self,
        sources: &'a crate::codebase::ts_source::SourceStore,
    ) -> Self {
        self.sources = Some(sources);
        self
    }

    pub(crate) fn retain_collected(mut self) -> Self {
        self.retain_collected = true;
        self
    }
}

pub(crate) struct LazyImportBuild<'a> {
    pub(crate) roots: &'a [NodeId],
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) max_depth: Option<usize>,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) allowed: Option<&'a HashSet<EdgeKind>>,
    pub(crate) facts: LazyImportFacts<'a>,
    pub(crate) workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    pub(crate) import_resolution_cache:
        Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
}

struct ExpandedImportNode {
    node: NodeId,
    neighbors: Vec<(NodeId, EdgeKind)>,
    collected: Option<(PathBuf, TsFileFacts)>,
}
