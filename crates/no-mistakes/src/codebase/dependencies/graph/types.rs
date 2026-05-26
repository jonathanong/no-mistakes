pub use crate::codebase::ts_source::SKIP_DIRS;

/// A node in the dependency graph: a source file, external module, or virtual queue-job node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeId {
    /// A source file on disk.
    File(PathBuf),
    /// A bare external module specifier that is not resolved to a local file.
    Module(String),
    /// A virtual job node representing one (queue, jobName) pair.
    QueueJob { queue_file: PathBuf, job: String },
}

impl NodeId {
    /// Return the underlying file path, if this is a `File` node.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            NodeId::File(p) => Some(p.as_path()),
            NodeId::Module(_) => None,
            NodeId::QueueJob { .. } => None,
        }
    }

    /// Render this node relative to `root` for display.
    pub fn display_name(&self, root: &Path) -> String {
        match self {
            NodeId::File(p) => {
                let rel = p.strip_prefix(root).unwrap_or(p);
                rel.display().to_string()
            }
            NodeId::Module(specifier) => specifier.clone(),
            NodeId::QueueJob { queue_file, job } => {
                let rel = queue_file
                    .strip_prefix(root)
                    .unwrap_or(queue_file.as_path());
                format!("{}#{}", rel.display(), job)
            }
        }
    }
}

/// The kind of dependency edge connecting two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    /// Regular TS/JS static import.
    Import,
    /// Type-only import (`import type ...`).
    TypeImport,
    /// Runtime dynamic import (`import("...")`).
    DynamicImport,
    /// CommonJS `require("...")` call.
    Require,
    /// Test correspondence: `foo.mts` ↔ `foo.test.mts`.
    TestOf,
    /// Frontend/backend route reference: ref_file → route_def_file.
    RouteRef,
    /// Enqueue site → QueueJob virtual node.
    QueueEnqueue,
    /// QueueJob virtual node → worker/processor file.
    QueueWorker,
    /// Playwright test ↔ frontend page file.
    RouteTest,
    /// Next.js App Router page → inherited layout/template/error file.
    Layout,
    /// Markdown link: `*.md` → linked file.
    MarkdownLink,
    /// Cross-workspace package import (via npm workspace resolution).
    WorkspaceImport,
    /// Dependency declared in a package.json dependency field.
    PackageDependency,
    /// CI workflow invokes a binary: `*.yml` → `src/bin/*.rs`.
    CiInvocation,
    /// HTTP call from a client file to a backend route-definition file.
    HttpCall,
    /// Process spawn: a file launches another file via `spawn`/`exec`/playwright webServer.
    ProcessSpawn,
    /// Explicit relative import of a non-code asset such as CSS, JSON, image, or wasm.
    AssetImport,
    /// React component render relationship: parent component file → rendered child component file.
    ReactRender,
    /// Playwright selector coverage: test file → app/component file matched by
    /// selector analysis (e.g. `data-pw` / `data-testid` attributes, locator
    /// text).  Direction mirrors `TestOf` so that `dependents_of(component)`
    /// returns tests that cover it via selector-based paths.
    Selector,
}

/// A single node in the traversal result.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEntry {
    /// The graph node (file or virtual queue-job).
    pub node: NodeId,
    /// Traversal depth (1 = direct dep/dependent, 2 = transitive, etc.).
    pub depth: usize,
    /// Edge kinds that led to this node (deduped, sorted).
    pub via: Vec<EdgeKind>,
}

type EdgeMap = HashMap<NodeId, Vec<(NodeId, EdgeKind)>>;

// An edge in both directions: (from, to, kind).
type Edge = (NodeId, NodeId, EdgeKind);

type ParsedImports<'a> = Vec<(
    &'a PathBuf,
    &'a crate::codebase::ts_source::facts::TsFileFacts,
)>;

/// Selects which edge producers run while building a dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphBuildPlan {
    pub imports: bool,
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
}

impl GraphBuildPlan {
    pub fn all() -> Self {
        Self {
            imports: true,
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
        }
    }

    pub(crate) fn ts_fact_plan(self) -> TsFactPlan {
        TsFactPlan {
            imports: self.imports || self.workspace || self.assets,
            function_calls: self.imports || self.workspace || self.assets,
            react: self.react,
            symbols: self.queues,
            route_refs: self.routes,
            backend_routes: self.routes || self.http,
            queue_usage: self.queues,
            queue_factory: self.queues,
            http_calls: self.http,
            process_spawns: self.process,
            ..TsFactPlan::default()
        }
    }
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
    fact_plan.symbols &= queue_configured;
    fact_plan.queue_usage &= queue_configured;
    fact_plan.queue_factory &= queue_configured;
    fact_plan
}
