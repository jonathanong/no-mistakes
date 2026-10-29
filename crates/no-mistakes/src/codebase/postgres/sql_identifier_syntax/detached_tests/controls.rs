use super::*;

#[test]
fn detached_targets_do_not_cross_profiles_or_select_unrelated_bindings() {
    let unrelated = EmbeddedSqlOptions::configured("@unrelated/db", &[]);
    let no_query = EmbeddedSqlOptions::configured("@example/db", &["execute".into()]);
    fixture(
        "exact.test.ts",
        &[profile(), unrelated, no_query],
        |facts, _| {
            assert_eq!(
                facts.profiles[0]
                    .1
                    .iter()
                    .filter(|call| matches!(call.input, Input::Sql { .. }))
                    .count(),
                7
            );
            assert!(facts.profiles[1].1.is_empty());
            assert!(facts.profiles[2].1.is_empty());
        },
    );
    fixture("controls.test.ts", &[profile()], |facts, _| {
        assert!(facts.profiles[0].1.is_empty());
    });
}
#[test]
fn known_callable_exports_are_not_detached_receiver_seeds() {
    let query_only = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    fixture("callable-controls.test.ts", &[query_only], |facts, _| {
        assert!(facts.recovery.sinks[&facts.profiles[0].0].is_empty());
        assert!(facts.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
    });
    let custom =
        EmbeddedSqlOptions::configured("@example/db", &["query".into(), "customExecutor".into()])
            .with_scoped_executors(&["start".into()], &[]);
    fixture("../callable-profile.fixture.ts", &[custom], |facts, _| {
        assert!(facts.recovery.sinks[&facts.profiles[0].0].is_empty());
        assert!(facts.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
    });
}

#[test]
fn nested_and_non_identifier_shapes_do_not_gain_low_trusted_sinks() {
    fixture("../receiver-proofs.fixture.ts", &[profile()], |facts, _| {
        assert!(!facts.profiles[0].1.is_empty());
        assert!(facts.recovery.sinks[&profile()].is_empty());
        assert!(facts.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
    });
}

#[test]
fn binding_an_unselected_import_does_not_establish_an_sql_executor() {
    fixture("format-bind.test.ts", &[profile()], |facts, _| {
        assert!(facts.profiles[0].1.is_empty());
        assert!(facts.recovery.sinks[&profile()].is_empty());
    });
}
