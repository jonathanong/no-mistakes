#[test]
fn edge_kind_str_all_variants() {
    assert_eq!(EdgeKind::Import.as_str(), "import");
    assert_eq!(EdgeKind::TypeImport.as_str(), "type-import");
    assert_eq!(EdgeKind::DynamicImport.as_str(), "dynamic-import");
    assert_eq!(EdgeKind::Require.as_str(), "require");
    assert_eq!(EdgeKind::TestOf.as_str(), "test");
    assert_eq!(
        EdgeKind::VitestSetup(
            crate::codebase::dependencies::graph::VitestSetupField::SetupFiles,
        )
        .as_str(),
        "vitest-setup"
    );
    assert_eq!(EdgeKind::RouteRef.as_str(), "route");
    assert_eq!(EdgeKind::QueueEnqueue.as_str(), "queue-enqueue");
    assert_eq!(EdgeKind::QueueWorker.as_str(), "queue-worker");
    assert_eq!(EdgeKind::RouteTest.as_str(), "route-test");
    assert_eq!(EdgeKind::Layout.as_str(), "layout");
    assert_eq!(EdgeKind::MarkdownLink.as_str(), "md");
    assert_eq!(EdgeKind::WorkspaceImport.as_str(), "workspace");
    assert_eq!(EdgeKind::PackageDependency.as_str(), "package");
    assert_eq!(EdgeKind::CiInvocation.as_str(), "ci");
    assert_eq!(EdgeKind::HttpCall.as_str(), "http");
    assert_eq!(EdgeKind::ProcessSpawn.as_str(), "process");
    assert_eq!(EdgeKind::AssetImport.as_str(), "asset");
    assert_eq!(EdgeKind::ReactRender.as_str(), "react-render");
    assert_eq!(EdgeKind::Selector.as_str(), "selector");
    assert_eq!(EdgeKind::SwiftImport.as_str(), "swift-import");
    assert_eq!(EdgeKind::SwiftReference.as_str(), "swift-ref");
    assert_eq!(EdgeKind::SwiftPackageDependency.as_str(), "swift-package");
}

#[test]
fn serialized_edge_kinds_are_documented() {
    let docs = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/graph-edges.md"),
    )
    .unwrap();
    for kind in [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::RouteImport,
        EdgeKind::Require,
        EdgeKind::TestOf,
        EdgeKind::VitestSetup(
            crate::codebase::dependencies::graph::VitestSetupField::SetupFiles,
        ),
        EdgeKind::VitestSetup(
            crate::codebase::dependencies::graph::VitestSetupField::GlobalSetup,
        ),
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
        EdgeKind::Resource,
        EdgeKind::ReactRender,
        EdgeKind::Selector,
        EdgeKind::DotnetUsing,
        EdgeKind::DotnetReference,
        EdgeKind::DotnetProjectDependency,
        EdgeKind::SwiftImport,
        EdgeKind::SwiftReference,
        EdgeKind::SwiftPackageDependency,
        EdgeKind::TerraformReference,
        EdgeKind::TerraformModuleRef,
        EdgeKind::TerraformOutputRef,
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
    ] {
        match kind {
            EdgeKind::Import => {}
            EdgeKind::TypeImport => {}
            EdgeKind::DynamicImport => {}
            EdgeKind::RouteImport => {}
            EdgeKind::Require => {}
            EdgeKind::TestOf => {}
            EdgeKind::VitestSetup(_) => {}
            EdgeKind::RouteRef => {}
            EdgeKind::QueueEnqueue => {}
            EdgeKind::QueueWorker => {}
            EdgeKind::RouteTest => {}
            EdgeKind::Layout => {}
            EdgeKind::MarkdownLink => {}
            EdgeKind::WorkspaceImport => {}
            EdgeKind::PackageDependency => {}
            EdgeKind::CiInvocation => {}
            EdgeKind::HttpCall => {}
            EdgeKind::ProcessSpawn => {}
            EdgeKind::AssetImport => {}
            EdgeKind::Resource => {}
            EdgeKind::ReactRender => {}
            EdgeKind::Selector => {}
            EdgeKind::DotnetUsing => {}
            EdgeKind::DotnetReference => {}
            EdgeKind::DotnetProjectDependency => {}
            EdgeKind::SwiftImport => {}
            EdgeKind::SwiftReference => {}
            EdgeKind::SwiftPackageDependency => {}
            EdgeKind::TerraformReference => {}
            EdgeKind::TerraformModuleRef => {}
            EdgeKind::TerraformOutputRef => {}
            EdgeKind::WorkflowJob => {}
            EdgeKind::WorkflowStep => {}
            EdgeKind::WorkflowNeeds => {}
            EdgeKind::WorkflowUses => {}
            EdgeKind::WorkflowRun => {}
            EdgeKind::WorkflowArtifact => {}
        }
        let serialized = kind.as_str();
        assert!(
            docs.contains(&format!("`{serialized}`")),
            "docs/graph-edges.md must document `{serialized}`"
        );
    }
}
