use super::*;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-no-test-state-mutation/fixture/appended-fragment",
        ),
    )
}

#[test]
fn appended_helper_keeps_statement_text_without_invalid_sql() {
    let root = root();
    let files = ["test-helpers/keys.mts", "tests/impact.test.mts"].map(|rel| root.join(rel));
    let sources = super::super::super::source_store_for_files(&files);
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/configs/appended-fragment.yml",
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
    let sql = facts
        .embedded_sql(&files[0], &verdicts[0].profile)
        .unwrap_or_else(|error| panic!("{error}"))
        .calls
        .iter()
        .flat_map(|call| {
            if call.variants.is_empty() {
                call.sql_text.iter().cloned().collect::<Vec<_>>()
            } else {
                call.variants
                    .iter()
                    .map(|variant| variant.sql_text.clone())
                    .collect()
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sql,
        ["/* q */WITH k AS (SELECT id FROM t WHERE flag = sql_placeholder_1)  SELECT id FROM k WHERE id = ANY(sql_placeholder_2::uuid[]) ORDER BY id LIMIT 3"
            .to_string()]
    );
    assert!(
        verdicts.iter().all(|row| {
            !row.residual_reasons
                .iter()
                .any(|reason| reason == "invalid-sql")
                && row.known_prefix.is_none()
        }),
        "invalid or truncated prefix: {verdicts:#?}"
    );
}
