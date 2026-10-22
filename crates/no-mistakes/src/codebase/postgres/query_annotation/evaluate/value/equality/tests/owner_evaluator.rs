use super::super::{hooks, Value};
use crate::codebase::postgres::query_annotation::{
    self,
    evaluate::{Evaluator, File},
    modules::Module,
};
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::{path::PathBuf, sync::Arc};

fn run(precise: bool) -> Vec<String> {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-shared-function-owner/root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
    let (mut facts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    facts.precise_records = precise;
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
    let owner = |name: &str| {
        let Value::Function(_, owner, _) = &evaluator.scopes[env][name] else {
            panic!("saved factory produces a concrete closure");
        };
        owner.clone()
    };
    assert!(Arc::ptr_eq(&owner("factory"), &owner("first")));
    assert!(Arc::ptr_eq(&owner("first"), &owner("second")));
    assert!(Arc::ptr_eq(&owner("first"), &owner("alias")));
    assert_eq!(owner("first").as_os_str(), path.as_os_str());
    let mut output = Vec::new();
    for (call, sql) in [
        ("read('SELECT 3')", "SELECT 3"),
        ("read('/* named */ SELECT 4')", "/* named */ SELECT 4"),
        ("read(first())", "SELECT 1"),
        ("read(second())", "/* named */ SELECT 2"),
    ] {
        let start = source.find(call).unwrap() as u32;
        let key = (path.clone(), (start, start + call.len() as u32));
        let events = &evaluator.events[&key];
        assert!(!events.is_empty());
        assert!(events.iter().all(|(_, value)| matches!(value,
            Value::Prefix(text, true, None) if text == sql)));
        output.push(format!("{}:{}", start, sql));
    }
    assert!(crate::codebase::postgres::sql_requires_query_annotation(
        "SELECT 3"
    ));
    assert!(!crate::codebase::postgres::sql_requires_query_annotation(
        "/* named */ SELECT 4"
    ));
    output
}

#[test]
fn saved_constructor_owners_and_concrete_sql_events_match_with_shortcut_disabled() {
    for precise in [false, true] {
        let (actual, _) = hooks::measure_owner(false, || run(precise));
        let (control, _) = hooks::measure_owner(true, || run(precise));
        assert_eq!(actual, control);
        assert_eq!(actual.len(), 4);
    }
}
