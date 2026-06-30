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
/// Returns `None` when "all" is present or the list is empty (= no filter).
#[inline(never)]
pub(crate) fn relationship_filter(
    relationships: &[RelationshipArg],
) -> Option<std::collections::HashSet<EdgeKind>> {
    if relationships.is_empty() {
        return None;
    }
    let mut set = std::collections::HashSet::new();
    for r in relationships {
        match r {
            RelationshipArg::Import => {
                set.insert(EdgeKind::Import);
                set.insert(EdgeKind::TypeImport);
                set.insert(EdgeKind::DynamicImport);
                set.insert(EdgeKind::Require);
            }
            RelationshipArg::ImportStatic => {
                set.insert(EdgeKind::Import);
            }
            RelationshipArg::ImportDynamic => {
                set.insert(EdgeKind::DynamicImport);
            }
            RelationshipArg::ImportType => {
                set.insert(EdgeKind::TypeImport);
            }
            RelationshipArg::ImportRequire => {
                set.insert(EdgeKind::Require);
            }
            RelationshipArg::Workspace => {
                set.insert(EdgeKind::WorkspaceImport);
            }
            RelationshipArg::Package => {
                set.insert(EdgeKind::PackageDependency);
            }
            RelationshipArg::Test => {
                set.insert(EdgeKind::TestOf);
                set.insert(EdgeKind::RouteTest);
                set.insert(EdgeKind::Layout);
                // Selector edges connect test files to the app components they
                // cover via data-pw attributes; include them in test traversals.
                set.insert(EdgeKind::Selector);
            }
            RelationshipArg::Route => {
                set.insert(EdgeKind::RouteRef);
                set.insert(EdgeKind::RouteTest);
                set.insert(EdgeKind::Layout);
            }
            RelationshipArg::Queue => {
                set.insert(EdgeKind::QueueEnqueue);
                set.insert(EdgeKind::QueueWorker);
            }
            RelationshipArg::Md => {
                set.insert(EdgeKind::MarkdownLink);
            }
            RelationshipArg::Ci => {
                set.insert(EdgeKind::CiInvocation);
            }
            RelationshipArg::Http => {
                set.insert(EdgeKind::HttpCall);
            }
            RelationshipArg::Process => {
                set.insert(EdgeKind::ProcessSpawn);
            }
            RelationshipArg::Asset => {
                set.insert(EdgeKind::AssetImport);
            }
            RelationshipArg::React => {
                set.insert(EdgeKind::ReactRender);
            }
            RelationshipArg::Dotnet => {
                set.insert(EdgeKind::DotnetUsing);
                set.insert(EdgeKind::DotnetReference);
                set.insert(EdgeKind::DotnetProjectDependency);
            }
            RelationshipArg::Swift => {
                set.insert(EdgeKind::SwiftImport);
                set.insert(EdgeKind::SwiftReference);
                set.insert(EdgeKind::SwiftPackageDependency);
            }
            RelationshipArg::Terraform => {
                set.insert(EdgeKind::TerraformReference);
                set.insert(EdgeKind::TerraformModuleRef);
                set.insert(EdgeKind::TerraformOutputRef);
            }
            RelationshipArg::All => return None,
        }
    }
    Some(set)
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
