use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{ExecutionEvents, Value};
use crate::codebase::postgres::query_annotation::project::{merge, output::DEFERRED_REASON};

const ANNOTATED: &str = "/* name */ SELECT 1";

fn projected_call(events: Vec<Value>) -> Option<Option<String>> {
    let mut files = prepared_fixture("helper-tracing-event-prerequisites", &["sink"]);
    let path = files.keys().next().unwrap().clone();
    let profile = files[&path].postgres().query_annotation[0].0.clone();
    let span = files[&path].postgres().embedded_sql[0].1.call_spans[0];
    let events = ExecutionEvents::from([(
        (path.clone(), span),
        events.into_iter().map(|value| (false, value)).collect(),
    )]);
    let projection = merge::reduce_with_sources(events, false, Default::default());
    let originals = [(path.clone(), path.clone())].into_iter().collect();
    merge::apply(
        &mut files,
        &originals,
        &profile,
        projection,
        &mut Default::default(),
    );
    let facts = &files[&path].postgres().query_annotation[0].1;
    facts.calls.get(&span).cloned()
}

fn annotated() -> Value {
    Value::Prefix(ANNOTATED.to_string(), true, None)
}

#[test]
fn deferred_destructured_events_yield_to_proven_invocations() {
    let value = projected_call(vec![annotated(), Value::Failure(DEFERRED_REASON)]);
    assert_eq!(value, Some(Some(ANNOTATED.to_string())));
}

#[test]
fn only_deferred_destructured_events_keep_the_lexical_facts() {
    let before = projected_call(vec![]);
    let value = projected_call(vec![Value::Failure(DEFERRED_REASON)]);
    assert_eq!(value, before);
}

#[test]
fn unsupported_caller_events_still_defeat_proven_invocations() {
    let value = projected_call(vec![annotated(), Value::Unsupported]);
    assert_eq!(value, Some(None));
}
