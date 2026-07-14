/// Selects which edge producers run while building a dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GraphBuildPlan {
    pub imports: bool,
    pub route_imports: bool,
    pub workspace: bool,
    pub package: bool,
    pub tests: bool,
    pub markdown: bool,
    pub ci: bool,
    pub routes: bool,
    pub queues: bool,
    pub playwright_routes: bool,
    /// Build `EdgeKind::Selector` edges from playwright analysis.
    pub playwright_selectors: bool,
    pub http: bool,
    pub process: bool,
    pub assets: bool,
    pub react: bool,
    pub symbols: bool,
    pub dotnet: bool,
    pub swift: bool,
    pub terraform: bool,
}

impl GraphBuildPlan {
    pub fn all() -> Self {
        Self {
            imports: true,
            // RouteImport is an alternate, deliberately conservative import
            // view. Legacy unfiltered traversal must opt in by name instead
            // of unioning it with ordinary call-pruned imports.
            route_imports: false,
            workspace: true,
            package: true,
            tests: true,
            markdown: true,
            ci: true,
            routes: true,
            queues: true,
            playwright_routes: true,
            playwright_selectors: true,
            http: true,
            process: true,
            assets: true,
            react: true,
            symbols: false,
            dotnet: true,
            swift: true,
            terraform: true,
        }
    }

    /// Full test-impact graph without conservative route-import reachability.
    ///
    /// Playwright selector analysis still uses route-import edges internally,
    /// but generic test impact must retain ordinary call-scope pruning so an
    /// import in a never-called loader does not select unrelated tests.
    pub fn test_impact() -> Self {
        Self {
            route_imports: false,
            ..Self::all()
        }
    }

    /// Minimal plan for import-only traversal (no routes, queues, http, etc.).
    pub fn imports_and_workspace() -> Self {
        Self {
            imports: true,
            workspace: true,
            ..Self::default()
        }
    }

    pub fn from_allowed(allowed: Option<&HashSet<EdgeKind>>) -> Self {
        let Some(allowed) = allowed else {
            return Self::all();
        };
        Self {
            imports: allowed.contains(&EdgeKind::Import)
                || allowed.contains(&EdgeKind::TypeImport)
                || allowed.contains(&EdgeKind::DynamicImport)
                || allowed.contains(&EdgeKind::Require),
            route_imports: allowed.contains(&EdgeKind::RouteImport),
            workspace: allowed.contains(&EdgeKind::WorkspaceImport),
            package: allowed.contains(&EdgeKind::PackageDependency),
            tests: allowed.contains(&EdgeKind::TestOf),
            markdown: allowed.contains(&EdgeKind::MarkdownLink),
            ci: allowed.contains(&EdgeKind::CiInvocation),
            routes: allowed.contains(&EdgeKind::RouteRef),
            queues: allowed.contains(&EdgeKind::QueueEnqueue)
                || allowed.contains(&EdgeKind::QueueWorker),
            playwright_routes: allowed.contains(&EdgeKind::RouteTest)
                || allowed.contains(&EdgeKind::Layout),
            playwright_selectors: allowed.contains(&EdgeKind::Selector),
            http: allowed.contains(&EdgeKind::HttpCall),
            process: allowed.contains(&EdgeKind::ProcessSpawn),
            assets: allowed.contains(&EdgeKind::AssetImport),
            react: allowed.contains(&EdgeKind::ReactRender),
            symbols: false,
            dotnet: allowed.contains(&EdgeKind::DotnetUsing)
                || allowed.contains(&EdgeKind::DotnetReference)
                || allowed.contains(&EdgeKind::DotnetProjectDependency),
            swift: allowed.contains(&EdgeKind::SwiftImport)
                || allowed.contains(&EdgeKind::SwiftReference)
                || allowed.contains(&EdgeKind::SwiftPackageDependency),
            terraform: allowed.contains(&EdgeKind::TerraformReference)
                || allowed.contains(&EdgeKind::TerraformModuleRef)
                || allowed.contains(&EdgeKind::TerraformOutputRef),
        }
    }

    pub(crate) fn include(&mut self, other: Self) {
        self.imports |= other.imports;
        self.route_imports |= other.route_imports;
        self.workspace |= other.workspace;
        self.package |= other.package;
        self.tests |= other.tests;
        self.markdown |= other.markdown;
        self.ci |= other.ci;
        self.routes |= other.routes;
        self.queues |= other.queues;
        self.playwright_routes |= other.playwright_routes;
        self.playwright_selectors |= other.playwright_selectors;
        self.http |= other.http;
        self.process |= other.process;
        self.assets |= other.assets;
        self.react |= other.react;
        self.symbols |= other.symbols;
        self.dotnet |= other.dotnet;
        self.swift |= other.swift;
        self.terraform |= other.terraform;
    }

    pub fn with_symbols(mut self, symbols: bool) -> Self {
        self.symbols = symbols;
        self
    }

    pub(crate) fn ts_fact_plan(self) -> TsFactPlan {
        TsFactPlan {
            imports: self.imports || self.route_imports || self.workspace || self.assets,
            function_calls: self.imports || self.workspace || self.assets || self.symbols,
            symbols: self.symbols || self.queues,
            react: self.react,
            route_refs: self.routes,
            backend_routes: self.routes || self.http,
            queue_usage: self.queues,
            queue_factory: self.queues,
            queue_project: self.queues,
            http_calls: self.http,
            process_spawns: self.process,
            ..TsFactPlan::default()
        }
    }
}

fn graph_plan_needs_config(plan: GraphBuildPlan) -> bool {
    plan.routes
        || plan.queues
        || plan.http
        || plan.tests
        || plan.dotnet
        || plan.swift
        || plan.terraform
}

fn effective_ts_fact_plan(
    plan: GraphBuildPlan,
    options: Option<&GraphConfigOptions>,
) -> TsFactPlan {
    let mut fact_plan = plan.ts_fact_plan();
    let route_refs_configured = options.is_some_and(route_ref_facts_configured);
    let route_backend_configured = options.is_some_and(route_backend_facts_configured);
    let http_configured = options.is_some_and(http_facts_configured);
    let queue_configured = options.is_some_and(queue_facts_configured);

    fact_plan.route_refs &= route_refs_configured;
    fact_plan.backend_routes &= route_backend_configured || http_configured;
    fact_plan.http_calls &= http_configured;
    fact_plan.symbols = plan.symbols || (fact_plan.symbols && queue_configured);
    fact_plan.queue_usage &= queue_configured;
    fact_plan.queue_factory &= queue_configured;
    fact_plan.queue_project &= queue_configured;
    fact_plan.server_routes = options.is_some_and(|options| {
        options.project_route_globset.is_some() && (plan.routes || plan.swift)
    });
    fact_plan
}
