use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;
use std::path::PathBuf;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/fixture/frozen-record-dynamic",
    ))
}

fn statements(call: &EmbeddedSqlCall) -> Vec<String> {
    if call.variants.is_empty() {
        call.sql_text.iter().cloned().collect()
    } else {
        call.variants
            .iter()
            .map(|variant| variant.sql_text.clone())
            .collect()
    }
}

fn marked(sql: &[String], marker: &str) -> Vec<String> {
    sql.iter()
        .filter(|text| text.contains(marker))
        .cloned()
        .collect()
}

#[test]
fn dynamic_key_enumerates_imported_frozen_records() {
    let root = root();
    let files = [
        "test-helpers/config.mts",
        "test-helpers/write.mts",
        "tests/impact.test.mts",
    ]
    .map(|rel| root.join(rel));
    let sources = super::super::super::source_store_for_files(&files);
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/configs/frozen-record-dynamic.yml",
    );
    let mut config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap()
    .with_fresh_sql_verdicts();
    let _findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let sql = facts
        .embedded_sql(&files[1], &facts.sql_verdicts()[0].profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(statements)
        .collect::<Vec<_>>();
    assert_eq!(
        marked(&sql, "/* friend */"),
        [
            "/* friend */ DELETE FROM users WHERE user_id = $1".to_string(),
            "/* friend */ DELETE FROM posts WHERE post_id = $1".to_string(),
        ],
        "{sql:#?}"
    );
    let providers = marked(&sql, "/* provider */");
    assert_eq!(providers.len(), 7, "{providers:#?}");
    for index in 1..=7 {
        let table = format!("prov{index:02}");
        assert!(
            providers.iter().any(|text| {
                text == &format!("/* provider */ INSERT INTO {table} (id) VALUES ($1)")
            }),
            "missing {table}: {providers:#?}"
        );
    }
    let tables = marked(&sql, "/* table */");
    assert_eq!(tables.len(), 18, "{tables:#?}");
    let columns = marked(&sql, "/* name */");
    assert_eq!(columns.len(), 18, "{columns:#?}");
    for index in 1..=18 {
        let name = format!("item{index:02}");
        assert!(
            tables.iter().any(|text| {
                text == &format!("/* table */ INSERT INTO {name} (id) VALUES ($1)")
            }),
            "missing table {name}: {tables:#?}"
        );
        let column = format!("{name}_id");
        assert!(
            columns.iter().any(|text| {
                text == &format!(
                    "/* name */ INSERT INTO impacts (request_id, subject_id, {column}) VALUES ($1, $2, $3) RETURNING id"
                )
            }),
            "missing column {column}: {columns:#?}"
        );
    }
    assert!(
        facts.sql_verdicts().iter().all(|row| {
            !row.residual_reasons
                .iter()
                .any(|reason| reason == "invalid-sql")
        }),
        "{:#?}",
        facts.sql_verdicts()
    );
}
