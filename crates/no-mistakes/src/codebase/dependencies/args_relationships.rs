pub enum Direction {
    Deps,
    Dependents,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Deserialize, serde::Serialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipArg {
    Import,
    ImportStatic,
    ImportDynamic,
    ImportType,
    ImportRequire,
    RouteImport,
    Workspace,
    Package,
    Test,
    Route,
    Queue,
    Md,
    Ci,
    Http,
    Process,
    Asset,
    React,
    Dotnet,
    Swift,
    Terraform,
    All,
}

impl RelationshipArg {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipArg::Import => "import",
            RelationshipArg::ImportStatic => "import-static",
            RelationshipArg::ImportDynamic => "import-dynamic",
            RelationshipArg::ImportType => "import-type",
            RelationshipArg::ImportRequire => "import-require",
            RelationshipArg::RouteImport => "route-import",
            RelationshipArg::Workspace => "workspace",
            RelationshipArg::Package => "package",
            RelationshipArg::Test => "test",
            RelationshipArg::Route => "route",
            RelationshipArg::Queue => "queue",
            RelationshipArg::Md => "md",
            RelationshipArg::Ci => "ci",
            RelationshipArg::Http => "http",
            RelationshipArg::Process => "process",
            RelationshipArg::Asset => "asset",
            RelationshipArg::React => "react",
            RelationshipArg::Dotnet => "dotnet",
            RelationshipArg::Swift => "swift",
            RelationshipArg::Terraform => "terraform",
            RelationshipArg::All => "all",
        }
    }
}

/// Convert `--relationship` values into a `HashSet<EdgeKind>` filter.
/// Empty input and `all` expand to the standard public edge set; the
/// conservative `route-import` relationship remains explicit opt-in.
#[inline(never)]
pub(crate) fn relationship_filter(
    relationships: &[RelationshipArg],
) -> Option<std::collections::HashSet<EdgeKind>> {
    if relationships.is_empty() {
        return Some(standard_relationship_edges());
    }
    let mut set = std::collections::HashSet::new();
    for r in relationships {
        let edges: &[EdgeKind] = match r {
            RelationshipArg::Import => &[
                EdgeKind::Import,
                EdgeKind::TypeImport,
                EdgeKind::DynamicImport,
                EdgeKind::Require,
            ],
            RelationshipArg::ImportStatic => &[EdgeKind::Import],
            RelationshipArg::ImportDynamic => &[EdgeKind::DynamicImport],
            RelationshipArg::ImportType => &[EdgeKind::TypeImport],
            RelationshipArg::ImportRequire => &[EdgeKind::Require],
            RelationshipArg::RouteImport => &[EdgeKind::RouteImport],
            RelationshipArg::Workspace => &[EdgeKind::WorkspaceImport],
            RelationshipArg::Package => &[EdgeKind::PackageDependency],
            // Selector edges connect tests to covered app components.
            RelationshipArg::Test => &[
                EdgeKind::TestOf,
                EdgeKind::RouteTest,
                EdgeKind::Layout,
                EdgeKind::Selector,
            ],
            RelationshipArg::Route => {
                &[EdgeKind::RouteRef, EdgeKind::RouteTest, EdgeKind::Layout]
            }
            RelationshipArg::Queue => &[EdgeKind::QueueEnqueue, EdgeKind::QueueWorker],
            RelationshipArg::Md => &[EdgeKind::MarkdownLink],
            RelationshipArg::Ci => &[EdgeKind::CiInvocation],
            RelationshipArg::Http => &[EdgeKind::HttpCall],
            RelationshipArg::Process => &[EdgeKind::ProcessSpawn],
            RelationshipArg::Asset => &[EdgeKind::AssetImport],
            RelationshipArg::React => &[EdgeKind::ReactRender],
            RelationshipArg::Dotnet => &[
                EdgeKind::DotnetUsing,
                EdgeKind::DotnetReference,
                EdgeKind::DotnetProjectDependency,
            ],
            RelationshipArg::Swift => &[
                EdgeKind::SwiftImport,
                EdgeKind::SwiftReference,
                EdgeKind::SwiftPackageDependency,
            ],
            RelationshipArg::Terraform => &[
                EdgeKind::TerraformReference,
                EdgeKind::TerraformModuleRef,
                EdgeKind::TerraformOutputRef,
            ],
            RelationshipArg::All => {
                set.extend(standard_relationship_edges());
                &[]
            }
        };
        for edge in edges {
            set.insert(*edge);
        }
    }
    Some(set)
}

/// Edge kinds included by legacy unfiltered traversal and `--relationship all`.
/// `RouteImport` is intentionally absent: it is a conservative alternate view
/// that must be requested explicitly to avoid weakening ordinary call pruning.
fn standard_relationship_edges() -> std::collections::HashSet<EdgeKind> {
    [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::TestOf,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::MarkdownLink,
        EdgeKind::WorkspaceImport,
        EdgeKind::PackageDependency,
        EdgeKind::CiInvocation,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
        EdgeKind::AssetImport,
        EdgeKind::ReactRender,
        EdgeKind::Selector,
        EdgeKind::SwiftImport,
        EdgeKind::SwiftReference,
        EdgeKind::SwiftPackageDependency,
        EdgeKind::DotnetUsing,
        EdgeKind::DotnetReference,
        EdgeKind::DotnetProjectDependency,
        EdgeKind::TerraformReference,
        EdgeKind::TerraformModuleRef,
        EdgeKind::TerraformOutputRef,
    ]
    .into_iter()
    .collect()
}

fn relationships_are_import_only(relationships: &[RelationshipArg]) -> bool {
    !relationships.is_empty()
        && relationships.iter().all(|relationship| {
            matches!(
                relationship,
                RelationshipArg::Import
                    | RelationshipArg::ImportStatic
                    | RelationshipArg::ImportDynamic
                    | RelationshipArg::ImportType
                    | RelationshipArg::ImportRequire
            )
        })
}

/// A resolved entrypoint: a file/module node, plus an optional exported symbol / queue job name.
struct Entrypoint {
    file: PathBuf,
    node: NodeId,
    symbol: Option<String>,
}

pub fn parse_entrypoint(s: &str) -> (PathBuf, Option<String>) {
    match s.split_once('#') {
        Some((file, symbol)) => (PathBuf::from(file), Some(symbol.to_string())),
        None => (PathBuf::from(s), None),
    }
}
