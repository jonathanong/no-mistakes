use super::is_direct_owner_edge;
use no_mistakes::codebase::dependencies::graph::{EdgeKind, VitestSetupField};

#[test]
fn direct_owner_edges_are_import_family_and_test_of() {
    let direct = [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::RequireResolve,
        EdgeKind::WorkspaceImport,
        EdgeKind::WorkspaceTypeImport,
        EdgeKind::TestOf,
    ];
    let indirect = [
        EdgeKind::RouteImport,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::MarkdownLink,
        EdgeKind::PackageDependency,
        EdgeKind::CiInvocation,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
        EdgeKind::AssetImport,
        EdgeKind::Resource,
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
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup),
    ];
    for kind in direct {
        assert!(is_direct_owner_edge(kind), "{kind:?} should be direct");
    }
    for kind in indirect {
        assert!(
            !is_direct_owner_edge(kind),
            "{kind:?} should stay in dependencies"
        );
    }
}
