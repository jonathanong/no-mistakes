use super::*;

fn plan(profile: &EmbeddedSqlOptions, head_only: bool) -> CheckFactPlan {
    CheckFactPlan {
        embedded_sql_options: vec![profile.clone()],
        embedded_sql_head_calls_only: head_only.then(|| profile.clone()).into_iter().collect(),
        ..Default::default()
    }
}

#[test]
fn full_profile_demand_wins_in_either_merge_order() {
    let a = EmbeddedSqlOptions::configured("@example/db", &[]);
    let b = EmbeddedSqlOptions::configured("@other/db", &[]);
    for full_first in [false, true] {
        let mut combined = plan(&a, !full_first);
        combined.include(plan(&a, full_first));
        combined.include(plan(&b, true));
        assert_eq!(combined.embedded_sql_head_calls_only, vec![b.clone()]);
    }
}

#[test]
fn unrelated_and_duplicate_requests_keep_head_only_demand() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let mut combined = plan(&profile, true);
    combined.include(CheckFactPlan::default());
    combined.include(combined.clone());
    assert_eq!(combined.embedded_sql_head_calls_only, vec![profile]);
}

#[test]
fn implicit_default_profile_demands_full_recovery() {
    let profile = EmbeddedSqlOptions::default();
    let mut combined = plan(&profile, true);
    combined.include(CheckFactPlan {
        embedded_sql: true,
        ..Default::default()
    });
    assert!(combined.embedded_sql_head_calls_only.is_empty());
}

#[test]
fn unspecified_direct_or_evaluator_demand_cannot_be_downgraded() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    for extra in [
        CheckFactPlan {
            query_annotation: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_dml: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_fragments: true,
            ..Default::default()
        },
        CheckFactPlan {
            postgres_variants: true,
            ..Default::default()
        },
    ] {
        let mut combined = plan(&profile, true);
        combined.include(extra);
        assert!(combined.embedded_sql_head_calls_only.is_empty());
    }
}

#[test]
fn explicit_annotation_profile_scope_only_promotes_the_selected_profile() {
    let selected = EmbeddedSqlOptions::configured("@example/db", &[]);
    let other = EmbeddedSqlOptions::configured("@other/db", &[]);
    let query_annotation = |profile| CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![profile]),
        ..Default::default()
    };

    let mut matching = plan(&selected, true);
    matching.include(query_annotation(selected.clone()));
    assert!(matching.embedded_sql_head_calls_only.is_empty());

    let mut unrelated = plan(&selected, true);
    unrelated.include(query_annotation(other));
    assert_eq!(unrelated.embedded_sql_head_calls_only, vec![selected]);
}
