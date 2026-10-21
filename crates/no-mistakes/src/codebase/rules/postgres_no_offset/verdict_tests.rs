use super::tests::{config_with_options, fixture};
use super::*;
use crate::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};

#[test]
fn prepared_verdicts_distinguish_exact_union_prefix_unknown_and_invalid_sql() {
    let root = fixture("verdicts");
    let file = root.join("src/query.ts");
    let files = vec![file.clone()];
    let config = config_with_options("importSpecifier: ''\nexecutorNames: [query]");
    let sources = super::super::source_store_for_files(&files);
    let mut facts =
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap();
    let profile =
        crate::codebase::postgres::EmbeddedSqlOptions::configured("", &["query".to_string()]);
    let requested =
        crate::codebase::postgres::configured_embedded_sql_options_for_checks(&config).unwrap();
    assert_eq!(requested, vec![profile.clone()]);
    let prepared = facts
        .ts
        .get(&file)
        .unwrap()
        .embedded_sql
        .iter()
        .map(|(profile, _)| profile.clone())
        .collect::<Vec<_>>();
    assert_eq!(prepared, requested);
    assert_eq!(facts.embedded_sql(&file, &profile).unwrap().calls.len(), 10);
    facts.enable_sql_verdicts();
    let _ = check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let verdicts = facts.sql_verdicts();
    let statuses = verdicts
        .iter()
        .map(|verdict| verdict.status)
        .collect::<Vec<_>>();
    assert_eq!(
        statuses,
        [
            SqlVerdictStatus::ProvenClear,
            SqlVerdictStatus::Violation,
            SqlVerdictStatus::Violation,
            SqlVerdictStatus::Violation,
            SqlVerdictStatus::Abstain,
            SqlVerdictStatus::Abstain,
            SqlVerdictStatus::Abstain,
            SqlVerdictStatus::Abstain,
            SqlVerdictStatus::Violation,
            SqlVerdictStatus::Abstain,
        ]
    );
    assert!(verdicts[2]
        .residual_reasons
        .contains(&"opaque-sql-tail".into()));
    assert!(verdicts[3]
        .residual_reasons
        .contains(&"opaque-sql-tail".into()));
    assert_eq!(verdicts[0].profile.executor_names, ["query"]);
    assert_eq!(verdicts[8].evidence, Some(SqlEvidenceKind::ExhaustiveUnion));
    assert!(verdicts[9]
        .residual_reasons
        .contains(&"opaque-sql-tail".into()));
    assert!(verdicts
        .iter()
        .all(|verdict| verdict.span.1 > verdict.span.0));
}
