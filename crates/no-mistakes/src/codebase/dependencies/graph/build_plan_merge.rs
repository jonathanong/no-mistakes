impl GraphBuildPlan {
    pub(crate) fn include(&mut self, other: Self) {
        self.assertion_owners |= other.assertion_owners;
        self.calls |= other.calls;
        self.mock_boundaries |= other.mock_boundaries;
        self.effect_selectors |= other.effect_selectors;
        self.extends |= other.extends;
        self.imports |= other.imports;
        self.route_imports |= other.route_imports;
        self.workspace |= other.workspace;
        self.package |= other.package;
        self.tests |= other.tests;
        self.markdown |= other.markdown;
        self.ci |= other.ci;
        self.workflow_topology |= other.workflow_topology;
        self.routes |= other.routes;
        self.queues |= other.queues;
        self.playwright_routes |= other.playwright_routes;
        self.playwright_selectors |= other.playwright_selectors;
        self.http |= other.http;
        self.process |= other.process;
        self.assets |= other.assets;
        self.resources |= other.resources;
        self.react |= other.react;
        self.symbols |= other.symbols;
        self.symbol_import_selectors |= other.symbol_import_selectors;
        self.dotnet |= other.dotnet;
        self.swift |= other.swift;
        self.terraform |= other.terraform;
        self.language_frontends |= other.language_frontends;
        self.trpc |= other.trpc;
    }
}
