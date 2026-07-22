use super::*;
use crate::codebase::ts_resources::{
    ResourceCall, ResourceCallKind, ResourceDiagnostic, ResourceDiagnosticKind, ResourcePath,
    ResourcePathBase,
};

#[test]
fn resource_edges_keep_dynamic_diagnostics_but_never_invent_untracked_targets() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("consumer.ts");
    let untracked = root.join("resources/not-tracked.txt");
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: vec![ResourceCall {
                kind: ResourceCallKind::ReadFile,
                path: ResourcePath {
                    value: "resources/not-tracked.txt".to_string(),
                    base: ResourcePathBase::AnalysisRoot,
                },
                cwd: None,
                line: 2,
                function_scope: None,
            }],
            resource_diagnostics: vec![ResourceDiagnostic {
                kind: ResourceDiagnosticKind::DynamicPath,
                line: 9,
                function_scope: None,
            }],
            ..TsFileFacts::default()
        },
    )]);
    let (edges, details, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        // A visible source file is not enough to make the literal target
        // tracked: only this prepared inventory participates in resource edges.
        std::slice::from_ref(&consumer),
    );
    assert!(
        !untracked.exists(),
        "fixture must retain this as an absent target"
    );
    assert!(edges.is_empty());
    assert!(details.is_empty());
    assert_eq!(
        diagnostics,
        vec![ResourceGraphDiagnostic {
            consumer,
            kind: ResourceDiagnosticKind::DynamicPath,
            line: 9,
        }]
    );
}

#[test]
fn module_relative_resource_paths_resolve_against_the_consumer_directory() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("nested/consumer.ts");
    let page = root.join("resources/page.txt");
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: vec![ResourceCall {
                kind: ResourceCallKind::ReadFileSync,
                path: ResourcePath {
                    value: "../resources/page.txt".to_string(),
                    base: ResourcePathBase::SourceModule,
                },
                cwd: None,
                line: 5,
                function_scope: None,
            }],
            ..TsFileFacts::default()
        },
    )]);
    let (edges, details, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        std::slice::from_ref(&page),
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        edges,
        vec![(
            NodeId::File(consumer.clone()),
            NodeId::File(page.clone()),
            EdgeKind::Resource,
        )]
    );
    assert_eq!(
        details.get(&(consumer, page)).unwrap(),
        &[ResourceCallSite {
            call_kind: "read-file-sync".to_string(),
            line: 5,
        }]
    );
}
