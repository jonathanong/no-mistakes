/// The kind of dependency edge connecting two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum EdgeKind {
    /// Regular TS/JS static import.
    Import,
    /// Type-only import (`import type ...`).
    TypeImport,
    /// Runtime dynamic import (`import("...")`).
    DynamicImport,
    /// Conservative runtime import used for Playwright route reachability.
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
    /// Playwright selector coverage: test file → app/component file matched by selector analysis.
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
    /// Terraform/OpenTofu resource reference.
    TerraformReference,
    /// Terraform/OpenTofu local module block reference.
    TerraformModuleRef,
    /// Terraform/OpenTofu output consumption reference.
    TerraformOutputRef,
    /// Workflow file → virtual job node.
    WorkflowJob,
    /// Virtual workflow job → virtual workflow step node.
    WorkflowStep,
    /// A `needs:` dependency between virtual workflow jobs.
    WorkflowNeeds,
    /// A local reusable workflow or action invoked by a job or step.
    WorkflowUses,
    /// A workflow step invokes a tracked code or package-script target.
    WorkflowRun,
    /// An upload-artifact step produces an artifact consumed by a download step.
    WorkflowArtifact,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        self.as_core_str().unwrap_or_else(|| self.as_domain_str())
    }

    fn as_core_str(&self) -> Option<&'static str> {
        match self {
            Self::Import => Some("import"),
            Self::TypeImport => Some("type-import"),
            Self::DynamicImport => Some("dynamic-import"),
            Self::RouteImport => Some("route-import"),
            Self::Require => Some("require"),
            Self::TestOf => Some("test"),
            Self::RouteRef => Some("route"),
            Self::QueueEnqueue => Some("queue-enqueue"),
            Self::QueueWorker => Some("queue-worker"),
            Self::RouteTest => Some("route-test"),
            Self::Layout => Some("layout"),
            Self::MarkdownLink => Some("md"),
            Self::WorkspaceImport => Some("workspace"),
            Self::PackageDependency => Some("package"),
            Self::CiInvocation => Some("ci"),
            Self::WorkflowJob => Some("workflow-job"),
            Self::WorkflowStep => Some("workflow-step"),
            Self::WorkflowNeeds => Some("workflow-needs"),
            Self::WorkflowUses => Some("workflow-uses"),
            Self::WorkflowRun => Some("workflow-run"),
            Self::WorkflowArtifact => Some("workflow-artifact"),
            _ => None,
        }
    }

    fn as_domain_str(&self) -> &'static str {
        match self {
            Self::HttpCall => "http",
            Self::ProcessSpawn => "process",
            Self::AssetImport => "asset",
            Self::Resource => "resource",
            Self::ReactRender => "react-render",
            Self::Selector => "selector",
            Self::SwiftImport => "swift-import",
            Self::SwiftReference => "swift-ref",
            Self::SwiftPackageDependency => "swift-package",
            Self::DotnetUsing => "dotnet-using",
            Self::DotnetReference => "dotnet-ref",
            Self::DotnetProjectDependency => "dotnet-project",
            Self::TerraformReference => "terraform-ref",
            Self::TerraformModuleRef => "terraform-module",
            Self::TerraformOutputRef => "terraform-output",
            _ => unreachable!("core edge kinds are handled before domain rendering"),
        }
    }
}

/// Stable ordering for deterministic graph output.
/// Keep variants append-only so their discriminants preserve this public order.
pub(crate) const fn edge_kind_rank(kind: EdgeKind) -> u8 {
    kind as u8
}
