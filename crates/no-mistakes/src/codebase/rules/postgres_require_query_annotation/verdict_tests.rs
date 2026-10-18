use super::*;
use crate::codebase::postgres::SqlVerdictStatus;

#[test]
fn advise_records_only_proven_executor_candidates() {
    let root = super::tests::fixture("provenance-advice");
    let file = root.join("src/query.ts");
    let files = [file];
    let config = super::tests::config_with_options(
        "importSpecifier: '@example/db'\nunanalyzableSql: advise",
    );
    let sources = super::super::source_store_for_files(&files);
    let mut facts =
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap();
    facts.enable_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 3, "{verdicts:?}");
    assert_eq!(verdicts[0].status, SqlVerdictStatus::ProvenClear);
    assert_eq!(verdicts[1].status, SqlVerdictStatus::Abstain);
    assert!(verdicts[1].advisory);
    assert!(verdicts[1]
        .residual_reasons
        .contains(&"unresolved-leading-sql".into()));
    assert_eq!(verdicts[2].status, SqlVerdictStatus::ProvenClear);
    assert!(verdicts[2].residual_reasons.is_empty());
}

#[test]
fn advice_does_not_promote_one_helper_context_to_a_whole_call_proof() {
    let root = super::tests::fixture("provenance-advice-helper");
    let file = root.join("src/helper.ts");
    let files = [file];
    let config = super::tests::config_with_options(
        "importSpecifier: '@example/db'\nunanalyzableSql: advise",
    );
    let sources = super::super::source_store_for_files(&files);
    let mut facts =
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap();
    facts.enable_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1, "{verdicts:?}");
    assert_eq!(verdicts[0].status, SqlVerdictStatus::Abstain);
}
