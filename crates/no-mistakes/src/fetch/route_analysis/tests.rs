use super::*;
use crate::fetch::file_analysis::{analyze_file_from_visible_with_facts, VisibleFileAnalysis};
use crate::fetch::file_facts::ParsedFileCache;
use crate::fetch::types::SourceType;
use std::collections::{HashMap, HashSet};

#[test]
fn legacy_route_collection_analyzes_page_and_checks_parent_layout_candidates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-coverage/with-fetches/fixture");
    let route = Route {
        file: root.join("app/page.tsx"),
        pattern: "/".to_string(),
    };
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };

    let fetches = collect_route_fetches(&route, &root.join("app"), &root, &mut cache)
        .expect("saved page fixture should analyze");

    assert!(
        fetches.iter().any(|fetch| {
            fetch.path == "/api/health"
                && fetch.file.ends_with("/app/page.tsx")
                && fetch.source_type == SourceType::Page
        }),
        "{fetches:#?}"
    );
}

#[test]
fn visible_fetch_analysis_skips_unlisted_and_already_visited_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-coverage/with-fetches/fixture");
    let page = crate::codebase::ts_resolver::normalize_path(&root.join("app/page.tsx"));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut parsed_files = ParsedFileCache::default();
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let empty_visible = crate::fx::PathSet::default();

    let skipped = analyze_file_from_visible_with_facts(
        &page,
        (false, false),
        &mut VisibleFileAnalysis {
            session: &session,
            root: &root,
            visited: &mut visited,
            fetches: &mut fetches,
            cache: &mut cache,
            parsed_files: &mut parsed_files,
            visible_files: &empty_visible,
        },
    )
    .unwrap();
    assert!(!skipped);
    assert!(fetches.is_empty());

    let visible: crate::fx::PathSet = [page.clone()].into_iter().collect();
    let first = analyze_file_from_visible_with_facts(
        &page,
        (false, false),
        &mut VisibleFileAnalysis {
            session: &session,
            root: &root,
            visited: &mut visited,
            fetches: &mut fetches,
            cache: &mut cache,
            parsed_files: &mut parsed_files,
            visible_files: &visible,
        },
    )
    .unwrap();
    let after_first = fetches.len();
    let second = analyze_file_from_visible_with_facts(
        &page,
        (false, false),
        &mut VisibleFileAnalysis {
            session: &session,
            root: &root,
            visited: &mut visited,
            fetches: &mut fetches,
            cache: &mut cache,
            parsed_files: &mut parsed_files,
            visible_files: &visible,
        },
    )
    .unwrap();
    assert!(!second);
    assert_eq!(fetches.len(), after_first);
    let _ = first;

    let hidden = collect_route_fetches_from_visible(
        &Route {
            file: page,
            pattern: "/".to_string(),
        },
        &root.join("app"),
        &root,
        &mut Cache {
            files: HashMap::new(),
            imports: HashMap::new(),
        },
        &crate::fx::PathSet::default(),
    )
    .unwrap();
    assert!(hidden.is_empty());
}

#[test]
fn visible_fetch_analysis_propagates_imported_fact_load_errors() {
    // A prepared parent can list a visible import whose facts failed to load.
    // The recursive `?` must surface that cached error instead of skipping it.
    let root = PathBuf::from("/repo");
    let parent = crate::codebase::ts_resolver::normalize_path(&root.join("page.tsx"));
    let child = crate::codebase::ts_resolver::normalize_path(&root.join("child.ts"));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut parsed_files = ParsedFileCache::default();
    parsed_files.insert(
        parent.clone(),
        crate::fetch::file_facts::ParsedFileFacts {
            has_use_client_directive: false,
            has_use_server_directive: false,
            fetches: Vec::new(),
            imports: vec![child.clone()],
            used_imports: vec![child.clone()],
        },
    );
    parsed_files.insert_error(child.clone(), "child source unreadable".to_string());
    let visible: crate::fx::PathSet = [parent.clone(), child].into_iter().collect();
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();

    let error = analyze_file_from_visible_with_facts(
        &parent,
        (false, false),
        &mut VisibleFileAnalysis {
            session: &session,
            root: &root,
            visited: &mut visited,
            fetches: &mut fetches,
            cache: &mut cache,
            parsed_files: &mut parsed_files,
            visible_files: &visible,
        },
    )
    .expect_err("imported fact load errors must surface");
    assert!(
        error.to_string().contains("child source unreadable"),
        "{error}"
    );
}
