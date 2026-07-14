use std::sync::Arc;

#[derive(Clone, Default)]
pub struct CheckFactPlan {
    pub imports: bool,
    pub symbols: bool,
    pub react: bool,
    pub react_usages: bool,
    pub queue: bool,
    pub queue_factory_names: Vec<String>,
    pub integration: bool,
    pub integration_runner_configs:
        Option<Arc<crate::integration_tests::PreparedIntegrationRunnerConfigs>>,
    pub dynamic_imports: bool,
    pub nextjs_caching: bool,
    pub storybook: bool,
    pub source: bool,
    pub raw_source: bool,
    pub graph: crate::codebase::ts_source::facts::TsFactPlan,
    pub graph_context: crate::codebase::ts_source::facts::TsFactContext,
}

impl CheckFactPlan {
    pub(crate) fn include(&mut self, other: Self) {
        self.imports |= other.imports;
        self.symbols |= other.symbols;
        self.react |= other.react;
        self.react_usages |= other.react_usages;
        self.queue |= other.queue;
        self.queue_factory_names.extend(other.queue_factory_names);
        self.queue_factory_names.sort();
        self.queue_factory_names.dedup();
        self.integration |= other.integration;
        if self.integration_runner_configs.is_none() {
            self.integration_runner_configs = other.integration_runner_configs;
        }
        self.dynamic_imports |= other.dynamic_imports;
        self.nextjs_caching |= other.nextjs_caching;
        self.storybook |= other.storybook;
        self.source |= other.source;
        self.raw_source |= other.raw_source;
        self.graph.include(other.graph);
        self.graph_context.include(other.graph_context);
    }

    /// TS graph fact shapes guaranteed for every file collected with this
    /// plan. Several check-only flags populate the canonical `TsFileFacts`
    /// fields too; recording only `graph` would make a strict prepared graph
    /// reject facts that were already collected in the same parse pass.
    pub(super) fn collected_ts_plan(&self) -> crate::codebase::ts_source::facts::TsFactPlan {
        let mut plan = self.graph;
        if self.imports {
            plan.include(crate::codebase::ts_source::facts::TsFactPlan::imports());
        }
        plan.symbols |= self.symbols;
        plan.source |= self.source || self.raw_source;
        plan.react |= self.react;
        plan.queue_project |= self.queue;
        plan
    }
}
