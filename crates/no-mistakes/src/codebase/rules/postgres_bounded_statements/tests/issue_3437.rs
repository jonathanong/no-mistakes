use super::super::evaluate::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3437");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema.json"),
    ));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn offender_tables(sql: &str, catalog: &SchemaCatalog) -> Vec<Vec<String>> {
    extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .map(|fact| {
            let mut names: Vec<_> = offenders(fact, catalog)
                .into_iter()
                .map(|offender| offender.table)
                .collect();
            names.sort();
            names.dedup();
            names
        })
        .collect()
}

#[test]
fn issue_3437_composite_foreign_key_keeps_its_negative_controls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3437");
    let sql = std::fs::read_to_string(root.join("composite-fk.sql")).unwrap();
    let catalog = catalog();
    assert_eq!(
        offender_tables(&sql, &catalog),
        vec![
            vec![],
            vec!["nullable_operations".to_string()],
            vec!["loose_operations".to_string(), "tag_pairs".to_string()],
            vec![
                "lineages".to_string(),
                "memberships".to_string(),
                "operations".to_string(),
                "sources".to_string(),
            ],
        ]
    );
}
