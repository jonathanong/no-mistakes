use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;
use std::path::PathBuf;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-no-test-state-mutation/fixture/identifier-holes",
        ),
    )
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
fn unknown_identifier_holes_keep_the_full_statement() {
    let root = root();
    let files = [root.join("tests/mutate.test.mts")];
    let sources = super::super::super::source_store_for_files(&files);
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/configs/identifier-holes.yml",
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
        .embedded_sql(&files[0], &facts.sql_verdicts()[0].profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(statements)
        .collect::<Vec<_>>();
    assert!(
        sql.iter()
            .any(|text| text == "UPDATE sql_placeholder_1 SET token = $1 WHERE $2 = $3"),
        "{sql:#?}"
    );
    assert!(
        sql.iter()
            .any(|text| text == "DELETE FROM sql_placeholder_1 WHERE $1 = $2"),
        "{sql:#?}"
    );
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
