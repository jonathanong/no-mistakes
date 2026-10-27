use super::*;
use crate::codebase::check_facts::{CheckFactPlan, SqlFactContext};
use crate::codebase::postgres::{EmbeddedSqlKind, SqlVerdictStatus};
use std::{path::PathBuf, sync::Arc};

const EXPLICIT: &str = "postgres-explicit-columns";
const REQUIRED: &str = "postgres-required-predicates";
fn root() -> PathBuf {
    // Resolver targets use normalized inventory identities, including imported helper files.
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(
        "postgres-low-policy",
    ))
}
fn prepare(mixed: bool) -> (CheckFactMap, crate::config::v2::NoMistakesConfig) {
    prepare_mode(mixed, Policies::Pair)
}
enum Policies {
    Pair,
    LegacyOnly,
    PairWithLegacy,
}
fn prepare_mode(
    mixed: bool,
    policies: Policies,
) -> (CheckFactMap, crate::config::v2::NoMistakesConfig) {
    let root = root();
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    match policies {
        Policies::LegacyOnly => {
            config.rules.truncate(1);
            config.rules[0].rule = "postgres-no-offset".into();
        }
        Policies::PairWithLegacy => {
            let mut legacy = config.rules[0].clone();
            legacy.rule = "postgres-no-offset".into();
            config.rules.push(legacy);
        }
        Policies::Pair => {}
    }
    if mixed {
        let mut max = config.rules[0].clone();
        max.options
            .as_mapping_mut()
            .unwrap()
            .insert("effort".into(), "max".into());
        config.rules.push(max);
    }
    let files = [
        "src/query.mts",
        "src/helpers.mts",
        "src/malformed.mts",
        "schema.sql",
        "tsconfig.json",
    ]
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
    let mut plan = CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    if !mixed {
        assert!(plan.embedded_sql_options.is_empty());
    }
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files,
            Vec::new(),
            plan,
            None,
            sources,
        );
    (facts, config)
}
fn profile() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &["query".into()])
}

#[test]
fn policy_context_shares_parsing_and_preserves_raw_sql_default_access() {
    let (facts, _) = prepare(false);
    let path = root().join("src/query.mts");
    assert!(facts.ts[&path].embedded_sql.is_empty());
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (first, second) = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        let explicit = facts
            .for_sql_application_with_advice(0, None)
            .with_low_sql(EXPLICIT);
        let required = facts
            .for_sql_application_with_advice(0, None)
            .with_low_sql(REQUIRED);
        let first = explicit
            .postgres_statements(&path, Some(&profile()))
            .unwrap()
            .len();
        let parses = observer.snapshot().work;
        let second = required
            .postgres_statements(&path, Some(&profile()))
            .unwrap()
            .len();
        assert_eq!(
            observer.snapshot().work,
            parses,
            "the second consumer must not parse SQL again"
        );
        let raw = root().join("schema.sql");
        assert!(std::ptr::eq(
            explicit.postgres_statements(&raw, None).unwrap(),
            facts.postgres_statements(&raw, None).unwrap()
        ));
        (first, second)
    });
    assert!(first > 0);
    assert_eq!(first, second);
    assert!(
        observer
            .snapshot()
            .work
            .get("postgres.low.recovered_parses")
            .copied()
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn unknown_and_invalid_inputs_never_hide_known_stars_or_physical_origins() {
    let (facts, _) = prepare(false);
    let path = root().join("src/query.mts");
    let context = facts
        .for_sql_application_with_advice(0, Some("low"))
        .with_low_sql(EXPLICIT);
    let embedded = context.embedded_sql(&path, &profile()).unwrap();
    assert!(embedded.calls.iter().any(|call| call.is_unanalyzable()));
    assert!(
        embedded
            .calls
            .iter()
            .any(|call| call.sql_text.as_deref() == Some("") && !call.is_unanalyzable()),
        "an unused executor transport proves nonexecution"
    );
    let statements = context
        .postgres_statements(&path, Some(&profile()))
        .unwrap();
    assert!(statements
        .iter()
        .any(|file| file.parse_failed && !file.selects.is_empty()));
    assert!(statements
        .iter()
        .any(
            |file| file.variant_locations.as_ref().is_some_and(|locations| {
                locations.positions.values().any(|position| {
                    position
                        .source_path
                        .as_ref()
                        .is_some_and(|path| path.ends_with("helpers.mts"))
                })
            })
        ));
    assert!(embedded
        .calls
        .iter()
        .any(|call| call.kind == EmbeddedSqlKind::Dynamic && call.sql_text.is_none()));
}

#[test]
fn mixed_max_and_application_profile_mismatches_keep_low_results_isolated() {
    let (low, _) = prepare(false);
    let (mixed, _) = prepare(true);
    let path = root().join("src/query.mts");
    let low_context = low
        .for_sql_application_with_advice(0, None)
        .with_low_sql(EXPLICIT);
    let mixed_context = mixed
        .for_sql_application_with_advice(0, None)
        .with_low_sql(EXPLICIT);
    assert_eq!(
        low_context.embedded_sql(&path, &profile()).unwrap(),
        mixed_context.embedded_sql(&path, &profile()).unwrap()
    );
    let wrong = mixed
        .for_sql_application_with_advice(1, None)
        .with_low_sql(EXPLICIT);
    assert!(wrong.embedded_sql(&path, &profile()).is_err());
    assert!(low_context
        .embedded_sql(&path, &EmbeddedSqlOptions::configured("other-db", &[]))
        .is_err());
    assert!(low_context
        .embedded_sql(&root().join("schema.sql"), &profile())
        .is_err());
    assert!(!mixed.for_sql_application_with_advice(1, None).is_low_sql());
}

#[test]
fn low_rewrite_reaches_both_unconditional_advice_and_verdict_sinks() {
    let (mut facts, _) = prepare(false);
    facts.enable_sql_verdicts();
    let context = facts
        .for_sql_application_with_advice(0, Some("low"))
        .with_low_sql(EXPLICIT);
    assert!(context
        .sql_unanalyzable_message("unknown")
        .contains("effort low"));
    let verdict = crate::codebase::postgres::SqlObligationVerdict {
        profile: profile(),
        rule: EXPLICIT.into(),
        application: None,
        subcheck: "stars".into(),
        file: "src/query.mts".into(),
        span: (0, 1),
        status: SqlVerdictStatus::Abstain,
        advisory: false,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec!["unknown-sql".into()],
    };
    context.record_sql_verdict(verdict.clone());
    context.record_sql_advisory(verdict);
    assert_eq!(facts.sql_verdicts().len(), 2);
    assert!(facts.sql_verdicts().iter().all(|verdict| verdict
        .residual_reasons
        .iter()
        .any(|reason| reason.contains("effort low"))));
    assert_eq!(
        facts
            .for_sql_application_with_advice(0, None)
            .sql_unanalyzable_message("unknown"),
        "unknown"
    );
}

#[test]
fn low_only_demand_never_builds_the_max_embedded_projection() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _ = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || prepare(false));
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.prepared.statement_projections")
            .copied()
            .unwrap_or(0),
        0
    );
}

#[test]
fn direct_helper_argument_suppression_keeps_exact_caller_line() {
    let (facts, _) = prepare(false);
    let path = root().join("src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let line = source
        .lines()
        .position(|line| line.contains("SUPPRESSED_HELPER_ARGUMENT"))
        .unwrap() as u32
        + 1;
    let context = facts
        .for_sql_application_with_advice(0, None)
        .with_low_sql(EXPLICIT);
    assert!(context
        .postgres_statements(&path, Some(&profile()))
        .unwrap()
        .iter()
        .any(|statement| {
            statement
                .variant_locations
                .as_ref()
                .is_some_and(|locations| locations.caller_lines == vec![line])
        }));
}

#[test]
fn existing_low_policies_keep_the_original_single_cache_shape() {
    let (facts, _) = prepare_mode(false, Policies::LegacyOnly);
    let path = root().join("src/query.mts");
    let file = &facts.ts[&path];
    assert!(file.sql_identifier_syntax.policy_projection.get().is_none());
    assert!(!super::super::literal_projection::recovered_statements(file).is_empty());
    assert!(file.sql_identifier_syntax.recovered_raw.get().is_none());
    assert!(file.sql_identifier_syntax.policy_direct.get().is_none());
}

#[test]
fn mixed_low_policies_share_raw_parses_without_clearing_invalid_legacy_inputs() {
    for legacy_first in [false, true] {
        let (facts, _) = prepare_mode(false, Policies::PairWithLegacy);
        let path = root().join("src/query.mts");
        let file = &facts.ts[&path];
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            if legacy_first {
                super::super::literal_projection::recovered_statements(file);
            }
            let context = facts
                .for_sql_application_with_advice(0, None)
                .with_low_sql(EXPLICIT);
            let statements = context
                .postgres_statements(&path, Some(&profile()))
                .unwrap();
            assert!(statements
                .iter()
                .any(|statement| statement.parse_failed && !statement.selects.is_empty()));
            let parses = observer
                .snapshot()
                .work
                .get("postgres.low.recovered_parses")
                .copied()
                .unwrap_or(0);
            let legacy = super::super::literal_projection::recovered_statements(file);
            assert_eq!(
                observer
                    .snapshot()
                    .work
                    .get("postgres.low.recovered_parses")
                    .copied()
                    .unwrap_or(0),
                parses
            );
            assert!(parses > 0);
            let key = super::super::helper_recovery::ApplicationKey::new(
                &profile(),
                "postgres-no-offset",
                0,
            );
            let raw = super::super::literal_projection::recovered::raw_statements(file);
            let invalid = raw
                .iter()
                .filter(|((application, _), _)| application == &key)
                .flat_map(|(key, rows)| {
                    rows.iter()
                        .enumerate()
                        .filter(|(_, row)| row.parse_failed)
                        .map(move |(index, _)| (key, index))
                })
                .collect::<Vec<_>>();
            assert!(!invalid.is_empty());
            assert!(invalid
                .into_iter()
                .all(|(key, index)| legacy[key][index].is_err()));
        });
    }
}

#[test]
fn malformed_owner_retains_parse_failure_without_trusting_recovered_ast() {
    let (facts, _) = prepare(false);
    let path = root().join("src/malformed.mts");
    let file = &facts.ts[&path];
    assert!(file.parse_error.is_some() || file.ts.parse_error.is_some());
    let context = facts
        .for_sql_application_with_advice(0, None)
        .with_low_sql(EXPLICIT);
    assert!(context
        .embedded_sql(&path, &profile())
        .unwrap()
        .calls
        .is_empty());
    assert!(context
        .postgres_statements(&path, Some(&profile()))
        .unwrap()
        .is_empty());
}

#[test]
fn opaque_helper_and_unknown_carrier_keep_residual_uncertainty() {
    let (mut facts, _) = prepare(false);
    facts.enable_sql_verdicts();
    let path = root().join("src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let context = facts
        .for_sql_application_with_advice(0, None)
        .with_low_sql(EXPLICIT);
    let embedded = context.embedded_sql(&path, &profile()).unwrap();
    let opaque_line = source
        .lines()
        .position(|line| line.contains("OPAQUE_RETURN"))
        .unwrap() as u32
        + 1;
    assert!(embedded
        .calls
        .iter()
        .any(|call| call.line == opaque_line && call.is_unanalyzable()));
    let key = super::super::helper_recovery::ApplicationKey::new(&profile(), EXPLICIT, 0);
    assert!(facts.ts[&path].sql_identifier_syntax.calls_for(&key).unwrap().iter().any(|call| {
        matches!(&call.input, super::super::Input::Recovered(proof) if proof.alternatives.is_empty() && !proof.complete())
    }));
    let statements = context
        .postgres_statements(&path, Some(&profile()))
        .unwrap();
    crate::codebase::postgres::record_sql_policy_verdicts(
        crate::codebase::postgres::SqlPolicyVerdictContext {
            facts: &context,
            embedded,
            statements,
        },
        EXPLICIT,
        "src/query.mts",
        &profile(),
        &["stars".into()],
        true,
        |_, _, _| crate::codebase::postgres::SqlBranchJudgment::clear(),
    );
    let carrier_line = source
        .lines()
        .position(|line| line.contains("OPAQUE_CARRIER"))
        .unwrap() as u32
        + 1;
    for line in [opaque_line, carrier_line] {
        let span = embedded
            .calls
            .iter()
            .position(|call| call.line == line)
            .map(|index| embedded.call_spans[index])
            .unwrap();
        assert!(facts
            .sql_verdicts()
            .iter()
            .any(|row| row.span == span && row.status == SqlVerdictStatus::Abstain));
    }
}
