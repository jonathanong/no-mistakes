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
    let empty_visible = HashSet::new();

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

    let visible = HashSet::from([page.clone()]);
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
        &HashSet::new(),
    )
    .unwrap();
    assert!(hidden.is_empty());
}
