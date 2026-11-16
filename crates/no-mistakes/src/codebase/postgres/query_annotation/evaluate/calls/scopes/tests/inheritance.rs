use super::super::{arguments, bind_this, locals};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashMap;
use query_annotation::{
    evaluate::{references::Edges, Scope, Value},
    Function, Step,
};
use std::path::PathBuf;

#[path = "legacy.rs"]
mod legacy;

fn saved(name: &str) -> (Scope, Vec<Function>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@bench/db", &[]),
        )
    })
    .unwrap();
    let functions = facts
        .globals
        .values()
        .filter_map(|expr| {
            if let query_annotation::Expr::Function(function) = expr {
                Some((**function).clone())
            } else {
                None
            }
        })
        .collect();
    let captured = facts
        .globals
        .into_iter()
        .map(|(name, value)| {
            let value = match value {
                query_annotation::Expr::Function(function) => {
                    Value::Function(function, path.clone().into(), 0)
                }
                _ => Value::Unknown,
            };
            (name, value)
        })
        .collect::<FxHashMap<_, _>>()
        .into();
    (captured, functions)
}

fn bind(scope: &mut Scope, function: &Function, argument: Value, receiver: Option<&Value>) {
    arguments(scope, function, argument);
    for (index, name) in function.params.iter().enumerate() {
        scope.insert(name.clone(), Value::Number(index as i64));
    }
    bind_this(scope, function, receiver);
}

#[test]
fn inherited_locals_match_raw_maps_for_shadows_and_lexical_arguments_and_this() {
    let (mut captured, functions) = saved("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-shadow-index/capture-32.cjs");
    captured.insert("arguments".into(), Value::Arguments(7));
    captured.insert("this".into(), Value::Array(8));
    let receiver = Value::Arguments(9);
    for arrow in [false, true] {
        let mut function = functions[0].clone();
        function.arrow = arrow;
        for name in &function.params {
            captured.insert(name.clone(), Value::Primitive);
        }
        let initial = captured.clone();
        let mut expected = legacy::locals(&captured, &function);
        let mut actual = locals(&captured, &function);
        legacy::arguments(&mut expected, &function, Value::Arguments(10));
        for (index, name) in function.params.iter().enumerate() {
            expected.insert(name.clone(), Value::Number(index as i64));
        }
        legacy::bind_this(&mut expected, &function, Some(&receiver));
        bind(
            &mut actual,
            &function,
            Value::Arguments(10),
            Some(&receiver),
        );
        assert!(*actual == expected);
        let raw = Edges::collect(expected.values());
        assert_eq!(actual.references().environments, raw.environments);
        assert_eq!(actual.references().arguments, raw.arguments);
        assert_eq!(actual.references().identities, raw.identities);
        actual.insert("capture0".into(), Value::Number(99));
        assert!(captured == initial);
        assert!(actual["capture0"] != captured["capture0"]);
    }
}

#[test]
fn reused_large_capture_frames_share_direct_summaries_and_reduce_scans() {
    let run = |optimized: bool| {
        let (captured, functions) =
            saved("../../fixtures/performance/annotation-excluded-callers-large/src/helper.mts");
        assert!(functions.len() >= 160);
        let observer = InvocationObserver::new(true);
        let mut frames = Vec::new();
        with_observer(Some(observer.clone()), || {
            for function in functions.iter().take(16) {
                let frame = if optimized {
                    let mut frame = locals(&captured, function);
                    bind(&mut frame, function, Value::Arguments(7), None);
                    frame
                } else {
                    let mut frame = legacy::locals(&captured, function);
                    legacy::arguments(&mut frame, function, Value::Arguments(7));
                    for (index, name) in function.params.iter().enumerate() {
                        frame.insert(name.clone(), Value::Number(index as i64));
                    }
                    legacy::bind_this(&mut frame, function, None);
                    frame.into()
                };
                frame.references();
                frames.push(frame);
            }
        });
        (
            frames,
            observer.snapshot().work["postgres.annotation.prefix_binding_scans"],
        )
    };
    let (raw, raw_scans) = run(false);
    let (cached, cached_scans) = run(true);
    assert!(raw == cached);
    assert!(
        raw_scans >= 16 * cached_scans,
        "{raw_scans} vs {cached_scans}"
    );
}
