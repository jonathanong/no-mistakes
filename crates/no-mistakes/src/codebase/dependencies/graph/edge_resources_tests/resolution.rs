use super::*;
use crate::codebase::ts_resources::{
    ResourceCall, ResourceCallKind, ResourcePath, ResourcePathBase,
};

#[test]
fn resource_edges_resolve_exact_directory_and_glob_with_sorted_provenance() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("consumer.ts");
    let page = root.join("resources/page.txt");
    let button = root.join("resources/button.txt");
    let calls = vec![
        ResourceCall {
            kind: ResourceCallKind::Glob,
            path: ResourcePath {
                value: "resources/*.txt".to_string(),
                base: ResourcePathBase::AnalysisRoot,
            },
            cwd: None,
            line: 7,
            function_scope: None,
        },
        ResourceCall {
            kind: ResourceCallKind::ReadDirectorySync,
            path: ResourcePath {
                value: "resources".to_string(),
                base: ResourcePathBase::AnalysisRoot,
            },
            cwd: None,
            line: 3,
            function_scope: None,
        },
        ResourceCall {
            kind: ResourceCallKind::ReadFile,
            path: ResourcePath {
                value: "resources/page.txt".to_string(),
                base: ResourcePathBase::AnalysisRoot,
            },
            cwd: None,
            line: 1,
            function_scope: None,
        },
    ];
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: calls,
            ..TsFileFacts::default()
        },
    )]);
    let (edges, details, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        &[consumer.clone(), page.clone(), button.clone()],
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        edges,
        vec![
            (
                NodeId::File(consumer.clone()),
                NodeId::File(button.clone()),
                EdgeKind::Resource,
            ),
            (
                NodeId::File(consumer.clone()),
                NodeId::File(page.clone()),
                EdgeKind::Resource,
            ),
        ]
    );
    assert_eq!(
        details.get(&(consumer, page)).unwrap(),
        &[
            ResourceCallSite {
                call_kind: "read-file".to_string(),
                line: 1,
            },
            ResourceCallSite {
                call_kind: "read-directory-sync".to_string(),
                line: 3,
            },
            ResourceCallSite {
                call_kind: "glob".to_string(),
                line: 7,
            },
        ]
    );
}

#[test]
fn resource_edges_exclude_untracked_candidates_and_unreachable_scopes() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("consumer.ts");
    let page = root.join("resources/page.txt");
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: vec![ResourceCall {
                kind: ResourceCallKind::ReadFile,
                path: ResourcePath {
                    value: "resources/page.txt".to_string(),
                    base: ResourcePathBase::AnalysisRoot,
                },
                cwd: None,
                line: 4,
                // No static call reaches this private helper.
                function_scope: Some("neverCalled".to_string()),
            }],
            ..TsFileFacts::default()
        },
    )]);
    let (edges, details, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        &[consumer.clone(), page],
    );
    assert!(edges.is_empty());
    assert!(details.is_empty());
    assert!(diagnostics.is_empty());
}
