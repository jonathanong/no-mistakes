#[derive(Clone, Copy)]
pub(crate) struct LazyImportFacts<'a> {
    prepared: Option<&'a dyn TsFactLookup>,
    collect_plan: TsFactPlan,
    context: &'a TsFactContext,
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
            retain_collected: false,
        }
    }

    pub(crate) fn retain_collected(mut self) -> Self {
        self.retain_collected = true;
        self
    }
}

struct ExpandedImportNode {
    node: NodeId,
    neighbors: Vec<(NodeId, EdgeKind)>,
    collected: Option<(PathBuf, TsFileFacts)>,
}
