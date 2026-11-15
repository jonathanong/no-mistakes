use super::super::evaluate::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3396");
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

fn sql(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3396")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn issue_3396_shapes_keep_their_negative_controls() {
    let catalog = catalog();
    assert_eq!(
        offender_tables(&sql("limit-join.sql"), &catalog),
        vec![vec![], vec!["flows".to_string()]]
    );
    assert_eq!(
        offender_tables(&sql("group-by.sql"), &catalog),
        vec![
            vec![],
            vec![],
            vec![],
            vec!["entry_signals".to_string()],
            vec!["entry_signals".to_string()],
            vec!["flows".to_string()],
            vec!["flows".to_string()],
            vec!["flows".to_string()],
            vec!["flows".to_string()],
            vec!["entry_signals".to_string(), "flows".to_string()],
            vec!["entry_signals".to_string(), "flows".to_string()],
            vec!["flows".to_string()],
            vec!["entry_signals".to_string(), "flows".to_string()],
            vec![],
            vec!["flows".to_string()],
            vec![],
            vec!["entry_signals".to_string()],
        ]
    );
    assert_eq!(
        offender_tables(&sql("foreign-key.sql"), &catalog),
        vec![
            vec![],
            vec!["operations".to_string()],
            vec!["operations".to_string()],
            vec!["pending_operations".to_string()],
            vec!["lineages".to_string(), "operations".to_string()],
            vec!["operations".to_string()],
            vec!["draft_operations".to_string()],
            vec!["operations".to_string()],
            vec![],
            vec![],
            vec!["operations".to_string()],
            vec!["operations".to_string()],
            vec!["operations".to_string()],
            vec![
                "partial_lineages".to_string(),
                "partial_operations".to_string(),
            ],
            vec![],
            vec![],
            vec!["labeled_keys".to_string()],
        ]
    );
}
