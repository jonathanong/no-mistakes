use super::*;
use crate::codebase::ts_resources::{
    ResourceCall, ResourceCallKind, ResourcePath, ResourcePathBase,
};

#[test]
fn graph_files_keep_tracked_resources_from_source_skipped_directories() {
    let source = fixture("resource-impact");
    let materialized = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(materialized.path());
    let skipped_resource = root.join("fixtures/schema.sql");
    let files = GraphFiles::discover(&root);

    assert!(
        !files.all().contains(&skipped_resource),
        "source discovery must not parse files below fixtures/"
    );
    assert!(
        files.resource_candidates().contains(&skipped_resource),
        "tracked runtime files below fixtures/ remain valid resource targets"
    );
}

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

#[test]
fn resource_edges_resolve_absolute_glob_patterns_inside_the_root() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("glob-consumer.ts");
    let page = root.join("resources/page.txt");
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: vec![ResourceCall {
                kind: ResourceCallKind::GlobSync,
                path: ResourcePath {
                    value: root.join("resources/*.txt").to_string_lossy().to_string(),
                    base: ResourcePathBase::AnalysisRoot,
                },
                cwd: None,
                line: 1,
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
            call_kind: "glob-sync".to_string(),
            line: 1,
        }]
    );
}
