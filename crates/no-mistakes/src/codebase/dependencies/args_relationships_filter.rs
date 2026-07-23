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
            RelationshipArg::Workflow => workflow_edges(),
            RelationshipArg::WorkflowJob => &[EdgeKind::WorkflowJob],
            RelationshipArg::WorkflowStep => &[EdgeKind::WorkflowJob, EdgeKind::WorkflowStep],
            RelationshipArg::WorkflowNeeds => &[EdgeKind::WorkflowJob, EdgeKind::WorkflowNeeds],
            RelationshipArg::WorkflowUses => &[
                EdgeKind::WorkflowJob,
                EdgeKind::WorkflowStep,
                EdgeKind::WorkflowUses,
            ],
            RelationshipArg::WorkflowRun => &[
                EdgeKind::WorkflowJob,
                EdgeKind::WorkflowStep,
                EdgeKind::WorkflowRun,
            ],
            RelationshipArg::WorkflowArtifact => &[
                EdgeKind::WorkflowJob,
                EdgeKind::WorkflowStep,
                EdgeKind::WorkflowArtifact,
            ],
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
            RelationshipArg::Resource => &[EdgeKind::Resource],
            RelationshipArg::All => {
                set.extend(standard_relationship_edges());
                &[]
            }
        };
        set.extend(edges.iter().copied());
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
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
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
        EdgeKind::Resource,
    ]
    .into_iter()
    .collect()
}

const fn workflow_edges() -> &'static [EdgeKind] {
    &[
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
    ]
}
