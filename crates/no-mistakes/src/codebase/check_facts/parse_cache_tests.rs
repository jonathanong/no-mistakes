use super::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use std::path::PathBuf;

#[test]
fn check_fact_collection_parses_each_path_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/shared-facts/fixture"),
    );
    let helper = normalize_path(&root.join("src/helper.ts"));
    let widget = normalize_path(&root.join("src/widget.tsx"));
    crate::ast::begin_parse_count(&root);
    crate::ast::with_request_parse_cache(|| {
        let facts = collect_check_facts(
            &root,
            vec![helper.clone(), widget.clone()],
            CheckFactPlan {
                imports: true,
                ..CheckFactPlan::default()
            },
        );
        assert!(facts.ts.get(&helper).is_some());
        assert!(facts.ts.get(&widget).is_some());
        assert_eq!(
            crate::ast::request_parse_cache_len(),
            0,
            "parallel check fact collection evicts per-file parse cache entries"
        );
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&helper), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&widget), Some(&1), "{counts:#?}");
}
