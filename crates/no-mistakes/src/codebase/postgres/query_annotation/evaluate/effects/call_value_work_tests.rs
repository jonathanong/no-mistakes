use super::super::{Evaluator, File, Value};
use super::tests::evaluator;
use crate::codebase::postgres::query_annotation::{self, modules::Module};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::fx::FxHashMap;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

thread_local! {
    static REFRESH_VISITS: Cell<usize> = const { Cell::new(0) };
    static CALLBACK_VISITS: Cell<usize> = const { Cell::new(0) };
}

pub(in super::super) fn record_refresh_visit() {
    REFRESH_VISITS.set(REFRESH_VISITS.get() + 1);
}

pub(in super::super) fn record_callback_visits(count: usize) {
    CALLBACK_VISITS.set(CALLBACK_VISITS.get() + count);
}

type Resolve = fn(&str, &Path) -> Option<PathBuf>;

fn with_scalar_record(test: impl FnOnce(&mut Evaluator<'_, Resolve>, Value)) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/performance/annotation-shared-record-dag/dag-primitive-leaf/root.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &EmbeddedSqlOptions::default())
    })
    .unwrap();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::default())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = evaluator(&files, FxHashMap::default());
    let root = evaluator.module_environment(&path);
    let record = evaluator.scopes[root]["a12"].clone();
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
fn opaque_callbacks_skip_shared_scalar_records() {
    with_scalar_record(|evaluator, record| {
        assert!(matches!(record, Value::Record(..)), "saved shared record");
        let before = CALLBACK_VISITS.get();
        evaluator.opaque_callbacks(&[record], 16);
        assert_eq!(CALLBACK_VISITS.get() - before, 1);
    });
}
