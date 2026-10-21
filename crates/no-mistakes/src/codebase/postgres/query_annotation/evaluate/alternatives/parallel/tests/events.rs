use super::super::Outcome;
use super::*;

#[test]
fn fork_events_keep_flags_locations_order_and_translated_private_values() {
    let (path, _, file) = support::fixture("no-deletion");
    let facts = &file.query_annotation[0].1;
    let embedded = &file.embedded_sql[0].1;
    let files = support::files(&path, facts, embedded);
    let mut baseline = support::evaluator(&files);
    let root = baseline.module_environment(&path);
    let Expr::Sequence(parts) = &facts.globals["selected"] else {
        panic!("saved conditional factory")
    };
    let Expr::Alternatives(arms) = &parts[1] else {
        panic!("saved factory arms")
    };
    let key = (path.clone(), embedded.call_spans[0]);
    let (base_env, base_id) = (baseline.scopes.len(), baseline.next_builder);
    let mut outcomes = Vec::new();
    for (index, arm) in arms.iter().enumerate() {
        let mut fork = baseline.fork_arm();
        let returned = fork.expr(arm, &path, &root, 32, false);
        assert!(matches!(returned, Value::Function(_, _, env) if env == base_env));
        assert!(fork.argument_objects.contains_key(&base_id));
        let sql = format!("SELECT unsafe_arm_{index}");
        // Eligible factories currently emit no events. Inject typed fork state
        // to protect the protocol if that eligibility expands in the future.
        let value = Value::Aggregate(
            vec![
                returned.clone(),
                Value::Arguments(base_id),
                Value::Prefix(sql, true, Some(base_id)),
            ]
            .into(),
        );
        fork.events
            .insert(key.clone(), vec![(false, value.clone()), (true, value)]);
        outcomes.push(Outcome::take(fork, returned, base_env, base_id));
    }
    assert_eq!(outcomes.len(), 2);
    let mut target = baseline.fork_arm();
    target.scopes.extend((0..2).map(|_| Default::default()));
    target.next_builder += 7;
    target.events.insert(
        key.clone(),
        vec![(true, Value::Prefix("SELECT existing".into(), true, None))],
    );
    let mut expected = Vec::new();
    for outcome in outcomes {
        let (env, id) = (target.scopes.len(), target.next_builder);
        let returned = outcome.install(&mut target);
        assert!(matches!(returned, Value::Function(_, _, actual) if actual == env));
        expected.push((env, id));
    }
    assert_eq!(target.events.len(), 1, "preserve the saved source location");
    let events = &target.events[&key];
    assert_eq!(events.len(), 5, "append both arms after existing events");
    assert!(
        matches!(&events[0], (true, Value::Prefix(sql, true, None)) if sql == "SELECT existing")
    );
    for (index, (env, id)) in expected.into_iter().enumerate() {
        for (offset, flag) in [(0, false), (1, true)] {
            let (actual_flag, value) = &events[1 + index * 2 + offset];
            assert_eq!(*actual_flag, flag);
            let Value::Aggregate(values) = value else {
                panic!("all private event operands survive")
            };
            assert_eq!(values.len(), 3);
            assert!(matches!(&values[0], Value::Function(_, _, actual) if *actual == env));
            assert!(matches!(&values[1], Value::Arguments(actual) if *actual == id));
            assert!(matches!(&values[2], Value::Prefix(sql, true, Some(actual))
                if *actual == id && sql == &format!("SELECT unsafe_arm_{index}")));
            assert!(target.argument_objects.contains_key(&id));
        }
    }
}
