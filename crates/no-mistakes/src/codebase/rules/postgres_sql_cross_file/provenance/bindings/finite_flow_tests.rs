use super::*;

#[test]
fn widened_binding_flows_follow_nested_awaits_and_reject_unproven_sources() {
    let file = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3501/binding-flow-controls.ts"),
    );
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let ts = collect_ts_facts(std::slice::from_ref(&file), plan)
        .into_iter()
        .next()
        .unwrap()
        .1;
    assert!(ts.parse_error.is_none());
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts).with_numeric_parameter_bounds();
    let ts = facts.get_ts_facts(&file).unwrap();
    for (name, expected) in [
        ("widenedAwaited", true),
        ("widenedNested", true),
        ("widenedLiteral", false),
        ("widenedMutable", false),
        ("widenedRecursive", false),
    ] {
        let binding = ts
            .value_flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Named {
            name: source,
            scope,
            offset,
            ..
        } = &binding.value
        else {
            panic!("{name}: {:?}", binding.value);
        };
        let mut seen = super::super::super::flow::Seen {
            bindings: fx_set(),
            calls: fx_set(),
        };
        assert_eq!(
            bindings.widened_finite_string_flow(
                &file,
                scope.unwrap_or(0),
                source,
                *offset,
                &mut seen
            ),
            expected,
            "{name}"
        );
    }
}
