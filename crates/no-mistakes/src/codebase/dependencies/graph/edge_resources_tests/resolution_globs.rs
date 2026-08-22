use super::*;
use crate::codebase::ts_resources::{
    ResourceCall, ResourceCallKind, ResourcePath, ResourcePathBase,
};

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
    assert!(edges
        .iter()
        .any(|(_, target, _)| target.as_file() == Some(page.as_path())));

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
