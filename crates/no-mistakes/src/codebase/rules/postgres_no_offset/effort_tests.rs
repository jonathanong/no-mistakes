use super::*;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&super::tests::fixture("effort"))
}
fn load(name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&root(), Some(&root().join(name))).unwrap()
}
fn run(name: &str) -> Vec<RuleFinding> {
    let cfg = load(name);
    let files = crate::codebase::ts_source::discover_files(&root(), &[]);
    let sources = super::super::source_store_for_files(&files);
    let mut findings = check_with_files(&root(), &cfg, &files).unwrap();
    super::super::suppress_rule_findings_with_sources(&root(), &mut findings, &sources);
    findings
}

#[test]
fn loose_selector_stays_on_named_imports_and_excludes_globals_and_local_shadows() {
    let mut cfg = load(".no-mistakes.max.yml");
    cfg.rules[0].options = serde_yaml::from_str("{ include: [src/loose.ts], importSpecifier: '', executorNames: [query, globalQuery, read] }").unwrap();
    let file = root().join("src/loose.ts");
    let max = check_with_files(&root(), &cfg, std::slice::from_ref(&file)).unwrap();
    assert_eq!(
        max.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        [2]
    );
    cfg.effort = Some(crate::config::v2::Effort::Low);
    assert_eq!(
        check_with_files(&root(), &cfg, std::slice::from_ref(&file)).unwrap(),
        max
    );
}

#[test]
fn namespace_non_query_methods_remain_out_of_scope() {
    let mut cfg = load(".no-mistakes.max.yml");
    cfg.rules[0].options = serde_yaml::from_str(
        "{ include: [src/namespace.ts], importSpecifier: '@example/db', executorNames: [read] }",
    )
    .unwrap();
    let file = root().join("src/namespace.ts");
    let max = check_with_files(&root(), &cfg, std::slice::from_ref(&file)).unwrap();
    assert!(max.is_empty());
    cfg.effort = Some(crate::config::v2::Effort::Low);
    assert!(check_with_files(&root(), &cfg, std::slice::from_ref(&file))
        .unwrap()
        .is_empty());
}

fn prepared(
    mut config: NoMistakesConfig,
) -> (
    NoMistakesConfig,
    Vec<PathBuf>,
    std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    crate::codebase::check_facts::CheckFactMap,
) {
    let files = crate::codebase::ts_source::discover_files(&root(), &[]);
    let sources = super::super::source_store_for_files(&files);
    super::super::analysis_policy::prepare_rule_analysis(&root(), &mut config, &files, &sources)
        .unwrap();
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&root(), &config, &mut plan)
        .unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root(),
            files.clone(),
            Vec::new(),
            plan,
            None,
            sources.clone(),
        );
    (config, files, sources, facts)
}

#[test]
fn low_skips_evaluator_and_reuses_literal_parse_across_calls_and_applications() {
    let mut cfg = load(".no-mistakes.low-ignore.yml");
    cfg.rules[0].name = Some("first".into());
    let mut second = cfg.rules[0].clone();
    second.name = Some("second".into());
    cfg.rules.push(second);
    let (cfg, files, sources, facts) = prepared(cfg);
    assert!(facts.ts.values().all(|file| file.embedded_sql.is_empty()));
    assert!(facts
        .ts
        .values()
        .any(|file| !file.sql_identifier_syntax.profiles.is_empty()));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
    });
    assert_eq!(
        observer.snapshot().work["postgres.no_offset.low.literal_parses"],
        6
    );
    let reads = sources.physical_read_count();
    check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
    assert_eq!(sources.physical_read_count(), reads);
}

#[test]
fn mixed_max_application_keeps_max_profile_and_borrows_its_sql_projection() {
    let mut cfg = load(".no-mistakes.low-ignore.yml");
    let mut max = cfg.rules[0].clone();
    max.options
        .as_mapping_mut()
        .unwrap()
        .insert("effort".into(), "max".into());
    cfg.rules.push(max);
    let (cfg, files, sources, facts) = prepared(cfg);
    assert!(facts.ts.values().any(|file| !file.embedded_sql.is_empty()));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
    });
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.no_offset.low.literal_parses")
            .copied()
            .unwrap_or(0),
        0
    );
    let mut low_cfg = cfg.clone();
    low_cfg.rules.truncate(1);
    let mut borrowed =
        check_with_files_sources_and_facts(&root(), &low_cfg, &files, &sources, &facts).unwrap();
    super::super::suppress_rule_findings_with_sources(&root(), &mut borrowed, &sources);
    assert_eq!(borrowed, run(".no-mistakes.low-ignore.yml"));
}

#[test]
fn malformed_sql_is_uncertainty_and_never_proven_clear() {
    let mut cfg = load(".no-mistakes.low-advise.yml");
    cfg.rules[0].options.as_mapping_mut().unwrap().insert(
        "include".into(),
        serde_yaml::from_str("[src/invalid.ts]").unwrap(),
    );
    let (cfg, files, sources, mut facts) = prepared(cfg);
    facts.enable_sql_verdicts();
    assert!(
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts)
            .unwrap()
            .iter()
            .all(|finding| finding.file.ends_with(".sql"))
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 2);
    assert!(verdicts.iter().all(|verdict| verdict.status
        == crate::codebase::postgres::SqlVerdictStatus::Abstain
        && verdict.residual_reasons == ["invalid-sql"]));
}

mod line_endings;
mod prepared_contracts;

mod helper_recovery;
