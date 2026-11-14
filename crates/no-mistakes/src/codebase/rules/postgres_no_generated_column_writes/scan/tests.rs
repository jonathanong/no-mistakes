use super::*;

#[test]
fn combined_catalog_reports_both_column_kinds() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/both-column-kinds",
    );
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let schema =
        crate::codebase::postgres::extract_schema_facts(&root, &sources, &paths[..1]).unwrap();
    let tables = super::super::catalog::live_tables(&schema.iter().collect::<Vec<_>>());
    let generated = super::super::catalog::catalog_from_tables(&tables, &[]);
    let trigger =
        super::super::catalog::trigger_catalog_from_tables(&tables, &["updated_at".into()]);
    let mut combined = trigger.clone();
    combined.extend_from(&generated);
    let catalogs = Catalogs {
        catalog: generated,
        trigger,
        combined,
        known_tables: tables
            .iter()
            .map(|(name, table)| (name.clone(), table.complete))
            .collect(),
    };
    let source = sources.read_path(&paths[1]).unwrap();
    let statements = crate::codebase::postgres::extract_sql_statement_facts(&source);
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    extend_writes(
        &mut findings,
        "writes.sql",
        &[statements],
        &mut crate::codebase::rules::VariantFindingDedup::default(),
        None,
        &|_, _| &catalogs,
        &super::positional::PositionalUncertainty::Ignore,
    );
    let findings = findings.finish();
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("trigger-maintained")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("generated column")));
}

fn votes_catalog(order: Option<Vec<String>>) -> Catalogs {
    let mut columns = crate::codebase::postgres::GeneratedTableColumns::default();
    columns.insert_table(crate::codebase::postgres::GeneratedTable {
        name: "votes".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: order,
    });
    Catalogs {
        catalog: columns.clone(),
        trigger: crate::codebase::postgres::GeneratedTableColumns::default(),
        combined: columns,
        known_tables: std::collections::BTreeMap::new(),
    }
}

fn positional_messages(order: Option<Vec<String>>, width: Option<usize>) -> Vec<String> {
    let catalogs = votes_catalog(order);
    let mut statement = crate::codebase::postgres::SqlStatementFileFacts::default();
    statement
        .writes
        .push(crate::codebase::postgres::SqlWriteFact {
            table: "votes".to_string(),
            columns: crate::codebase::postgres::SqlWriteColumns::Positional(width),
            line: 1,
            column: 1,
            column_keys: None,
        });
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    extend_writes(
        &mut findings,
        "writes.sql",
        &[statement],
        &mut crate::codebase::rules::VariantFindingDedup::default(),
        None,
        &|_, _| &catalogs,
        &super::positional::PositionalUncertainty::Report,
    );
    findings
        .finish()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn unknown_order_requires_values_and_default_values_stay_clean() {
    let unproven = positional_messages(None, Some(2));
    assert_eq!(unproven.len(), 1);
    assert!(unproven[0].contains("column order is unknown"));
    assert!(positional_messages(None, None).is_empty());
    assert!(positional_messages(None, Some(0)).is_empty());
    let order = Some(vec!["id".to_string(), "created_at".to_string()]);
    assert!(positional_messages(order.clone(), None).is_empty());
    assert!(positional_messages(order.clone(), Some(1)).is_empty());
    let hit = positional_messages(order, Some(2));
    assert_eq!(hit.len(), 1);
    assert!(hit[0].contains("votes.created_at"));
}
