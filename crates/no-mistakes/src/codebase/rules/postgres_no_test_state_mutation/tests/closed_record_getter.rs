use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/fixture/closed-record-getter",
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

#[test]
fn closed_key_fills_the_insert_column() {
    let root = root();
    let files = [
        "test-helpers/config.mts",
        "test-helpers/insert.mts",
        "tests/impact.test.mts",
    ]
    .map(|rel| root.join(rel));
    let sources = super::super::super::source_store_for_files(&files);
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/configs/closed-record-getter.yml",
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
        Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap()
    .with_fresh_sql_verdicts();
    let _findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let insert = facts
        .embedded_sql(&files[1], &facts.sql_verdicts()[0].profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(statements)
        .collect::<Vec<_>>();
    assert_eq!(
        insert
            .iter()
            .filter(|sql| sql.contains("/* c */"))
            .cloned()
            .collect::<Vec<_>>(),
        [
            "/* c */ INSERT INTO impacts (request_id, subject_id, a_id) VALUES ($1, $2, $3)"
                .to_string()
        ],
        "throw guard: {insert:#?}"
    );
    assert_eq!(
        insert
            .iter()
            .filter(|sql| sql.contains("/* early */"))
            .cloned()
            .collect::<Vec<_>>(),
        [
            "/* early */ INSERT INTO impacts (request_id, subject_id, b_id) VALUES ($1, $2, $3)"
                .to_string()
        ],
        "early return: {insert:#?}"
    );
    let verdicts = facts.sql_verdicts();
    assert!(
        verdicts.iter().all(|row| {
            !row.residual_reasons
                .iter()
                .any(|reason| reason == "invalid-sql")
                && row.known_prefix.as_deref().is_none_or(|prefix| {
                    !prefix.ends_with("subject_id, ") && !prefix.contains("subject_id, \"")
                })
        }),
        "truncated or invalid prefix: {verdicts:#?}"
    );
}
