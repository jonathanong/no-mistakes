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
    /// Build canonical GitHub Actions workflow topology edges.
    pub workflow_topology: bool,
    pub routes: bool,
    pub queues: bool,
    pub playwright_routes: bool,
    /// Build `EdgeKind::Selector` edges from playwright analysis.
    pub playwright_selectors: bool,
    pub http: bool,
    pub process: bool,
    pub assets: bool,
    /// Runtime filesystem resource reads and static glob calls in TS/JS.
    pub resources: bool,
    pub react: bool,
    pub symbols: bool,
    pub dotnet: bool,
    pub swift: bool,
    pub terraform: bool,
    pub language_frontends: bool,
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
            workflow_topology: true,
            routes: true,
            queues: true,
            playwright_routes: true,
            playwright_selectors: true,
            http: true,
            process: true,
            assets: true,
            resources: true,
            react: true,
            symbols: false,
            dotnet: true,
            swift: true,
            terraform: true,
            language_frontends: true,
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
                || allowed.contains(&EdgeKind::Require)
                || allowed.contains(&EdgeKind::RequireResolve),
            route_imports: allowed.contains(&EdgeKind::RouteImport),
            workspace: allowed.contains(&EdgeKind::WorkspaceImport)
                || allowed.contains(&EdgeKind::WorkspaceTypeImport)
                || allowed.contains(&EdgeKind::RequireResolve),
            package: allowed.contains(&EdgeKind::PackageDependency),
            tests: allowed.contains(&EdgeKind::TestOf)
                || allowed.contains(&EdgeKind::VitestSetup(VitestSetupField::SetupFiles))
                || allowed.contains(&EdgeKind::VitestSetup(VitestSetupField::GlobalSetup)),
            markdown: allowed.contains(&EdgeKind::MarkdownLink),
            ci: allowed.contains(&EdgeKind::CiInvocation),
            workflow_topology: allowed.contains(&EdgeKind::WorkflowJob)
                || allowed.contains(&EdgeKind::WorkflowStep)
                || allowed.contains(&EdgeKind::WorkflowNeeds)
                || allowed.contains(&EdgeKind::WorkflowUses)
                || allowed.contains(&EdgeKind::WorkflowRun)
                || allowed.contains(&EdgeKind::WorkflowArtifact),
            routes: allowed.contains(&EdgeKind::RouteRef),
            queues: allowed.contains(&EdgeKind::QueueEnqueue)
                || allowed.contains(&EdgeKind::QueueWorker),
            playwright_routes: allowed.contains(&EdgeKind::RouteTest)
                || allowed.contains(&EdgeKind::Layout),
            playwright_selectors: allowed.contains(&EdgeKind::Selector),
            http: allowed.contains(&EdgeKind::HttpCall),
            process: allowed.contains(&EdgeKind::ProcessSpawn),
            assets: allowed.contains(&EdgeKind::AssetImport),
            resources: allowed.contains(&EdgeKind::Resource),
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
            language_frontends: allowed_requests_language_frontends(allowed),
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
        self.dotnet |= other.dotnet;
        self.swift |= other.swift;
        self.terraform |= other.terraform;
        self.language_frontends |= other.language_frontends;
    }

    pub fn with_symbols(mut self, symbols: bool) -> Self {
        self.symbols = symbols;
        self
    }

    pub(crate) fn ts_fact_plan(self) -> TsFactPlan {
        TsFactPlan {
            imports: self.imports || self.route_imports || self.workspace || self.assets,
            function_calls: self.imports || self.workspace || self.assets || self.symbols || self.resources,
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
            ..TsFactPlan::default()
        }
    }
}
