use super::super::super::super::value::Values;
use super::super::translate::Translation;
use super::*;
#[test]
fn both_namespaces_translate_nested_values_without_losing_dag_sharing() {
    let (path, _, file) = support::fixture("no-deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let root = evaluator.module_environment(&path);
    let Value::Function(function, path, _) = evaluator.scopes[root]["pack"].clone() else {
        panic!()
    };
    let inner: Values = vec![
        Value::Arguments(7),
        Value::Array(7),
        Value::Prefix("SELECT 601".into(), true, Some(7)),
        Value::Function(function, path, 5),
    ]
    .into();
    let variants = vec![
        Value::Aggregate(inner.clone()),
        Value::References(inner.clone()),
        Value::Object(inner.clone()),
        Value::Record(inner.clone(), vec!["sql".into()]),
        Value::Joined(inner.clone()),
        Value::Possible(inner.clone()),
        Value::Promise(Box::new(Value::Arguments(7))),
        Value::Evaluated(Box::new(Value::Arguments(7)), true),
    ];
    let mut translation = Translation::new(5, 7, 8, 11);
    let mut identities = Vec::new();
    for variant in variants {
        match translation.value(variant) {
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                assert!(matches!(values[0], Value::Arguments(11)));
                assert!(matches!(values[1], Value::Array(11)));
                assert!(matches!(values[2], Value::Prefix(_, true, Some(11))));
                assert!(matches!(values[3], Value::Function(_, _, 8)));
                identities.push(values.identity());
            }
            Value::Promise(value) | Value::Evaluated(value, true) => {
                assert!(matches!(*value, Value::Arguments(11)))
            }
            _ => panic!("translated variant must retain its shape"),
        }
    }
    assert!(identities.iter().all(|identity| *identity == identities[0]));
    assert_ne!(identities[0], inner.identity());
    let baseline: Values = vec![Value::Arguments(1), Value::Primitive].into();
    assert!(translation
        .changed(&Value::Aggregate(baseline.clone()))
        .is_none());
    assert!(translation.changed(&Value::Aggregate(baseline)).is_none());
    let plain: Values = vec![Value::Number(1)].into();
    assert!(translation.changed(&Value::Object(plain)).is_none());
    let mixed: Values = vec![Value::Primitive, Value::Arguments(7), Value::Boolean(false)].into();
    let Value::Aggregate(mixed) = translation.value(Value::Aggregate(mixed)) else {
        panic!()
    };
    assert!(matches!(mixed[0], Value::Primitive));
    assert!(matches!(mixed[1], Value::Arguments(11)));
    assert!(matches!(mixed[2], Value::Boolean(false)));
    for unchanged in [
        Value::Promise(Box::new(Value::Primitive)),
        Value::Evaluated(Box::new(Value::Primitive), false),
    ] {
        assert!(translation.changed(&unchanged).is_none());
    }
    // Function-only DAGs have no object references, but still require frame translation.
    let functions_only: Values = vec![inner[3].clone()].into();
    let Value::Object(functions_only) = translation.value(Value::Object(functions_only)) else {
        panic!()
    };
    assert!(matches!(functions_only[0], Value::Function(_, _, 8)));
}
