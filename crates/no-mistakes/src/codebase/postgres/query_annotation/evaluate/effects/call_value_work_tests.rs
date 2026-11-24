use super::super::{Evaluator, File, Value};
use super::hooks::{CALLBACK_VISITS, REFRESH_VISITS};
use super::tests::evaluator;
use crate::codebase::postgres::query_annotation::{self, modules::Module};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Resolve = fn(&str, &Path) -> Option<PathBuf>;

fn with_scalar_record(test: impl FnOnce(&mut Evaluator<'_, Resolve>, Value)) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/performance/annotation-shared-record-dag/dag-primitive-leaf/root.mts",
    );
    with_record(&path, "a12", test);
}

fn with_record(path: &Path, binding: &str, test: impl FnOnce(&mut Evaluator<'_, Resolve>, Value)) {
    let source = std::fs::read_to_string(path).unwrap();
    let facts = crate::ast::with_program(path, &source, |program, _| {
        query_annotation::collect(program, &source, &EmbeddedSqlOptions::default())
    })
    .unwrap();
    let files = FxHashMap::from_iter([(
        path.to_path_buf(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::default())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = evaluator(&files, FxHashMap::default());
    let root = evaluator.module_environment(path);
    let record = evaluator.scopes[root][binding].clone();
    test(&mut evaluator, record);
}

#[test]
fn argument_refresh_skips_shared_scalar_records() {
    with_scalar_record(|evaluator, record| {
        let Value::Record(parts, _) = &record else {
            panic!("saved shared record");
        };
        let mut arguments = [record.clone()];
        let before = REFRESH_VISITS.get();
        evaluator.refresh_arguments(&mut arguments);
        assert_eq!(REFRESH_VISITS.get() - before, 0, "no builder changes");

        evaluator.invalidated_builders.insert(0);
        let before = REFRESH_VISITS.get();
        evaluator.refresh_arguments(&mut arguments);
        assert_eq!(REFRESH_VISITS.get() - before, 1, "unrelated builder change");
        let Value::Record(refreshed, _) = &arguments[0] else {
            panic!("record survives refresh");
        };
        assert_eq!(refreshed.identity(), parts.identity());
    });
}

#[test]
fn argument_refresh_clears_only_the_invalidated_nested_builder() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("argument-refresh-builders.mts");
    with_record(&path, "argument", |evaluator, record| {
        let Value::Record(parts, _) = &record else {
            panic!("saved argument record");
        };
        let [Value::Record(nested, _), retained @ Value::Prefix(_, _, Some(retained_id))] =
            &parts[..]
        else {
            panic!("nested record and retained builder");
        };
        let [invalidated @ Value::Prefix(_, _, Some(invalidated_id))] = &nested[..] else {
            panic!("nested builder before invalidation");
        };
        assert_ne!(invalidated_id, retained_id);
        evaluator.invalidate_builders(std::slice::from_ref(invalidated));

        let mut arguments = [record.clone()];
        evaluator.refresh_arguments(&mut arguments);

        let [Value::Record(refreshed, _)] = &arguments else {
            panic!("argument stays a record");
        };
        let [Value::Record(nested, _), current] = &refreshed[..] else {
            panic!("argument record keeps both fields");
        };
        assert!(matches!(&nested[..], [Value::Unknown]));
        assert!(current == retained, "unrelated builder survives refresh");
    });
}

#[test]
fn opaque_callbacks_skip_shared_scalar_records() {
    with_scalar_record(|evaluator, record| {
        assert!(matches!(record, Value::Record(..)), "saved shared record");
        let before = CALLBACK_VISITS.get();
        evaluator.opaque_callbacks(&[record], 16);
        assert_eq!(CALLBACK_VISITS.get() - before, 1);
    });
}
