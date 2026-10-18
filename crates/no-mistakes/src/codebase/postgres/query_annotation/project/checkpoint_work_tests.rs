use super::project_executed;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::query_annotation;
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::{
    facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
    FileIdMap,
};
use crate::diagnostics::{with_observer, InvocationObserver};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn prepared(names: &[&str]) -> (PathBuf, FileIdMap<CheckFileFacts>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-alternative-checkpoints")
        .canonicalize()
        .unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
    let files = names
        .iter()
        .map(|name| {
            let path = root.join(name).canonicalize().unwrap();
            let source = std::fs::read_to_string(&path).unwrap();
            let file = crate::ast::with_program(&path, &source, |program, _| {
                let mut annotation = query_annotation::collect(program, &source, &options);
                // Match the precise-records path used by the current query consumer.
                annotation.precise_records = true;
                CheckFileFacts {
                    ts: Arc::new(collect_file_facts_from_program(
                        &path,
                        TsFactPlan {
                            imports: true,
                            module_bindings: true,
                            ..Default::default()
                        },
                        &TsFactContext::default(),
                        &source,
                        program,
                        None,
                        None,
                    )),
                    embedded_sql: vec![(
                        options.clone(),
                        extract_embedded_sql_from_program(&path, program, &source, &options),
                    )],
                    query_annotation: vec![(options.clone(), annotation)],
                    source: Some(source.clone().into()),
                    ..Default::default()
                }
            })
            .unwrap();
            (path, file)
        })
        .collect::<Vec<_>>();
    let entry = root.join(names[0]).canonicalize().unwrap();
    (entry, FileIdMap::from_iter(files))
}

fn run(
    names: &[&str],
) -> (
    Option<String>,
    Result<Vec<String>, &'static str>,
    crate::diagnostics::DiagnosticsSnapshot,
) {
    let (path, mut files) = prepared(names);
    let candidates = files
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let observer = InvocationObserver::new(true);
    let mut calls = with_observer(Some(Arc::clone(&observer)), || {
        project_executed(&mut files, |specifier: &str, from: &Path| {
            from.parent()?
                .join(specifier)
                .canonicalize()
                .ok()
                .filter(|path| candidates.contains(path))
        })
    });
    let annotation_calls = &files[&path].query_annotation[0].1.calls;
    assert_eq!(annotation_calls.len(), 1, "one configured query per entry");
    let (span, annotation) = annotation_calls.iter().next().unwrap();
    let span = *span;
    let annotation = annotation.clone();
    let expected = calls
        .remove(&(path.clone(), span))
        .expect("the saved fixture has one configured query call")
        .map(|items| items.into_iter().map(|item| item.text).collect());
    assert_eq!(files.len(), names.len());
    assert!(files.contains_key(&path));
    (annotation, expected, observer.snapshot())
}

fn work(snapshot: &crate::diagnostics::DiagnosticsSnapshot, key: &str) -> u64 {
    snapshot.work.get(key).copied().unwrap_or(0)
}

#[test]
fn untouched_state_size_does_not_change_transaction_work() {
    let (small_call, small_sql, small) = run(&["state-small.mts"]);
    let (large_call, large_sql, large) = run(&["state-large.mts"]);
    assert!(matches!(
        small_call.as_deref(),
        Some("SELECT 1 /* left */" | "SELECT 2 /* right */")
    ));
    assert_eq!(large_call, small_call);
    assert_eq!(small_sql, Err("unresolved-value-or-effect"));
    assert_eq!(large_sql, small_sql);
    let copied_key = "postgres.annotation.incoming_dense_slots_copied";
    let small_copied = work(&small, copied_key);
    let large_copied = work(&large, copied_key);
    assert!(
        large_copied == small_copied,
        "untouched dense width should not add copies (the previous baseline grew from {small_copied} to {large_copied})"
    );
    for key in [
        "postgres.annotation.undo_entries_recorded",
        "postgres.annotation.undo_values_cloned",
        "postgres.annotation.undo_entries_restored",
        "postgres.annotation.joined_dense_locations",
    ] {
        assert!(
            work(&small, key) > 0,
            "checkpoint work is observable: {key}"
        );
        assert_eq!(work(&large, key), work(&small, key), "{key}");
    }
}

fn mapped(name: &str) -> (Vec<String>, crate::diagnostics::DiagnosticsSnapshot) {
    use super::super::{evaluate::File, modules::Module};
    let (path, prepared) = prepared(&[name]);
    let file = &prepared[&path];
    let inputs = crate::fx::FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &file.query_annotation[0].1,
            module: Arc::new(Module::new(Arc::clone(&file.ts))),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = super::evaluator::new(&inputs, |_: &str, _: &Path| None, false);
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || evaluator.run(&path));
    let env = evaluator.modules[&path];
    let mut sql = super::output::projected_sql(&evaluator.scopes[env]["result"])
        .expect("sloppy mapped-parameter joins keep both complete SQL alternatives")
        .into_iter()
        .map(|value| value.text)
        .collect::<Vec<_>>();
    sql.sort();
    (sql, observer.snapshot())
}

#[test]
fn join_work_tracks_written_bindings_instead_of_untouched_state() {
    let (small_sql, small) = mapped("mapped-small.cjs");
    let (wide_sql, wide) = mapped("mapped-wide.cjs");
    assert_eq!(
        small_sql,
        ["SELECT 1 /* left-00 */", "SELECT 2 /* right-00 */"]
    );
    assert_eq!(
        wide_sql,
        ["SELECT 1 /* left-09 */", "SELECT 2 /* right-09 */"]
    );
    let key = "postgres.annotation.joined_binding_locations";
    assert_eq!(work(&small, key), 1);
    assert_eq!(work(&wide, key), 10);
}

#[path = "checkpoint_work_tests/scaling.rs"]
mod scaling;
