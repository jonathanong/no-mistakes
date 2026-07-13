use super::*;
use crate::fetch::types::SourceType;
use std::collections::HashMap;

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
