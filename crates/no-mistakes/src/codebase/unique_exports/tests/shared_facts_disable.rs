use super::*;

#[test]
fn source_files_from_facts_preserves_disabled_parse_errors() {
    let root = fixture("unique-exports-edge-cases");
    let file = root.join("src/disabled-invalid.ts");
    let files = vec![file];
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let error =
        match scan::test_support::collect_source_files_from_facts(&root, &files, &facts, false) {
            Err(error) => error,
            Ok(_) => panic!("suppression cannot make an unparseable source complete"),
        };
    assert!(error.to_string().contains("disabled-invalid.ts"));
}
