use super::*;
use crate::codebase::postgres::{EmbeddedSqlCall, SqlVerdictStatus};

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/fixture/closed-record-prefix",
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
fn closed_record_prefix_matches_main_or_resolves() {
    let root = root();
    let files = [
        "test-helpers/config.mts",
        "test-helpers/insert.mts",
        "test-helpers/lock.mts",
        "test-helpers/assert.mts",
        "tests/impact.test.mts",
    ]
    .map(|rel| root.join(rel));
    let sources = super::super::super::source_store_for_files(&files);
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/configs/closed-record-prefix.yml",
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
    let verdicts = facts.sql_verdicts();
    let profile = verdicts
        .first()
        .unwrap_or_else(|| panic!("no verdicts: {verdicts:#?}"))
        .profile
        .clone();
    let insert = facts
        .embedded_sql(&files[1], &profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(statements)
        .collect::<Vec<_>>();
    let lock = facts
        .embedded_sql(&files[2], &profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(statements)
        .collect::<Vec<_>>();
    assert_eq!(
        insert
            .iter()
            .filter(|sql| sql.contains("/* insertImpact */"))
            .cloned()
            .collect::<Vec<_>>(),
        [
            "/* insertImpact */ INSERT INTO impacts (request_id, a_id) VALUES ($1, $2)".to_string(),
            "/* insertImpact */ INSERT INTO impacts (request_id, b_id) VALUES ($1, $2)".to_string(),
        ],
        "column record: {insert:#?}"
    );
    assert_eq!(
        insert
            .iter()
            .filter(|sql| sql.contains("/* insertTable */"))
            .cloned()
            .collect::<Vec<_>>(),
        [
            "/* insertTable */ INSERT INTO retained_a (id) VALUES ($1)".to_string(),
            "/* insertTable */ INSERT INTO retained_b (id) VALUES ($1)".to_string(),
        ],
        "table record: {insert:#?}"
    );
    assert!(
        verdicts.iter().all(|row| {
            !row.residual_reasons
                .iter()
                .any(|reason| reason == "invalid-sql")
                && row.known_prefix.as_deref().is_none_or(|prefix| {
                    !prefix.contains("request_id, \"") && !prefix.ends_with("request_id, ")
                })
        }),
        "truncated or invalid prefix: {verdicts:#?}"
    );
    let lock_rows: Vec<_> = verdicts
        .iter()
        .filter(|row| row.file == "test-helpers/lock.mts")
        .collect();
    assert!(
        lock_rows.iter().any(|row| {
            row.status == SqlVerdictStatus::Abstain
                && row.residual_reasons == ["opaque-sql-tail"]
                && row.known_prefix.as_deref()
                    == Some(
                        "/* lockReference */ LOCK TABLE sql_placeholder_1 IN ACCESS EXCLUSIVE MODE",
                    )
        }),
        "lock prefix: {lock:#?} {lock_rows:#?}"
    );
    assert!(
        lock.iter().all(|sql| !sql.contains('\u{10FFFE}')),
        "lock leaked a hole marker: {lock:#?}"
    );
}
