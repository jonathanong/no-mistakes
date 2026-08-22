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
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        edges,
        vec![
            (
                NodeId::file(consumer.clone()),
                NodeId::file(button.clone()),
                EdgeKind::Resource,
            ),
            (
                NodeId::file(consumer.clone()),
                NodeId::file(page.clone()),
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
        &crate::codebase::analysis_session::PathInterner::new(),
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
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        edges,
        vec![(
            NodeId::file(consumer.clone()),
            NodeId::file(page.clone()),
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

#[test]
fn resource_edges_cover_source_module_globs_invalid_patterns_and_missing_exact_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("consumer.ts");
    let page = root.join("resources/page.txt");
    let facts = TsFactMap::from([(
        consumer.clone(),
        TsFileFacts {
            resource_calls: vec![
                ResourceCall {
                    kind: ResourceCallKind::Glob,
                    path: ResourcePath {
                        value: "././resources/*.txt".to_string(),
                        base: ResourcePathBase::AnalysisRoot,
                    },
                    cwd: None,
                    line: 1,
                    function_scope: None,
                },
                ResourceCall {
                    kind: ResourceCallKind::GlobSync,
                    path: ResourcePath {
                        value: "resources/*.txt".to_string(),
                        base: ResourcePathBase::SourceModule,
                    },
                    cwd: None,
                    line: 2,
                    function_scope: None,
                },
                ResourceCall {
                    kind: ResourceCallKind::Glob,
                    path: ResourcePath {
                        value: "[".to_string(),
                        base: ResourcePathBase::AnalysisRoot,
                    },
                    cwd: None,
                    line: 3,
                    function_scope: None,
                },
                ResourceCall {
                    kind: ResourceCallKind::ReadFileSync,
                    path: ResourcePath {
                        value: "missing.txt".to_string(),
                        base: ResourcePathBase::AnalysisRoot,
                    },
                    cwd: None,
                    line: 4,
                    function_scope: None,
                },
            ],
            ..TsFileFacts::default()
        },
    )]);
    let (edges, _, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        &[consumer.clone(), page.clone()],
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(diagnostics.is_empty());
    assert!(
        edges
            .iter()
            .any(|(_, target, _)| target.as_file() == Some(page.as_path()))
    );

    let missing_root = PathBuf::from("/no-mistakes-missing-resource-root");
    let (edges, _, _) = collect_resource_edges(
        &missing_root,
        std::slice::from_ref(&consumer),
        &facts,
        std::slice::from_ref(&page),
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.is_empty());
}
