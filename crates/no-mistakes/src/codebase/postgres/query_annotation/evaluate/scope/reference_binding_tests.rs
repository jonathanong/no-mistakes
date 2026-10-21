use super::binding_scan_count::SCANS as REFERENCE_BINDING_SCANS;
use super::*;

fn saved_function() -> Value {
    use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-callback-profile/reads.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let query_annotation::Expr::Function(function) = facts.globals.remove("profile").unwrap()
    else {
        panic!("saved callback summary");
    };
    Value::Function(function, path, 0)
}

fn assert_matches_scan(scope: &Scope) {
    // Exact legacy map order matters: the indexed iterator must not sort names.
    let legacy = scope
        .iter()
        .filter(|(_, value)| value.contains_reference())
        .collect::<Vec<_>>();
    assert!(scope.reference_bindings().collect::<Vec<_>>() == legacy);
}

#[test]
fn reference_binding_index_scans_once_and_retains_only_reference_names() {
    REFERENCE_BINDING_SCANS.with(|count| count.set(0));
    let mut scope = Scope::from(FxHashMap::from_iter(
        (0..128).map(|n| (format!("scalar{n}"), Value::Primitive)),
    ));
    scope.insert("callback".into(), saved_function());
    scope.insert("args".into(), Value::Arguments(7));
    scope.insert("array".into(), Value::Array(8));
    scope.insert(
        "wrapped".into(),
        Value::Promise(Box::new(Value::Evaluated(
            Box::new(Value::Possible(
                vec![Value::Prefix("SELECT".into(), true, Some(9))].into(),
            )),
            true,
        ))),
    );
    let snapshot = scope.clone();
    let mut legacy_visits = 0;
    for arm in 0..16 {
        legacy_visits += scope.len();
        assert_matches_scan(if arm % 2 == 0 { &scope } else { &snapshot });
    }
    assert_eq!(legacy_visits, 132 * 16);
    assert_eq!(REFERENCE_BINDING_SCANS.with(|count| count.get()), 132);
    let names = scope.0.reference_binding_names.get().unwrap();
    assert_eq!(names.len(), 3);
    assert_eq!(names.iter().map(String::len).sum::<usize>(), 4 + 5 + 7);
    // Retained payload is 16 name bytes plus the small vector/String headers.
    assert!(names.iter().map(String::capacity).sum::<usize>() >= 16);
    assert!(names.capacity() >= 3);
    let retained_bytes = std::mem::size_of_val(&scope.0.reference_binding_names)
        + names.capacity() * std::mem::size_of::<String>()
        + names.iter().map(String::capacity).sum::<usize>();
    assert!(retained_bytes <= 192, "{retained_bytes}");
    eprintln!("reference-binding index retained bytes: {retained_bytes}");
    assert!(std::ptr::eq(
        names,
        snapshot.0.reference_binding_names.get().unwrap()
    ));
    // COW and uniquely owned writes must invalidate, including mutation iteration.
    scope.insert("args".into(), Value::Unknown);
    assert!(scope.0.reference_binding_names.get().is_none());
    assert_matches_scan(&scope);
    assert_eq!(snapshot.reference_bindings().count(), 3);
    for (_, value) in &mut scope {
        *value = Value::Unknown;
    }
    assert_matches_scan(&scope);
    assert_eq!(scope.reference_bindings().count(), 0);
    scope.insert("fresh".into(), Value::Arguments(10));
    assert_matches_scan(&scope);
    assert_eq!(scope.reference_bindings().count(), 1);
    scope.clear();
    assert_matches_scan(&scope);
}
