pub use crate::codebase::ts_source::SKIP_DIRS;

/// A node in the dependency graph: a source file, external module, or virtual queue-job node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    /// A source file on disk.
    File(PathBuf),
    /// An exported symbol in a source file.
    Symbol { file: PathBuf, symbol: String },
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
            NodeId::Symbol { file, .. } => Some(file.as_path()),
            NodeId::Module(_) => None,
            NodeId::QueueJob { .. } => None,
        }
    }

    fn is_in_file_universe(&self, universe: &HashSet<PathBuf>) -> bool {
        match self {
            Self::File(path) | Self::Symbol { file: path, .. } => universe.contains(path),
            Self::Module(_) => true,
            Self::QueueJob { queue_file, .. } => universe.contains(queue_file),
        }
    }

    /// Render this node relative to `root` for display.
    pub fn display_name(&self, root: &Path) -> String {
        match self {
            NodeId::File(p) => {
                let rel = p.strip_prefix(root).unwrap_or(p);
                rel.display().to_string()
            }
            NodeId::Symbol { file, symbol } => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                format!("{}#{}", rel.display(), symbol)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    /// Regular TS/JS static import.
    Import,
    /// Type-only import (`import type ...`).
    TypeImport,
    /// Runtime dynamic import (`import("...")`).
    DynamicImport,
    /// Conservative runtime import used for Playwright route reachability.
    /// Unlike ordinary import edges, function-scoped imports are not pruned.
    RouteImport,
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
    /// Statically resolved runtime filesystem resource consumed by a TS/JS file.
    Resource,
    /// React component render relationship: parent component file → rendered child component file.
    ReactRender,
    /// Playwright selector coverage: test file → app/component file matched by
    /// selector analysis (e.g. `data-pw` / `data-testid` attributes, locator
    /// text).  Direction mirrors `TestOf` so that `dependents_of(component)`
    /// returns tests that cover it via selector-based paths.
    Selector,
    /// Swift module import from one Swift file to local files in the imported target.
    SwiftImport,
    /// Swift symbol/member reference from one Swift file to the declaring file.
    SwiftReference,
    /// SwiftPM target dependency fallback edge between package targets.
    SwiftPackageDependency,
    /// C# using directive from one file to local files in the used namespace.
    DotnetUsing,
    /// C# type/member reference from one file to the declaring file.
    DotnetReference,
    /// .NET ProjectReference fallback edge between project source files.
    DotnetProjectDependency,
    /// Terraform/OpenTofu resource reference: a file referencing `<type>.<name>`
    /// → the file declaring that resource/data source.
    TerraformReference,
    /// Terraform/OpenTofu module block reference: the file with the `module` block
    /// → files in the module's local source directory.
    TerraformModuleRef,
    /// Terraform/OpenTofu output consumption: a file referencing
    /// `module.<name>.<output>` → the file declaring that output.
    TerraformOutputRef,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::Import => "import",
            EdgeKind::TypeImport => "type-import",
            EdgeKind::DynamicImport => "dynamic-import",
            EdgeKind::RouteImport => "route-import",
            EdgeKind::Require => "require",
            EdgeKind::TestOf => "test",
            EdgeKind::RouteRef => "route",
            EdgeKind::QueueEnqueue => "queue-enqueue",
            EdgeKind::QueueWorker => "queue-worker",
            EdgeKind::RouteTest => "route-test",
            EdgeKind::Layout => "layout",
            EdgeKind::MarkdownLink => "md",
            EdgeKind::WorkspaceImport => "workspace",
            EdgeKind::PackageDependency => "package",
            EdgeKind::CiInvocation => "ci",
            EdgeKind::HttpCall => "http",
            EdgeKind::ProcessSpawn => "process",
            EdgeKind::AssetImport => "asset",
            EdgeKind::Resource => "resource",
            EdgeKind::ReactRender => "react-render",
            EdgeKind::Selector => "selector",
            EdgeKind::SwiftImport => "swift-import",
            EdgeKind::SwiftReference => "swift-ref",
            EdgeKind::SwiftPackageDependency => "swift-package",
            EdgeKind::DotnetUsing => "dotnet-using",
            EdgeKind::DotnetReference => "dotnet-ref",
            EdgeKind::DotnetProjectDependency => "dotnet-project",
            EdgeKind::TerraformReference => "terraform-ref",
            EdgeKind::TerraformModuleRef => "terraform-module",
            EdgeKind::TerraformOutputRef => "terraform-output",
        }
    }
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

/// Provenance for one statically resolved runtime-resource call.
///
/// The graph deliberately keeps this outside the generic adjacency index: the
/// relationship remains a normal typed edge while callers that need debug or
/// test-impact explanations can retrieve every call site that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceCallSite {
    pub call_kind: String,
    /// One-based source line containing the call expression.
    pub line: usize,
}

impl PartialOrd for ResourceCallSite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceCallSite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.line, &self.call_kind).cmp(&(other.line, &other.call_kind))
    }
}

type ResourceEdgeDetails = HashMap<(PathBuf, PathBuf), Vec<ResourceCallSite>>;

/// A dynamic runtime-resource call that deliberately did not create an edge.
/// Kept alongside the canonical graph so impact rendering can surface only
/// diagnostics that are relevant to a selected path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceGraphDiagnostic {
    pub consumer: PathBuf,
    pub kind: crate::codebase::ts_resources::ResourceDiagnosticKind,
    /// One-based source line containing the dynamic call expression.
    pub line: usize,
}

type ParsedImports<'a> = Vec<(
    &'a PathBuf,
    &'a crate::codebase::ts_source::facts::TsFileFacts,
    HashSet<String>,
)>;
