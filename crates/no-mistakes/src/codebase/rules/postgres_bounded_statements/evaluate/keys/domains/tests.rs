use super::super::super::{evaluate, ViewBounds};
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog, SqlBoundItemKind};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture")
}

fn catalog() -> SchemaCatalog {
    let root = fixture();
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn recursive_finite_values_bound_keys_without_claiming_finite_recursive_rows() {
    let sql = std::fs::read_to_string(fixture().join("r3-domain-basic.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    let catalog = catalog();
    for (fact, expected) in facts.bounds.iter().zip([true, true, false, true, false]) {
        let found = evaluate(&fact.query, &catalog, &ViewBounds::default(), false);
        assert_eq!(found.items[0], expected);
    }
    for index in [0, 3] {
        let SqlBoundItemKind::Query(recursive) = &facts.bounds[index].query.items[1].kind else {
            panic!("expected recursive source");
        };
        let found = evaluate(recursive, &catalog, &ViewBounds::default(), false);
        assert!(!found.bounded);
        assert!(found
            .finite_outputs
            .as_ref()
            .is_some_and(|outputs| outputs[0]));
    }
}

#[test]
fn recursive_domain_guards_keep_unbounded_donors_and_transforms_unproved() {
    let sql = std::fs::read_to_string(fixture().join("r3-domain-guards.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 8);
    let catalog = catalog();
    for (fact, expected) in facts.bounds[..7]
        .iter()
        .zip([false, false, false, false, true, true, false])
    {
        let found = evaluate(&fact.query, &catalog, &ViewBounds::default(), false);
        assert_eq!(found.items[0], expected, "line {}", fact.line);
    }
    let SqlBoundItemKind::Query(recursive) = &facts.bounds[1].query.items[1].kind else {
        panic!("expected recursive source");
    };
    assert!(recursive.outputs[0].recursive_domain);
    let capped_anchor = evaluate(recursive, &catalog, &ViewBounds::default(), false);
    assert!(capped_anchor.items[0]);
    assert!(capped_anchor.finite_outputs.is_none());
    let selected = evaluate(
        &facts.bounds[7].query,
        &catalog,
        &ViewBounds::default(),
        false,
    );
    assert!(!selected.bounded);
}
