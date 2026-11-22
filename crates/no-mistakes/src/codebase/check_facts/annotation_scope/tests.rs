use super::*;
use crate::codebase::{check_facts, ts_resolver::TsConfigCatalog};
use crate::diagnostics::InvocationObserver;
use std::sync::Arc;

#[test]
fn scoped_max_annotation_lowers_only_its_runtime_closure_after_one_parse() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-scale")
        .canonicalize()
        .unwrap();
    let mut expected = None;
    for (config_path, batch, syntax_collections) in [
        ("configs/small-max.yml", false, 36),
        ("configs/small-max.yml", true, 36),
        ("configs/full-max.yml", false, 3076),
    ] {
        let observer = InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(Arc::clone(&observer)));
        let (config, plan, files, sources) = prepare(&session, &root, Some(config_path));
        assert!(config.audits_disable_directives());
        let facts = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            crate::ast::with_request_parse_cache(|| {
                let client = root.join("src/db.mts");
                session
                    .with_program(
                        &client,
                        &sources.read_source_path(&client).unwrap(),
                        |_, _| (),
                    )
                    .unwrap();
                let facts = if batch {
                    check_facts::collect_check_fact_batch_with_session(
                        &session,
                        vec![BatchCheckFactRequest {
                            root: root.clone(),
                            files: files.clone(),
                            graph_files: Vec::new(),
                            plan,
                            integration_discovery: check_facts::FamilyPathScope::All,
                            playwright: None,
                            sources: Arc::clone(&sources),
                        }],
                    )
                    .pop()
                    .unwrap()
                } else {
                    check_facts::collect_prepared_check_facts(
                        &session,
                        &root,
                        (files.clone(), Vec::new()),
                        Vec::new(),
                        plan,
                        None,
                        Arc::clone(&sources),
                    )
                };
                assert_eq!(crate::ast::request_parse_cache_len(), 0);
                facts
            })
        });
        let work = observer.snapshot().work;
        assert_eq!(work["postgres.annotation.profiles"], 1);
        assert_eq!(
            work["postgres.annotation.syntax_collections"],
            syntax_collections
        );
        if config_path == "configs/small-max.yml" {
            assert!(work["resolver.requests"] > 6000);
            assert!(work["resolver.computations"] < 20);
        }
        let attempts = session.work_snapshot().parse_attempts;
        assert_eq!(attempts.len(), 3076);
        assert!(attempts.values().all(|attempts| *attempts == 1));
        let findings = crate::codebase::rules::postgres_require_query_annotation::check_with_files_sources_and_facts(
            &root,
            &config,
            &files,
            &sources,
            &facts,
        ).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "src/module-0007.mts");
        if let Some(expected) = &expected {
            assert_eq!(&findings, expected);
        }
        expected = Some(findings);
    }
}

#[test]
fn scoped_max_annotation_preserves_an_isolated_lowering_panic_in_a_shared_batch() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-reachability");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let (_, plan, files, sources) = prepare(&session, &root, None);
    let helper = root.join("src/helper.mts");
    let request = |plan| BatchCheckFactRequest {
        root: root.clone(),
        files: files.clone(),
        graph_files: Vec::new(),
        plan,
        integration_discovery: check_facts::FamilyPathScope::All,
        playwright: None,
        sources: Arc::clone(&sources),
    };
    let graph = CheckFactPlan {
        graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    let maps =
        crate::codebase::fact_panic::tests::with_injected_panic_on_attempt(&helper, 2, || {
            crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
                check_facts::collect_check_fact_batch_with_session(
                    &session,
                    vec![request(plan), request(graph)],
                )
            })
        });
    let [annotation, graph] = maps.as_slice() else {
        panic!("annotation and graph requests");
    };
    let caller = root.join("src/root.mts");
    for facts in [annotation, graph] {
        let failed = &facts.ts[&helper];
        assert!(failed.fact_extraction_panicked);
        assert!(failed
            .parse_error
            .as_ref()
            .unwrap()
            .contains("injected fact extraction failure"));
        assert!(facts.ts[&caller].parsed);
    }
    assert!(graph.ts[&caller]
        .ts
        .imports
        .iter()
        .any(|import| import.specifier == "./cycle-a.mjs"));
    assert_eq!(observer.snapshot().work["postgres.annotation.roots"], 0);
    let attempts = session.work_snapshot().parse_attempts;
    assert_eq!(attempts.len(), files.len());
    assert!(attempts.values().all(|attempts| *attempts == 1));
}

#[test]
fn broad_and_unscoped_max_projects_keep_streaming_syntax_collection() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-reachability")
        .canonicalize()
        .unwrap();
    for unrestricted in [true, false] {
        let observer = InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(Arc::clone(&observer)));
        let (_, mut plan, files, sources) = prepare(&session, &root, None);
        if unrestricted {
            plan.query_annotation_scans = None;
            for scans in plan.embedded_sql_rule_ids.values_mut() {
                for scan in scans.iter_mut().filter(|scan| scan.annotation_seed) {
                    let mut broad = crate::codebase::postgres::EmbeddedSqlRuleScan::new(
                        &scan.rule_id,
                        &[],
                        &[],
                    )
                    .unwrap();
                    broad.annotation_seed = true;
                    *scan = broad;
                }
            }
        } else {
            plan.query_annotation_catalog = None;
        }
        let file_count = files.len() as u64;
        crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            check_facts::collect_prepared_check_facts(
                &session,
                &root,
                (files, Vec::new()),
                Vec::new(),
                plan,
                None,
                sources,
            )
        });
        assert_eq!(
            observer.snapshot().work["postgres.annotation.syntax_collections"],
            file_count
        );
    }
}

#[test]
fn scoped_max_annotation_marks_failed_helper_before_embedded_projection() {
    use crate::codebase::check_facts::sql_source_prerequisites::{SqlProjection, SqlSourceKey};
    let saved = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-required-predicates/variants/imported-barrel");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let mut config = crate::config::v2::NoMistakesConfig::default();
    for (rule, options) in [
        (
            "postgres-require-query-annotation",
            "include: ['src/caller.mts']\nimportSpecifier: '@example/db'\neffort: max",
        ),
        ("postgres-no-offset", "importSpecifier: '@example/db'"),
    ] {
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: rule.into(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        });
    }
    let (_, plan, files, sources) = prepare_config(&session, &root, config);
    let helper = root.join("src/helpers/index.mts");
    let facts =
        crate::codebase::fact_panic::tests::with_injected_panic_on_attempt(&helper, 2, || {
            crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
                check_facts::collect_prepared_check_facts(
                    &session,
                    &root,
                    (files, Vec::new()),
                    Vec::new(),
                    plan,
                    None,
                    sources,
                )
            })
        });
    assert!(facts.ts[&helper].fact_extraction_panicked);
    let caller = &facts.ts[&root.join("src/caller.mts")];
    assert!(caller.embedded_sql.iter().any(|(profile, _)| {
        let key = SqlSourceKey::profile(SqlProjection::Embedded, profile);
        caller
            .sql_source_prerequisites
            .failures(&key)
            .any(|path| path == &helper)
    }));
}

#[test]
fn scoped_max_retains_array_exposure_without_lowering_the_excluded_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-intrinsics")
        .canonicalize()
        .unwrap();
    let query = root.join("query.mts");
    let unrelated = root.join("unrelated.mts");
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let (_, plan, files, sources) = prepare(&session, &root, None);
    let facts = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        check_facts::collect_prepared_check_facts(
            &session,
            &root,
            (files, Vec::new()),
            Vec::new(),
            plan,
            None,
            sources,
        )
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.syntax_collections"], 1);
    assert_eq!(work["parse.files"], 2);
    assert!(facts.ts[&unrelated].query_annotation[0]
        .1
        .exposes_array_prototype());
    assert!(!facts.ts[&query].query_annotation[0]
        .1
        .exposes_array_prototype());
    let [(profile, embedded)] = facts.ts[&query].embedded_sql.as_slice() else {
        panic!("one embedded SQL executor profile");
    };
    let [call] = embedded.call_spans.as_slice() else {
        panic!("one configured query call");
    };
    assert_eq!(
        facts.query_annotation_prefix(&query, profile, *call),
        Some(&None)
    );
}

#[test]
fn max_annotation_seeds_intersect_rule_and_option_filters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-intrinsics")
        .canonicalize()
        .unwrap();
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.rules[0].options["include"] = serde_yaml::from_str("[unrelated.mts]").unwrap();
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let (_, plan, files, sources) = prepare_config(&session, &root, config);
    let facts = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        check_facts::collect_prepared_check_facts(
            &session,
            &root,
            (files, Vec::new()),
            Vec::new(),
            plan,
            None,
            sources,
        )
    });
    let work = observer.snapshot().work;
    assert_eq!(work["parse.files"], 2);
    assert!(!work.contains_key("postgres.annotation.syntax_collections"));
    assert_eq!(work["postgres.annotation.roots"], 0);
    assert!(facts.ts[&root.join("query.mts")].parsed);
}

#[test]
fn generic_annotation_include_keeps_legacy_executor_roots_for_failed_source_evidence() {
    use crate::codebase::check_facts::sql_source_prerequisites::{SqlProjection, SqlSourceKey};
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("scoped-source-prerequisite")
        .canonicalize()
        .unwrap();
    let caller = root.join("caller.mts");
    let broken = root.join("broken.mts");
    for (config, expected) in [("no-mistakes.yaml", true), ("options-control.yaml", false)] {
        let observer = InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(Arc::clone(&observer)));
        let (_, plan, files, sources) = prepare(&session, &root, Some(config));
        let facts = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            check_facts::collect_prepared_check_facts(
                &session,
                &root,
                (files, Vec::new()),
                Vec::new(),
                plan,
                None,
                sources,
            )
        });
        let caller = &facts.ts[&caller];
        let key =
            SqlSourceKey::profile(SqlProjection::AnnotationMax, &caller.query_annotation[0].0);
        assert_eq!(
            caller
                .sql_source_prerequisites
                .failures(&key)
                .any(|path| path == &broken),
            expected
        );
        assert_eq!(
            observer.snapshot().work["postgres.annotation.syntax_collections"],
            2
        );
    }
}

#[test]
fn merged_annotation_plans_keep_both_prepared_scopes_for_the_same_profile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-intrinsics")
        .canonicalize()
        .unwrap();
    let session = AnalysisSession::disabled();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let (_, mut merged, _, _) = prepare_config(&session, &root, config.clone());
    let mut second = config;
    second.rules[0].include = vec!["unrelated.mts".to_owned()];
    let (_, additional, _, _) = prepare_config(&session, &root, second);
    merged.include(additional);
    let [profile] = merged.query_annotation_options.as_ref().unwrap().as_slice() else {
        panic!("one executor profile across both plans");
    };
    let profile = profile.clone();
    let scans = &merged.annotation_scans()[&profile];
    for (path, expected) in [
        ("query.mts", true),
        ("unrelated.mts", true),
        ("other.mts", false),
    ] {
        assert_eq!(
            scans
                .iter()
                .any(|scan| scan.includes_annotation(&root.join(path), path)),
            expected
        );
    }
    merged.include(CheckFactPlan {
        query_annotation: true,
        ..Default::default()
    });
    assert!(merged.query_annotation_options.is_none());
    assert!(merged.annotation_scans()[&profile]
        .iter()
        .any(|scan| scan.includes_annotation(&root.join("other.mts"), "other.mts")));
}

fn prepare(session: &AnalysisSession, root: &Path, config_path: Option<&str>) -> Prepared {
    let config =
        crate::config::v2::load_v2_config(root, config_path.map(|path| root.join(path)).as_deref())
            .unwrap();
    prepare_config(session, root, config)
}

type Prepared = (
    crate::config::v2::NoMistakesConfig,
    CheckFactPlan,
    Vec<PathBuf>,
    Arc<crate::codebase::ts_source::SourceStore>,
);

fn prepare_config(
    session: &AnalysisSession,
    root: &Path,
    mut config: crate::config::v2::NoMistakesConfig,
) -> Prepared {
    let snapshot = session.visible_paths(root);
    let visible = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let files = visible
        .iter()
        .filter(|path| is_indexable(path))
        .cloned()
        .collect();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &visible,
        &sources,
    )
    .unwrap();
    let mut plan = CheckFactPlan::default();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    crate::codebase::postgres::configure_prepared_postgres_plan(root, &config, &mut plan).unwrap();
    plan.query_annotation_catalog = Some(Arc::new(TsConfigCatalog::from_visible_and_sources(
        root,
        &[root.to_path_buf()],
        &visible,
        &sources,
    )));
    (config, plan, files, sources)
}
