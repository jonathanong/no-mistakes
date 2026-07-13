use super::TsFactPlan;

impl TsFactPlan {
    pub fn imports() -> Self {
        Self {
            imports: true,
            function_calls: true,
            symbols: false,
            ..Self::default()
        }
    }

    pub fn imports_and_symbols() -> Self {
        Self {
            imports: true,
            function_calls: true,
            symbols: true,
            ..Self::default()
        }
    }

    pub fn is_empty(self) -> bool {
        !self.imports
            && !self.function_calls
            && !self.symbols
            && !self.source
            && !self.route_refs
            && !self.backend_routes
            && !self.queue_usage
            && !self.queue_factory
            && !self.queue_project
            && !self.http_calls
            && !self.process_spawns
            && !self.server_routes
            && !self.react
    }

    pub fn has_domain_facts(self) -> bool {
        self.route_refs
            || self.backend_routes
            || self.queue_usage
            || self.queue_factory
            || self.queue_project
            || self.http_calls
            || self.process_spawns
            || self.server_routes
    }

    pub fn covers(self, required: Self) -> bool {
        (!required.imports || self.imports)
            && (!required.function_calls || self.function_calls)
            && (!required.symbols || self.symbols)
            && (!required.source || self.source)
            && (!required.route_refs || self.route_refs)
            && (!required.backend_routes || self.backend_routes)
            && (!required.queue_usage || self.queue_usage)
            && (!required.queue_factory || self.queue_factory)
            && (!required.queue_project || self.queue_project)
            && (!required.http_calls || self.http_calls)
            && (!required.process_spawns || self.process_spawns)
            && (!required.server_routes || self.server_routes)
            && (!required.react || self.react)
    }
}
