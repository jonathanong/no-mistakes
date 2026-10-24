impl GraphBuildPlan {
    pub fn with_symbols(mut self, symbols: bool) -> Self {
        self.symbols = symbols;
        self
    }

    /// Import-only plans can walk reachable files lazily. Any other domain flag
    /// requires eager facts for the whole visible universe.
    pub fn is_lazy_import_plan(self) -> bool {
        let mut domains = self;
        domains.imports = false;
        domains.route_imports = false;
        self.imports && domains == Self::default()
    }

    pub(crate) fn ts_fact_plan(self) -> TsFactPlan {
        TsFactPlan {
            imports: self.imports || self.route_imports || self.workspace || self.assets,
            function_calls: self.calls
                || self.extends
                || self.imports
                || self.workspace
                || self.assets
                || self.symbols
                || self.resources,
            call_eligibility: self.assertion_owners,
            value_flow: self.calls,
            resources: self.resources,
            symbols: self.symbols || self.queues,
            react: self.react,
            route_refs: self.routes,
            backend_routes: self.routes || self.http,
            queue_usage: self.queues,
            queue_factory: self.queues,
            queue_project: self.queues,
            http_calls: self.http,
            process_spawns: self.process,
            trpc_router: self.trpc,
            trpc_calls: self.trpc,
            ..TsFactPlan::default()
        }
    }
}
