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

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::Import => "import",
            EdgeKind::TypeImport => "type-import",
            EdgeKind::DynamicImport => "dynamic-import",
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
            EdgeKind::ReactRender => "react-render",
            EdgeKind::Selector => "selector",
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

type ParsedImports<'a> = Vec<(
    &'a PathBuf,
    &'a crate::codebase::ts_source::facts::TsFileFacts,
    HashSet<String>,
)>;
