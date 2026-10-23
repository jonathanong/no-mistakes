use super::*;
use crate::codebase::check_facts::CheckFactMap;

#[test]
fn truncated_variants_cannot_prove_clear_but_keep_known_offset_witnesses() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-append-verdicts.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let embedded =
        crate::codebase::postgres::extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(embedded.calls.len(), 2);
    assert!(embedded
        .calls
        .iter()
        .all(EmbeddedSqlCall::variants_truncated));
    assert_eq!(embedded.calls[0].variants.len(), 16);
    assert!(embedded.calls[0]
        .variants
        .iter()
        .all(|variant| !variant.sql_text.contains("OFFSET")));
    let statements =
        crate::codebase::postgres::collect::dml::embedded_call_facts(&embedded, false, false);
    assert!(statements.iter().all(|statement| !statement.parse_failed));
    let mut facts = CheckFactMap::default();
    facts.enable_sql_verdicts();
    record_sql_policy_verdicts(
        SqlPolicyVerdictContext {
            facts: &facts,
            embedded: &embedded,
            statements: &statements,
        },
        "postgres-no-offset",
        "query.ts",
        &options,
        &["offset".into()],
        false,
        |_, _, statement| {
            if statement.offset_uses.is_empty() {
                SqlBranchJudgment::clear()
            } else {
                SqlBranchJudgment::violation(true)
            }
        },
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 2);
    assert_eq!(verdicts[0].status, SqlVerdictStatus::Abstain);
    assert_eq!(verdicts[0].evidence, None);
    assert_eq!(verdicts[1].status, SqlVerdictStatus::Violation);
    assert_eq!(verdicts[1].evidence, Some(SqlEvidenceKind::VerifiedPrefix));
    assert!(verdicts.iter().all(|verdict| verdict
        .residual_reasons
        .contains(&"sql-variants-truncated".into())));
    assert_eq!(
        crate::codebase::postgres::sql_evidence_for_call(
            &embedded.calls[1],
            SqlVerdictStatus::Violation
        ),
        Some(SqlEvidenceKind::VerifiedPrefix)
    );
}

#[test]
fn unresolved_runtime_sql_abstains_before_statement_facts_are_needed() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/verdict-unresolved-runtime.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let embedded =
        crate::codebase::postgres::extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(embedded.calls.len(), 1);
    assert_eq!(embedded.calls[0].sql_text, None);

    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.enable_sql_verdicts();
    record_sql_policy_verdicts(
        SqlPolicyVerdictContext {
            facts: &facts,
            embedded: &embedded,
            statements: &[],
        },
        "postgres-rule",
        "query.ts",
        &options,
        &["bound".to_string()],
        false,
        |_, _, _| panic!("unresolved text cannot be judged"),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    assert_eq!(verdicts[0].status, SqlVerdictStatus::Abstain);
    assert_eq!(
        verdicts[0].residual_reasons,
        vec![
            "opaque-sql-tail".to_string(),
            "unresolved-sql-fragment".to_string()
        ]
    );
}
