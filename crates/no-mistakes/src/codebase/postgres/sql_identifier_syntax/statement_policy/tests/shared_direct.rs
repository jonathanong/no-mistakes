use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::sql_identifier_syntax::literal_projection::ProjectedLiteral;
use crate::codebase::postgres::SqlObligationVerdict;
use crate::codebase::rules::{postgres_no_transformed_uuid_predicates, RuleFinding};
use std::sync::Arc;

const LEGACY: &str = "postgres-no-transformed-uuid-predicates";
const LEGACY_PARSES: &str = "postgres.no_transformed_uuid_predicates.low.literal_parses";
const SHARED_PARSES: &str = "postgres.low.policy_literal_parses";
struct Outcome {
    findings: Vec<RuleFinding>,
    verdicts: Vec<SqlObligationVerdict>,
    legacy_parses: u64,
    shared_parses: u64,
}
fn run(shared: bool) -> Outcome {
    let config_root = super::root();
    let mut config = crate::config::v2::load_v2_config(
        &config_root,
        Some(&config_root.join(".no-mistakes.yml")),
    )
    .unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(LEGACY),
    );
    for rule in &mut config.rules {
        let options = rule.options.as_mapping_mut().unwrap();
        options.insert("importSpecifier".into(), "db".into());
        options.insert(
            "include".into(),
            serde_yaml::to_value(["embedded.ts", "invalid.ts"]).unwrap(),
        );
        options.insert(
            "sqlInclude".into(),
            serde_yaml::to_value(Vec::<String>::new()).unwrap(),
        );
    }
    let mut legacy = config.rules[0].clone();
    legacy.rule = LEGACY.into();
    legacy.options.as_mapping_mut().unwrap().retain(|key, _| {
        matches!(
            key.as_str(),
            Some(
                "importSpecifier"
                    | "executorNames"
                    | "include"
                    | "sqlInclude"
                    | "schemaCatalogPath"
            )
        )
    });
    legacy
        .options
        .as_mapping_mut()
        .unwrap()
        .insert("unanalyzableSql".into(), "fail".into());
    if !shared {
        config.rules.clear();
    }
    config.rules.push(legacy);
    let files = ["embedded.ts", "invalid.ts", "schema.json"]
        .map(|name| root.join(name))
        .to_vec();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let mut plan =
        CheckFactPlan {
            postgres_schema_catalog_paths:
                crate::codebase::postgres::configured_schema_catalog_paths(&config, &[LEGACY])
                    .unwrap(),
            ..Default::default()
        };
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    assert!(
        plan.embedded_sql_options.is_empty(),
        "a max projection must not hide the shared low fallback"
    );
    let mut facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            plan,
            None,
            Arc::clone(&sources),
        );
    facts.enable_sql_verdicts();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        postgres_no_transformed_uuid_predicates::check_with_files_sources_and_facts(
            &root, &config, &files, &sources, &facts,
        )
        .unwrap()
    });
    let invalid = &facts.ts[&root.join("invalid.ts")].sql_identifier_syntax;
    assert!(
        invalid
            .transformed_literals
            .get()
            .unwrap()
            .values()
            .any(|literal| { matches!(literal, ProjectedLiteral::Direct(Err(()))) }),
        "invalid SQL must remain uncertain to the legacy consumer"
    );
    for name in ["embedded.ts", "invalid.ts"] {
        let file = &facts.ts[&root.join(name)];
        assert!(file.embedded_sql.is_empty());
        assert_eq!(
            file.sql_identifier_syntax.policy_direct.get().is_some(),
            shared
        );
    }
    let work = observer.snapshot().work;
    Outcome {
        findings,
        verdicts: facts.sql_verdicts(),
        legacy_parses: work.get(LEGACY_PARSES).copied().unwrap_or(0),
        shared_parses: work.get(SHARED_PARSES).copied().unwrap_or(0),
    }
}

#[test]
fn shared_direct_sql_preserves_legacy_uuid_findings_and_invalid_input_without_reparsing() {
    let legacy = run(false);
    let shared = run(true);
    assert_eq!(shared.findings, legacy.findings);
    assert_eq!(shared.verdicts, legacy.verdicts);
    assert!(shared
        .findings
        .iter()
        .any(|finding| finding.file == "embedded.ts"));
    assert!(shared
        .findings
        .iter()
        .any(|finding| finding.file == "invalid.ts"));
    assert!(legacy.legacy_parses > 0);
    assert_eq!(legacy.shared_parses, 0);
    assert_eq!(
        shared.legacy_parses, 0,
        "the old consumer must borrow the richer direct facts"
    );
    assert_eq!(shared.shared_parses, legacy.legacy_parses);
}
