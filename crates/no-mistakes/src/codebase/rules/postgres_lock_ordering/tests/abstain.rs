use super::*;
use std::sync::Arc;

fn inspect(scenario: &str, config_name: &str) -> (Vec<String>, Vec<String>) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(scenario));
    let file = ts_file(&root);
    let files = [file.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config_path = fixture_root().join("configs").join(config_name);
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
    let findings = check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts)
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    let profile = &facts.sql_verdicts()[0].profile;
    let embedded = facts
        .embedded_sql(&file, profile)
        .unwrap_or_else(|error| panic!("{error}"));
    let sql = embedded
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
        .collect();
    (sql, findings)
}

#[test]
fn closed_relation_set_does_not_report_a_catalog_key_miss() {
    let (sql, findings) = inspect("closed-table-identifier", "catalog.yml");
    assert!(
        findings.is_empty()
            && sql.iter().any(|text| text.contains("relation__topic"))
            && sql.iter().any(|text| text.contains("relation__post")),
        "sql={sql:#?}\nfindings={findings:#?}"
    );
}

#[test]
fn appended_unresolved_fragment_does_not_fail_parse() {
    let (sql, findings) = inspect("appended-scope-fragment", "ignore-unanalyzable.yml");
    assert!(findings.is_empty(), "sql={sql:#?}\nfindings={findings:#?}");
    assert!(
        sql.iter().all(|text| !glued_placeholder(text)),
        "bare placeholder in rule text: {sql:#?}"
    );
    assert!(
        sql.iter()
            .all(|text| !text.contains("FOR UPDATE") || text.contains("SKIP LOCKED")),
        "unresolved fragment became FOR UPDATE text: {sql:#?}"
    );
}

fn glued_placeholder(text: &str) -> bool {
    text.match_indices("sql_placeholder_").any(|(index, _)| {
        text[..index]
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}
