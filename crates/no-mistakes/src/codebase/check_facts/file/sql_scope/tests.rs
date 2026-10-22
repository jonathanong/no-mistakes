use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::{EmbeddedSqlOptions, EmbeddedSqlRuleScan};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-verdicts/fixture/narrowed-preparation")
            .join(name),
    )
}

fn profile() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@db", &["pool".into()])
}

fn scoped_plan() -> CheckFactPlan {
    let profile = profile();
    CheckFactPlan {
        embedded_sql: true,
        embedded_sql_options: vec![profile.clone()],
        embedded_sql_rule_ids: [(
            profile,
            vec![
                EmbeddedSqlRuleScan::new("postgres-no-offset", &["selected/**/*.ts".into()], &[])
                    .unwrap(),
            ],
        )]
        .into(),
        postgres_dml: true,
        postgres_fragments: true,
        ..Default::default()
    }
}

fn collect(name: &str, plan: &CheckFactPlan) -> (CheckFileFacts, usize) {
    let path = fixture(name);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    work_probe::reset();
    let facts = super::super::collect_file_facts_with_session_and_sources(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &fixture(""),
        &path,
        plan,
        None,
        &sources,
        false,
    )
    .unwrap();
    (facts, work_probe::count())
}

#[test]
fn nonexporting_unscanned_file_skips_actual_sql_walk_but_keeps_ts_imports() {
    let plan = scoped_plan();
    let (narrowed, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(
        walks, 0,
        "out-of-scope private executors must not visit the AST"
    );
    assert!(narrowed.embedded_sql.is_empty());
    assert!(!narrowed.ts.imported_bindings.is_empty());
    let mut direct = plan;
    direct.embedded_sql_rule_ids.clear();
    let (legacy, walks) = collect("unselected/private.ts", &direct);
    assert_eq!(walks, 1);
    assert_eq!(legacy.embedded_sql[0].1.calls.len(), 1);
    assert!(legacy.embedded_sql[0].1.calls[0]
        .sql_text
        .as_ref()
        .unwrap()
        .contains("OFFSET 9"));
}

#[test]
fn all_potential_exporters_and_selected_nonexporters_keep_the_walk() {
    let plan = scoped_plan();
    for name in [
        "selected/query.ts",
        "providers/helpers.ts",
        "providers/constants.ts",
        "providers/barrel.ts",
        "providers/cycle.ts",
        "unselected/exporter.ts",
    ] {
        let (facts, walks) = collect(name, &plan);
        assert_eq!(walks, 1, "provider or selected file {name}");
        assert_eq!(facts.embedded_sql.len(), 1);
    }
}

#[test]
fn parser_recovery_keeps_the_conservative_walk() {
    let (facts, walks) = collect("unselected/recovery.ts", &scoped_plan());
    assert!(facts.parse_error.is_some());
    assert_eq!(walks, 1);
    assert_eq!(facts.embedded_sql.len(), 1);
}

#[test]
fn incomplete_provider_metadata_never_prunes_the_walk() {
    use crate::codebase::ts_source::facts::TsFileFacts;
    assert!(retains_provider(&TsFileFacts {
        operational_error: Some("collection failed".into()),
        ..Default::default()
    }));
    assert!(retains_provider(&TsFileFacts {
        parse_error: Some("recovered parse".into()),
        ..Default::default()
    }));
    assert!(!retains_provider(&TsFileFacts::default()));
}

#[test]
fn statements_and_fragments_are_prepared_only_for_the_profile_scan_union() {
    let mut plan = scoped_plan();
    let other = EmbeddedSqlOptions::configured("@other", &["client".into()]);
    plan.embedded_sql_options.push(other.clone());
    plan.embedded_sql_rule_ids.insert(
        other.clone(),
        vec![
            EmbeddedSqlRuleScan::new("postgres-no-offset", &["other/**/*.ts".into()], &[]).unwrap(),
        ],
    );
    // A duplicate overlapping application must not multiply preparation.
    let same = plan.embedded_sql_rule_ids[&profile()][0].clone();
    plan.embedded_sql_rule_ids
        .get_mut(&profile())
        .unwrap()
        .push(same);
    let names = [
        "selected/query.ts",
        "selected/suppressed.ts",
        "other/query.ts",
        "unselected/private.ts",
        "unselected/exporter.ts",
        "providers/helpers.ts",
        "providers/constants.ts",
        "providers/barrel.ts",
        "providers/cycle.ts",
    ];
    let paths: Vec<_> = names.iter().map(|name| fixture(name)).collect();
    let narrowed = collect_check_facts(&fixture(""), paths.clone(), plan.clone());
    assert_eq!(
        narrowed.postgres.embedded.len(),
        3,
        "old all-file/profile preparation produces 18 entries"
    );
    assert_eq!(narrowed.postgres.fragments.len(), 3);
    assert!(narrowed
        .postgres_statements(&fixture("other/query.ts"), Some(&profile()))
        .is_err());
    assert!(narrowed
        .postgres_fragments(&fixture("unselected/exporter.ts"), &profile())
        .is_err());
    plan.embedded_sql_rule_ids.clear();
    let direct = collect_check_facts(&fixture(""), paths, plan);
    assert_selected_calls(&narrowed, &direct);
    assert_eq!(direct.postgres.embedded.len(), 18);
}

fn assert_selected_calls(narrowed: &CheckFactMap, direct: &CheckFactMap) {
    let path = fixture("selected/query.ts");
    let narrowed = narrowed.embedded_sql(&path, &profile()).unwrap();
    let direct = direct.embedded_sql(&path, &profile()).unwrap();
    assert_eq!(narrowed.calls, direct.calls);
    assert_eq!(narrowed.fragments, direct.fragments);
    assert_eq!(narrowed.calls.len(), 2);
    for (call, offset) in narrowed.calls.iter().zip(["OFFSET 2", "OFFSET 3"]) {
        assert!(
            call.sql_text.as_ref().unwrap().contains(offset),
            "helper SQL must remain proven"
        );
    }
}

#[test]
fn default_exported_helper_outside_the_scan_retains_concrete_sql() {
    let mut plan = scoped_plan();
    plan.embedded_sql_rule_ids.insert(
        profile(),
        vec![
            EmbeddedSqlRuleScan::new("postgres-no-offset", &["default/query.ts".into()], &[])
                .unwrap(),
        ],
    );
    let sink = fixture("default/query.ts");
    let provider = fixture("providers/default.ts");
    let paths = vec![sink.clone(), provider.clone()];
    let narrowed = collect_check_facts(&fixture(""), paths.clone(), plan.clone());
    assert!(narrowed.embedded_sql(&provider, &profile()).is_ok());
    let calls = &narrowed.embedded_sql(&sink, &profile()).unwrap().calls;
    assert_eq!(calls.len(), 1);
    assert!(calls[0].sql_text.as_ref().unwrap().contains("OFFSET 6"));
    plan.embedded_sql_rule_ids.clear();
    let direct = collect_check_facts(&fixture(""), paths, plan);
    assert_eq!(
        calls,
        &direct.embedded_sql(&sink, &profile()).unwrap().calls
    );
}

#[test]
fn unspecified_annotation_profiles_keep_required_sql_summaries() {
    let mut plan = scoped_plan();
    let second = EmbeddedSqlOptions::configured("@other", &["client".into()]);
    plan.embedded_sql_options
        .extend([second.clone(), profile(), second.clone()]);
    plan.embedded_sql_rule_ids.insert(second, Vec::new());
    plan.query_annotation = true;
    let expected = profiles(&plan);
    let (facts, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(walks, 2);
    assert_eq!(facts.embedded_sql.len(), 2);
    assert_eq!(
        facts
            .query_annotation
            .iter()
            .map(|(profile, _)| profile.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(profiles(&CheckFactPlan::default()), Vec::new());
    let defaults = CheckFactPlan {
        embedded_sql: true,
        ..Default::default()
    };
    assert_eq!(profiles(&defaults), vec![EmbeddedSqlOptions::default()]);
}

#[test]
fn explicit_annotation_profiles_do_not_widen_other_sql_profiles() {
    let mut plan = scoped_plan();
    let second = EmbeddedSqlOptions::configured("@other", &["client".into()]);
    plan.embedded_sql_options.push(second.clone());
    plan.embedded_sql_rule_ids.insert(second, Vec::new());
    plan.query_annotation = true;
    plan.query_annotation_options = Some(vec![profile()]);
    let (facts, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(walks, 1);
    assert_eq!(facts.embedded_sql[0].0, profile());
    assert_eq!(facts.embedded_sql[0].1.calls.len(), 1);
    assert_eq!(facts.embedded_sql[0].1.call_spans.len(), 1);
    assert_eq!(facts.query_annotation.len(), 1);
    plan.query_annotation_options = Some(Vec::new());
    let (facts, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(walks, 0);
    assert!(facts.embedded_sql.is_empty());
    assert!(facts.query_annotation.is_empty());
}

#[test]
fn implicit_default_annotation_profile_keeps_its_raw_pair() {
    let plan = CheckFactPlan {
        embedded_sql: true,
        query_annotation: true,
        embedded_sql_rule_ids: [(EmbeddedSqlOptions::default(), Vec::new())].into(),
        ..Default::default()
    };
    let (facts, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(walks, 1);
    assert_eq!(facts.embedded_sql.len(), 1);
    assert_eq!(facts.embedded_sql[0].0, EmbeddedSqlOptions::default());
    assert_eq!(facts.query_annotation[0].0, EmbeddedSqlOptions::default());
}

#[test]
fn low_effort_heads_keep_raw_profiles_even_outside_all_sql_scans() {
    let mut plan = scoped_plan();
    let second = EmbeddedSqlOptions::configured("@other", &["client".into()]);
    plan.embedded_sql_options.push(second.clone());
    plan.embedded_sql_rule_ids.insert(second, Vec::new());
    plan.query_annotation_heads = true;
    plan.query_annotation_options = Some(Vec::new());
    let (facts, walks) = collect("unselected/private.ts", &plan);
    assert_eq!(walks, 2);
    assert_eq!(facts.embedded_sql.len(), 2);
    assert!(facts.query_annotation.is_empty());
    assert!(facts.query_annotation_heads.is_some());
    let raw = facts
        .embedded_sql
        .iter()
        .find(|(options, _)| options == &profile())
        .unwrap();
    assert_eq!(raw.1.call_spans.len(), 1);
    assert!(raw.1.calls[0]
        .sql_text
        .as_ref()
        .unwrap()
        .contains("OFFSET 9"));
    plan.query_annotation_heads = false;
    assert_eq!(collect("unselected/private.ts", &plan).1, 0);
}
