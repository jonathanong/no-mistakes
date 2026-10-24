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
    merge_sql(
        &mut advice,
        Path::new("/missing"),
        &sources,
        &CheckFactMap::default(),
        &rows,
    );
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
    merge_sql(
        &mut advice,
        Path::new("/missing"),
        &sources,
        &CheckFactMap::default(),
        &[],
    );
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
        &CheckFactMap::default(),
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
        &CheckFactMap::default(),
        &[first, second],
    );
    assert_eq!(advice, expected);
}

#[test]
fn low_display_locations_require_the_exact_rule_local_application() {
    use no_mistakes::codebase::check_facts::{
        collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
    };
    use no_mistakes::codebase::postgres::configure_prepared_postgres_plan;
    use no_mistakes::codebase::ts_source::FileInventory;
    use std::sync::Arc;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/opaque-escape-sites");
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let path = root.join("src/mixed.mts");
    let config =
        no_mistakes::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes-mixed.yml")))
            .unwrap();
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&path),
    ))));
    let mut plan = CheckFactPlan::default();
    configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![path.clone()],
        Vec::new(),
        plan,
        None,
        sources.clone(),
    );
    let source = sources.read_path(&path).unwrap();
    let start = source.find("read, // MAX_SITE").unwrap() as u32;
    let mut low = row();
    low.rule = "postgres-no-offset".into();
    low.profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    low.file = "src/mixed.mts".into();
    low.span = (start, start + 4);
    low.application = Some(SqlRuleApplication {
        index: 0,
        name: Some("low-helper".into()),
    });
    let mut max = low.clone();
    max.application = Some(SqlRuleApplication {
        index: 1,
        name: Some("max-helper".into()),
    });
    // The preceding other-rule application makes the low scan global index 1,
    // while its public verdict application index remains rule-local index 0.
    assert_eq!(facts.prepared_low_sql_display_line(&root, &low), Some(3));
    assert_eq!(facts.prepared_low_sql_display_line(&root, &max), None);
    let mut wrong = low.clone();
    wrong.rule = "postgres-no-unbounded-sql-identifier".into();
    assert_eq!(facts.prepared_low_sql_display_line(&root, &wrong), None);
    wrong = low.clone();
    wrong.span = (0, 1);
    assert_eq!(facts.prepared_low_sql_display_line(&root, &wrong), None);
    wrong = low.clone();
    wrong.file = "missing.mts".into();
    assert_eq!(facts.prepared_low_sql_display_line(&root, &wrong), None);
    let mut advice = Vec::new();
    merge_sql(
        &mut advice,
        &root,
        &sources,
        &facts,
        &[low.clone(), max.clone()],
    );
    assert_eq!(advice.len(), 2);
    assert_eq!(
        advice
            .iter()
            .find(|row| row.message.contains("low-helper"))
            .unwrap()
            .line,
        3
    );
    assert_eq!(
        advice
            .iter()
            .find(|row| row.message.contains("max-helper"))
            .unwrap()
            .line,
        4
    );
    assert_eq!(
        low.span, max.span,
        "display projection must never rewrite verdict identity"
    );
}
