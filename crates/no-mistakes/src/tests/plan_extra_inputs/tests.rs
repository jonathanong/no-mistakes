use super::*;
use no_mistakes::codebase::dependencies::graph::VitestSetupField;

#[test]
fn deleted_transitive_details_align_with_synthetic_and_vitest_edges() {
    let edge_path = [
        EdgeKind::Import,
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
    ];
    let via = std::iter::once("deleted-dependency")
        .chain(edge_path.iter().map(|kind| impact_reason_label(*kind)))
        .collect::<Vec<_>>();
    let via_details = deleted_transitive_via_details(&edge_path);

    assert_eq!(via.len(), via_details.len());
    assert_eq!(via_details[0], None, "synthetic edge has no detail");
    assert_eq!(via_details[1], None, "import has no detail");
    assert_eq!(
        via_details[2],
        Some(ImpactEdgeDetail::VitestSetup {
            field: "setupFiles".to_string(),
        })
    );
}

#[test]
fn impact_reason_labels_preserve_public_spellings() {
    // Keep every edge's prior reason label, including canonical graph spellings.
    let expected: std::collections::BTreeMap<String, String> = serde_json::from_str(include_str!(
        "../../../../../fixtures/test-plan/impact-reason-labels.json"
    ))
    .unwrap();
    let edges = [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::ConditionalDynamicImport,
        EdgeKind::RouteImport,
        EdgeKind::Require,
        EdgeKind::RequireResolve,
        EdgeKind::WorkspaceImport,
        EdgeKind::WorkspaceTypeImport,
        EdgeKind::PackageDependency,
        EdgeKind::RouteRef,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::TestOf,
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup),
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::MarkdownLink,
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
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
        EdgeKind::GoImport,
        EdgeKind::GoReference,
        EdgeKind::RustUse,
        EdgeKind::RustMod,
        EdgeKind::RustPackage,
        EdgeKind::RubyRequire,
        EdgeKind::RubyReference,
        EdgeKind::PhpUse,
        EdgeKind::PhpPackage,
        EdgeKind::JavaImport,
        EdgeKind::JavaReference,
        EdgeKind::KotlinImport,
        EdgeKind::KotlinReference,
        EdgeKind::ElixirImport,
        EdgeKind::ElixirReference,
        EdgeKind::DartImport,
        EdgeKind::DartReference,
        EdgeKind::TrpcCall,
        EdgeKind::TrpcProcedure,
        EdgeKind::Call,
        EdgeKind::Extends,
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
    ];
    let actual = edges
        .into_iter()
        .map(|edge| (format!("{edge:?}"), impact_reason_label(edge).to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(actual, expected);
}
