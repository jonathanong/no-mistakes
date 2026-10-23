use super::*;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlVerdictStatus};

fn verdict(subcheck: &str) -> SqlObligationVerdict {
    SqlObligationVerdict {
        profile: EmbeddedSqlOptions::configured("@example/db", &[]),
        rule: "postgres-require-query-annotation".to_string(),
        application: None,
        subcheck: subcheck.to_string(),
        file: "src/query.ts".to_string(),
        span: (10, 20),
        status: SqlVerdictStatus::Abstain,
        advisory: false,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec!["invalid-composed-variant".to_string()],
    }
}

#[test]
fn lazy_advisory_sink_stays_off_the_regular_verdict_path() {
    let facts = CheckFactMap::default();
    assert!(!facts.wants_sql_verdicts());
    assert!(facts.sql_advisories.get().is_none());
    assert!(facts.sql_verdicts().is_empty());

    facts.record_sql_advisory(verdict("invalid-composed-variant"));

    assert!(!facts.wants_sql_verdicts());
    assert_eq!(facts.sql_verdicts().len(), 1);
    assert!(facts.sql_verdicts()[0].advisory);
}

#[test]
fn application_advisories_keep_identity_with_regular_sink_disabled() {
    let facts = CheckFactMap::default();
    assert!(!facts.wants_sql_verdicts());
    assert!(facts.for_sql_application(2, Some("annotation")).is_none());

    let view = facts.view_with_supplemental(&CheckFactMap::default(), Vec::new());
    let application = view.for_sql_application_with_advice(2, Some("annotation"));
    application.record_sql_verdict(verdict("leading-annotation"));
    assert!(facts.sql_advisories.get().is_none());
    assert!(facts.sql_verdicts().is_empty());

    application.record_sql_advisory(verdict("invalid-composed-variant"));

    assert!(!facts.wants_sql_verdicts());
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    let advisory = &verdicts[0];
    assert_eq!(advisory.subcheck, "invalid-composed-variant");
    assert!(advisory.advisory);
    assert_eq!(
        advisory.application,
        Some(crate::codebase::postgres::SqlRuleApplication {
            index: 2,
            name: Some("annotation".to_string()),
        })
    );
}

#[test]
fn fresh_verdict_view_does_not_inherit_prior_lazy_advisories() {
    let facts = CheckFactMap::default();
    facts.record_sql_advisory(verdict("invalid-composed-variant"));

    let fresh = facts.with_fresh_sql_verdicts();

    assert!(fresh.sql_verdicts().is_empty());
    assert_eq!(facts.sql_verdicts().len(), 1);
}
