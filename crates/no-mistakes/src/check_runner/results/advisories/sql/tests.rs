use super::*;
use no_mistakes::codebase::postgres::{EmbeddedSqlOptions, SqlRuleApplication};

fn row() -> SqlObligationVerdict {
    SqlObligationVerdict {
        profile: EmbeddedSqlOptions::default(),
        rule: "policy".into(),
        application: None,
        subcheck: "first".into(),
        file: "missing.ts".into(),
        span: (0, 1),
        status: SqlVerdictStatus::Abstain,
        advisory: true,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec!["opaque".into()],
    }
}

#[test]
fn groups_only_matching_calls_rules_applications_and_profiles() {
    let base = row();
    let mut rows = vec![base.clone()];
    let mut other = base.clone();
    other.span = (1, 2);
    rows.push(other);
    let mut other = base.clone();
    other.rule = "other-policy".into();
    rows.push(other);
    let mut other = base.clone();
    other.application = Some(SqlRuleApplication {
        index: 0,
        name: None,
    });
    rows.push(other);
    let mut other = base.clone();
    other.profile.import_specifier = "other-profile".into();
    rows.push(other);
    let mut other = base.clone();
    other.file = "other.ts".into();
    rows.push(other);
    for (advisory, status, residual_reasons) in [
        (false, SqlVerdictStatus::Abstain, vec!["opaque".into()]),
        (true, SqlVerdictStatus::ProvenClear, vec!["opaque".into()]),
        (true, SqlVerdictStatus::Abstain, vec![]),
    ] {
        let mut ignored = base.clone();
        ignored.advisory = advisory;
        ignored.status = status;
        ignored.residual_reasons = residual_reasons;
        rows.push(ignored);
    }
    let sources = SourceStore::new(std::sync::Arc::new(
        no_mistakes::codebase::ts_source::FileInventory::from_paths(&[]),
    ));
    let mut advice = Vec::new();
    merge_sql(&mut advice, Path::new("/missing"), &sources, &rows);
    assert_eq!(advice.len(), 6);
    assert!(advice.iter().all(|finding| finding.line == 1));
    assert!(advice.iter().all(|finding| {
        finding.message.contains("has unresolved SQL for")
            && !finding.message.contains("proved a violation")
    }));
    assert!(advice
        .iter()
        .any(|finding| finding.message.contains("application #0")));
    let original = advice.clone();
    merge_sql(&mut advice, Path::new("/missing"), &sources, &[]);
    assert_eq!(advice, original);
}

#[test]
fn aggregation_sorts_and_deduplicates_all_residual_evidence() {
    let mut first = row();
    first.helper_lines = vec![8, 4];
    first.known_prefix = Some("SELECT ".into());
    let mut second = first.clone();
    second.subcheck = "second".into();
    second.residual_reasons = vec!["invalid".into(), "opaque".into()];
    second.helper_lines = vec![4, 2];
    second.status = SqlVerdictStatus::Violation;
    let mut advice = Vec::new();
    let sources = SourceStore::new(std::sync::Arc::new(
        no_mistakes::codebase::ts_source::FileInventory::from_paths(&[]),
    ));
    merge_sql(
        &mut advice,
        Path::new("/missing"),
        &sources,
        &[second.clone(), first.clone()],
    );
    assert_eq!(advice.len(), 1);
    assert_eq!(advice[0].target.as_deref(), Some("first, second"));
    assert!(advice[0]
        .message
        .contains("proved a violation, but remaining SQL is unresolved for"));
    assert!(!advice[0].message.contains("has unresolved SQL for"));
    assert!(advice[0]
        .message
        .contains("invalid, opaque; known SQL prefix \"SELECT \"; helper lines [2, 4, 8]"));
    let expected = advice.clone();
    advice.clear();
    merge_sql(
        &mut advice,
        Path::new("/missing"),
        &sources,
        &[first, second],
    );
    assert_eq!(advice, expected);
}
