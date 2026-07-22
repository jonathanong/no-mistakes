use super::TsFactPlan;

impl TsFactPlan {
    pub fn include(&mut self, other: Self) {
        self.imports |= other.imports;
        self.function_calls |= other.function_calls;
        self.resources |= other.resources;
        self.symbols |= other.symbols;
        self.source |= other.source;
        self.route_refs |= other.route_refs;
        self.backend_routes |= other.backend_routes;
        self.queue_usage |= other.queue_usage;
        self.queue_factory |= other.queue_factory;
        self.queue_project |= other.queue_project;
        self.http_calls |= other.http_calls;
        self.process_spawns |= other.process_spawns;
        self.server_routes |= other.server_routes;
        self.react |= other.react;
        self.effect_calls |= other.effect_calls;
        self.rsc_environment |= other.rsc_environment;
    }

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
            && !self.resources
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
            && !self.effect_calls
            && !self.rsc_environment
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
            || self.effect_calls
            || self.rsc_environment
    }

    pub fn covers(self, required: Self) -> bool {
        (!required.imports || self.imports)
            && (!required.function_calls || self.function_calls)
            && (!required.resources || self.resources)
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
            && (!required.effect_calls || self.effect_calls)
            && (!required.rsc_environment || self.rsc_environment)
    }
}
