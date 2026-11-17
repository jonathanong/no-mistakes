#[test]
fn only_closed_private_frozen_candidates_trigger_reference_walks() {
    let path = crate::test_support::rule_fixture_root("postgres-closed-identifiers")
        .with_file_name("frozen-followups")
        .join("tests/candidate-filtering.test.ts");
    let source = std::fs::read_to_string(&path).expect("saved fixture");
    crate::ast::with_program(&path, &source, |module, _| {
        let mut checked = Vec::new();
        let records = super::module_records_with(module, &source, |module, name| {
            checked.push(name.to_string());
            super::refs::references_allowed(module, name)
        });
        assert_eq!(checked, ["tables"]);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, "tables");
    })
    .expect("parse saved fixture");
}
