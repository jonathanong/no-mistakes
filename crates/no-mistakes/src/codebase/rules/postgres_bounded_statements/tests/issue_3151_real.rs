use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn catalog_function_body_and_saved_union_sql_have_the_same_bounded_inputs() {
    let root = super::fixture_root().join("../issue-3151-real/fixture");
    let files = [root.join("schema.json"), root.join("raw-catalog.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let raw = sources.read_path(&files[1]).unwrap();
    let function = catalog
        .functions()
        .find(|function| function.key == "public.notice_target_states(integer, integer[])")
        .expect("saved catalog function");
    assert_eq!(function.returns_set, Some(true));
    let body = function.body.as_deref().expect("parsed SQL function body");
    assert_eq!(body.trim(), raw.trim());
    for (name, sql) in [("raw SQL", raw.as_ref()), ("catalog function body", body)] {
        let facts = extract_sql_statement_facts(sql);
        assert!(!facts.parse_failed, "{name}");
        assert!(!facts.bounds.is_empty(), "{name}: missing query facts");
        for fact in &facts.bounds {
            assert!(
                super::super::evaluate::offenders(fact, &catalog).is_empty(),
                "{name}: unexpected unbounded input"
            );
        }
    }
}

#[test]
fn lateral_projection_does_not_bound_an_independent_cte_driver() {
    let root = super::fixture_root().join("../issue-3151-real/fixture");
    let files = [root.join("schema.json"), root.join("cte-driver.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let sql = sources.read_path(&files[1]).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert!(!facts.bounds.is_empty());
    assert!(facts.bounds.iter().any(|fact| {
        super::super::evaluate::offenders(fact, &catalog)
            .iter()
            .any(|offender| catalog.same_relation(&offender.table, "notice_targets"))
    }));
}
