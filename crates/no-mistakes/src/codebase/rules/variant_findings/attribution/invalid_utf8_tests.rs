use std::path::PathBuf;

#[test]
fn invalid_utf8_caller_directive_still_matches() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-conflict-ordering/invalid-utf8");
    let path = root.join("caller.ts");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let dedup = super::super::VariantFindingDedup::for_root(&root, &sources);
    let facts = crate::codebase::postgres::SqlStatementFileFacts {
        path,
        variant_locations: Some(crate::codebase::postgres::statements::SqlVariantLocations {
            original_call_line: 2,
            ..Default::default()
        }),
        ..Default::default()
    };
    let locations = facts.variant_locations.clone().unwrap();
    let hit = dedup.caller_suppression(&facts, &locations, "postgres-conflict-ordering");
    assert_eq!(hit, Some(("caller.ts".to_string(), 2)));
}
