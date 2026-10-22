use super::super::Value;
use crate::codebase::postgres::query_annotation::{
    self,
    evaluate::{Evaluator, File},
    modules::Module,
    Expr,
};
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[test]
fn saved_legacy_tag_checks_actual_local_function_owner_and_start() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-shared-function-owner/local-tag.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
    let (facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    let tag_start = facts.legacy_tag_spans["String"];
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(TsFileFacts::default().into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    evaluator.run(&path);
    let env = evaluator.module_environment(&path);
    let Value::Function(function, owner, captured) = evaluator.scopes[env]["String"].clone() else {
        panic!("saved legacy tag retains its local Function binding");
    };
    assert_eq!(function.start, tag_start);
    assert_eq!(owner.as_ref(), path.as_path());
    let call = "read(String`SELECT 5`)";
    let start = source.find(call).unwrap() as u32;
    let events = &evaluator.events[&(path.clone(), (start, start + call.len() as u32))];
    assert!(!events.is_empty());
    assert!(events.iter().all(|(_, value)| matches!(value,
        Value::Prefix(sql, true, Some(_)) if sql == "SELECT 5")));
    assert!(crate::codebase::postgres::sql_requires_query_annotation(
        "SELECT 5"
    ));
    let parts = [Expr::Text("SELECT 5".into())];
    // Effects are empty here to isolate the identity guard from callback execution.
    evaluator.scopes[env].insert(
        "String".into(),
        Value::Function(
            function.clone(),
            Path::new("different-owner.mts").into(),
            captured,
        ),
    );
    assert!(matches!(
        evaluator.tagged("String", &parts, &[], &path, &env, (16, false)),
        Value::Unknown
    ));
    let mut different_start = (*function).clone();
    different_start.start += 1;
    evaluator.scopes[env].insert(
        "String".into(),
        Value::Function(Arc::new(different_start), owner, captured),
    );
    assert!(matches!(
        evaluator.tagged("String", &parts, &[], &path, &env, (16, false)),
        Value::Unknown
    ));
}
