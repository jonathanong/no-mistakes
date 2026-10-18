use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Fixture {
    reuse_count: usize,
    path: PathBuf,
    start: u32,
    parameter: String,
    labels: Vec<String>,
}

#[test]
fn revisit_order_uses_allocation_birth_after_reuse_and_capacity_growth() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-frame-order.json");
    let fixture: Fixture = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let mut scopes = ScopeArena::default();
    for _ in 0..fixture.reuse_count {
        let disposable = scopes.allocate(Scope::default());
        assert!(scopes.release(disposable).is_some());
    }
    let first = scopes.allocate(Scope::default());
    let second = scopes.allocate(Scope::default());
    assert!(scopes.birth(first) < scopes.birth(second));
    assert!(
        first > second,
        "the saved reuse history reverses raw handle order"
    );
    let identity = |env| {
        (
            fixture.path.clone(),
            fixture.start,
            env,
            vec![fixture.parameter.clone()],
        )
    };
    let mut callbacks = vec![
        (
            identity(second),
            8,
            Value::Prefix(fixture.labels[1].clone(), true, None),
        ),
        (
            identity(first),
            8,
            Value::Prefix(fixture.labels[0].clone(), true, None),
        ),
    ];
    // Invoke the same helper as the live scheduler, not a copied comparator.
    sort(&mut callbacks, &scopes);
    let order = callbacks
        .iter()
        .map(|(key, _, _)| key.2)
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        vec![first, second],
        "live callback effects follow original allocation order"
    );
    assert!(
        matches!(&callbacks[0].2, Value::Prefix(text, true, None) if text == &fixture.labels[0])
    );
    assert!(
        matches!(&callbacks[1].2, Value::Prefix(text, true, None) if text == &fixture.labels[1])
    );
}
