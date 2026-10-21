use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{prepare_rule_sql_facts, EmbeddedSqlOptions};
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::{path::PathBuf, sync::Arc};

fn config(ids: &[&str]) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: ids
            .iter()
            .map(|id| RuleDef {
                rule: (*id).into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str("importSpecifier: '@example/db'").unwrap(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

#[test]
fn conflict_only_boundary_omits_unrequested_exact_location_families() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/embedded"),
    );
    let ts = root.join("conflict-location-demand.ts");
    let files = vec![ts.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let config = config(&["postgres-conflict-ordering"]);
    let prepared = prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &config,
        &["postgres-conflict-ordering"],
    )
    .unwrap();
    let embedded = prepared.embedded_sql(&ts, &profile).unwrap();
    assert!(embedded
        .calls
        .iter()
        .all(|call| call.variants.is_empty() && call.sql_source_positions.is_empty()));
    let statements = prepared.postgres_statements(&ts, Some(&profile)).unwrap();
    assert_eq!(statements.len(), 1);
    let locations = statements[0].variant_locations.as_ref().unwrap();
    assert_eq!(
        locations.call_index, 1,
        "irrelevant SELECT should not be prepared"
    );
    assert!(
        locations.locking.is_empty(),
        "conflict-only demand collected locking metadata"
    );
    assert!(locations
        .positions
        .keys()
        .all(|site| matches!(site, SqlFactSite::Origin | SqlFactSite::Conflict(_))));
    assert_eq!(
        statements[0]
            .variant_locations
            .as_ref()
            .unwrap()
            .conflicts
            .len(),
        1
    );
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn conflict_request_union_keeps_general_lock_and_legacy_demand_full() {
    use crate::codebase::check_facts::CheckFactPlan;
    use crate::codebase::postgres::configure_prepared_postgres_plan;
    let conflict = || {
        let mut plan = CheckFactPlan::default();
        configure_prepared_postgres_plan(&config(&["postgres-conflict-ordering"]), &mut plan)
            .unwrap();
        plan
    };
    let narrow = conflict();
    assert!(narrow.postgres_conflict_locations);
    assert!(!narrow.wants_full_sql_locations());
    for other in [
        CheckFactPlan {
            postgres_dml: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_variants: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_bounds: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_fragments: true,
            ..Default::default()
        },
        CheckFactPlan {
            query_annotation: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_exact_locations: true,
            ..Default::default()
        },
    ] {
        let mut first = conflict();
        first.include(other.clone());
        assert!(first.wants_full_sql_locations());
        let mut reverse = other;
        reverse.include(conflict());
        assert!(reverse.wants_full_sql_locations());
    }
    let mut repeated = conflict();
    repeated.include(conflict());
    assert!(!repeated.wants_full_sql_locations());
    for ids in [
        vec!["postgres-lock-ordering"],
        vec!["postgres-conflict-ordering", "postgres-required-predicates"],
        vec!["postgres-conflict-ordering", "postgres-lock-ordering"],
    ] {
        let mut plan = CheckFactPlan::default();
        configure_prepared_postgres_plan(&config(&ids), &mut plan).unwrap();
        assert!(plan.wants_full_sql_locations());
    }
    let mut general = CheckFactPlan::default();
    configure_prepared_postgres_plan(&config(&["postgres-required-predicates"]), &mut general)
        .unwrap();
    assert!(!general.wants_full_sql_locations());
}

#[test]
fn direct_and_shared_boundaries_preserve_full_for_mixed_consumers() {
    use crate::codebase::check_facts::{
        collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
    };
    use crate::codebase::postgres::configure_prepared_postgres_plan;
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/embedded"),
    );
    let ts = root.join("conflict-location-demand.ts");
    let files = vec![ts.clone()];
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    for ids in [
        vec!["postgres-lock-ordering"],
        vec!["postgres-conflict-ordering", "postgres-required-predicates"],
    ] {
        let config = config(&ids);
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let direct =
            prepare_rule_sql_facts(&root, &files, Arc::clone(&sources), &config, &ids).unwrap();
        let mut plan = CheckFactPlan::default();
        configure_prepared_postgres_plan(&config, &mut plan).unwrap();
        crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan)
            .unwrap();
        let shared = collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            vec![],
            plan,
            None,
            sources,
        );
        for facts in [&direct, &shared] {
            let statements = facts.postgres_statements(&ts, Some(&profile)).unwrap();
            assert!(!statements[0]
                .variant_locations
                .as_ref()
                .unwrap()
                .locking
                .is_empty());
        }
    }
}
