use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, FxHashMap};
use std::path::PathBuf;

fn fixture(name: &str) -> (PathBuf, query_annotation::QueryAnnotationFileFacts) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-argument-slot-write/src/{name}"
    ));
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    (path, facts)
}

fn slot_write(facts: &query_annotation::QueryAnnotationFileFacts, name: &str) -> (u32, usize) {
    let Some(query_annotation::Expr::Function(function)) = facts.globals.get(name) else {
        panic!("missing function {name}");
    };
    let Some(index) = function.body.iter().find_map(|step| match step {
        query_annotation::Step::Bind(_, query_annotation::Expr::SlotWrite { index, .. }) => {
            Some(*index)
        }
        _ => None,
    }) else {
        panic!("missing recognized argument slot write in {name}");
    };
    (function.start, index)
}

#[test]
fn only_canonical_static_arguments_indexes_are_collected_as_slot_writes() {
    let (_, strict) = fixture("query.mts");
    let (_, sloppy) = fixture("sloppy.cjs");
    let (strict_start, strict_index) = slot_write(&strict, "strictReplacement");
    let (sloppy_start, sloppy_index) = slot_write(&sloppy, "sloppyReplacement");

    assert_eq!(strict_index, 0);
    assert_eq!(sloppy_index, 0);
    assert!(!strict.mapped_arguments.contains(&strict_start));
    assert!(sloppy.mapped_arguments.contains(&sloppy_start));

    let Some(query_annotation::Expr::Function(function)) = strict.globals.get("dynamicReplacement")
    else {
        panic!("missing dynamic key case");
    };
    assert!(matches!(
        function.body.iter().find_map(|step| match step {
            query_annotation::Step::Bind(_, expr) => Some(expr),
            _ => None,
        }),
        Some(query_annotation::Expr::OpaqueWrite { .. })
    ));
}

#[test]
fn argument_slot_index_classification_is_canonical_and_receiver_specific() {
    let (_, facts) = fixture("index-cases.mts");
    let Some(query_annotation::Expr::Function(function)) = facts.globals.get("indexCases") else {
        panic!("missing index classification fixture function");
    };
    let mut indexes = Vec::new();
    let mut opaque_writes = 0;
    for step in &function.body {
        let query_annotation::Step::Bind(_, expression) = step else {
            continue;
        };
        match expression {
            query_annotation::Expr::SlotWrite { index, .. } => indexes.push(*index),
            query_annotation::Expr::OpaqueWrite { .. } => opaque_writes += 1,
            _ => {}
        }
    }

    assert_eq!(indexes, [0, 1, 2]);
    // Fractional, unsafe/huge, noncanonical, nonnumeric, dynamic, and other
    // receiver writes stay opaque so they cannot update a proven argument slot.
    assert_eq!(opaque_writes, 7);
}

#[test]
fn static_slot_replacement_preserves_strict_formal_and_updates_sloppy_alias() {
    let (path, _) = fixture("query.mts");
    let facts = query_annotation::QueryAnnotationFileFacts::default();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: Default::default(),
            imports: Default::default(),
            exports: Default::default(),
        },
    )]);
    let original = Value::Prefix("/* annotated */ SELECT 1".into(), true, Some(9));
    let replacement = Value::Prefix("replacement".into(), true, None);
    let mut scope = fx_map();
    scope.insert("arguments".into(), Value::Arguments(1));
    scope.insert("statement".into(), original.clone());
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: vec![scope],
        modules: Default::default(),
        next_builder: 10,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: FxHashMap::from_iter([(1, vec![original.clone()])]),
        definite_deleted_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        mapped_arguments: Default::default(),
    };
    let strict = evaluator.slot_write(
        &query_annotation::Expr::Name("arguments".into()),
        0,
        &query_annotation::Expr::Text("replacement".into()),
        &path,
        &0,
        (16, false),
    );

    assert!(matches!(strict, Value::Evaluated(_, true)));
    assert!(evaluator.scopes[0]["statement"] == original);
    assert!(evaluator.argument_objects[&1][0] == replacement);
    assert!(evaluator.invalidated_builders.contains(&1));
    assert!(!evaluator.invalidated_builders.contains(&9));

    evaluator
        .mapped_arguments
        .insert(0, vec![(1, vec!["statement".into()])]);
    evaluator.rebuild_mapped_argument_owners();
    evaluator.scopes[0].insert(
        "statement".into(),
        Value::Prefix("/* annotated */ SELECT 1".into(), true, Some(9)),
    );
    evaluator.argument_objects.insert(
        1,
        vec![Value::Prefix(
            "/* annotated */ SELECT 1".into(),
            true,
            Some(9),
        )],
    );
    evaluator.invalidated_builders.clear();
    let sloppy = evaluator.slot_write(
        &query_annotation::Expr::Name("arguments".into()),
        0,
        &query_annotation::Expr::Text("replacement".into()),
        &path,
        &0,
        (16, false),
    );

    assert!(matches!(sloppy, Value::Evaluated(_, true)));
    assert!(evaluator.scopes[0]["statement"] == replacement);
    assert!(matches!(
        evaluator.mapped_parameter_value(0, "statement"),
        Some(Value::Unknown)
    ));
    assert!(!evaluator.invalidated_builders.contains(&9));
}

#[test]
fn sloppy_fixture_slot_write_updates_its_actual_mapped_parameter() {
    let (path, facts) = fixture("sloppy.cjs");
    assert!(facts.mapped_arguments.iter().any(|start| {
        matches!(facts.globals.get("sloppyReplacement"),
            Some(query_annotation::Expr::Function(function)) if function.start == *start)
    }));
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: Default::default(),
            imports: Default::default(),
            exports: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        mapped_arguments: Default::default(),
    };
    let root = evaluator.module_environment(&path);
    let call = query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name("sloppyReplacement".into())),
        args: vec![query_annotation::Expr::Text("original".into())],
        start: 0,
    };
    let result = evaluator.expr(&call, &path, &root, 16, false);

    assert!(matches!(result, Value::Unknown));
    let Some((frame, _)) = evaluator.mapped_arguments.iter().find(|(_, mappings)| {
        mappings
            .iter()
            .any(|(_, params)| params.iter().any(|name| name == "statement"))
    }) else {
        panic!("missing actual sloppy mapped frame");
    };
    assert!(matches!(
        evaluator.scopes[*frame].get("statement"),
        Some(Value::Prefix(text, true, None)) if text == "replacement"
    ));
}

#[test]
fn replacing_arguments_slot_does_not_invalidate_the_detached_strict_formal() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-argument-slot-write/src/query.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            ts: &ts,
            executors: embedded.call_starts.iter().copied().collect(),
            imports: Default::default(),
            exports: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.module_environment(&path);

    let events = embedded
        .calls
        .iter()
        .zip(&embedded.call_starts)
        .filter_map(|(call, start)| {
            let line = source.lines().nth(call.line as usize - 1)?;
            let value = evaluator
                .events
                .get(&(path.clone(), *start))?
                .first()?
                .1
                .clone();
            Some((line.to_string(), value))
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        events.iter().find(|(sql, _)| sql.contains("preserved formal")),
        Some((_, Value::Prefix(sql, true, _))) if sql.contains("preserved formal")
    ));
    assert!(matches!(
        events.iter().find(|(sql, _)| sql.contains("dynamic slot")),
        Some((_, Value::Unknown))
    ));
    let Some((_, sloppy)) = events
        .iter()
        .find(|(line, _)| line.contains("sloppy mapped slot"))
    else {
        panic!("missing imported sloppy helper executor event");
    };
    assert!(!matches!(sloppy, Value::Prefix(_, true, _)));
}
