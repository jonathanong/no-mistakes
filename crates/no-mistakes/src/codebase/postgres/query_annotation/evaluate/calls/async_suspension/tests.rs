
use super::*;
use crate::codebase::postgres::query_annotation::DeleteKey;

#[test]
fn suspension_discards_only_events_created_after_the_await() {
    let key = (PathBuf::from("builder.ts"), (2, 9));
    let earlier = Value::Prefix("SELECT id FROM posts".into(), true, None);
    let mut events = ExecutionEvents::from([(key.clone(), vec![(false, earlier.clone())])]);
    let before = event_lengths(&events);
    events.get_mut(&key).unwrap().push((
        false,
        Value::Prefix("SELECT secret FROM posts".into(), true, None),
    ));
    invalidate_new_events(&mut events, &before);
    assert!(events[&key][0].1 == earlier);
    assert!(matches!(events[&key][1].1, Value::Unknown));
}

#[test]
fn nested_await_in_sql_builder_mutations_is_a_suspension() {
    let awaited = || Expr::Await(Box::new(Expr::Text("ready".into())));
    let conditional_index = Expr::Sequence(vec![Expr::Alternatives(vec![
        Expr::ComputedMember(Box::new(Expr::Name("query".into())), Box::new(awaited())),
        Expr::Primitive,
    ])]);
    assert!(has_suspension(&[Step::Append(
        "query".into(),
        conditional_index
    )]));

    let slot_write = Expr::SlotWrite {
        receiver: Box::new(Expr::Name("query".into())),
        index: 0,
        value: Box::new(Expr::Call {
            callee: Box::new(Expr::Name("prepare".into())),
            args: vec![awaited()],
            start: 0,
            end: 0,
        }),
    };
    assert!(has_suspension(&[Step::Effect(slot_write)]));

    let delete = Expr::Delete(
        vec![Expr::Tagged("String.raw".into(), vec![awaited()], vec![])],
        DeleteKey::Dynamic,
    );
    assert!(has_suspension(&[Step::Effect(delete)]));
    assert!(!has_suspension(&[
        Step::Var("query".into()),
        Step::Return(Expr::Text("SELECT 1".into()))
    ]));
}
