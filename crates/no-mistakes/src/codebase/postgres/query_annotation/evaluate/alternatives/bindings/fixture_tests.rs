use super::super::super::{Evaluator, File, Value};
use super::binding_scan_count::Guard;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

fn resolve(_: &str, _: &Path) -> Option<PathBuf> {
    None
}
type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;
fn evaluator<'a>(files: &'a FxHashMap<PathBuf, File<'a>>) -> TestEvaluator<'a> {
    Evaluator {
        files,
        resolve,
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
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    }
}

#[test]
fn saved_unchanged_branches_match_legacy_state_and_reduce_actual_binding_visits() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-alternative-callback/src/shared-binding-joins.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    for whole_sql in [false, true] {
        facts.precise_records = whole_sql;
        let files = FxHashMap::from_iter([(
            path.clone(),
            File {
                facts: &facts,
                module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
                executors: Default::default(),
            },
        )]);
        let (mut old, mut new) = (evaluator(&files), evaluator(&files));
        let mode = Guard::new(true);
        let old_env = old.module_environment(&path);
        let old_visits = mode.visits();
        drop(mode);
        let mode = Guard::new(false);
        let new_env = new.module_environment(&path);
        let new_visits = mode.visits();
        assert_eq!(old_env, new_env);
        assert!(
            old.scopes == new.scopes,
            "all binding values match the frozen legacy join"
        );
        macro_rules! equal { ($($field:ident),+ $(,)?) => { $(assert!(old.$field == new.$field, "{}", stringify!($field));)+ }; }
        equal!(
            events,
            modules,
            active_callback_functions,
            active_callback_executions,
            untrusted_array_intrinsics,
            next_builder,
            invalidated_builders,
            builder_updates,
            captured_bindings,
            captured_binding_readers,
            mapped_argument_owners,
            mapped_parameter_indices,
            deleted_argument_slots,
            mapped_arguments,
            fresh_mapped_parameters,
            fresh_mapped_argument_bindings,
            argument_objects,
            argument_extra_slots,
            definite_deleted_argument_slots,
            recreated_argument_slots,
            disconnected_argument_slots
        );
        assert!(old.active_module_initials.is_empty() && new.active_module_initials.is_empty());
        assert!(old.transient_roots.is_empty() && new.transient_roots.is_empty());
        let prefix = Value::Prefix("/* stable00 */ SELECT 0".into(), true, None);
        let expected = if whole_sql {
            prefix
        } else {
            Value::Aggregate(vec![prefix.clone(), prefix].into())
        };
        assert!(
            new.scopes[new_env]["retained"] == expected,
            "the branch must retain a specific SQL value"
        );
        assert!(
            old_visits >= 33 * 32,
            "all saved branches must exercise legacy joins"
        );
        println!(
            "whole_sql={whole_sql}: legacy binding visits={old_visits}, optimized={new_visits}"
        );
        assert!(
            new_visits < old_visits,
            "count visits at the actual binding iterator"
        );
    }
}

#[test]
fn saved_dirty_branches_match_legacy_state_and_bound_binding_visits() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-dirty-binding-joins/src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (mut facts, executors) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            crate::codebase::postgres::embedded::extract_embedded_sql_from_program(
                &path, program, &source, &options,
            )
            .call_spans,
        )
    })
    .unwrap();
    assert_eq!(
        executors.len(),
        2,
        "both saved configured executor calls are selected"
    );
    use crate::codebase::postgres::query_annotation::{Expr, Step};
    assert!(!facts
        .roots
        .iter()
        .any(|step| matches!(step, Step::Unsupported | Step::Branch(..))));
    let alternatives = facts
        .roots
        .iter()
        .filter_map(|step| match step {
            Step::Bind(_, Expr::Sequence(expressions)) => {
                expressions.iter().find_map(|expr| match expr {
                    Expr::Alternatives(arms) => Some(arms),
                    _ => None,
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        alternatives.len(),
        32,
        "saved AST must execute actual alternatives in both modes"
    );
    assert!(alternatives.iter().all(
        |arms| matches!(arms.first(), Some(Expr::OpaqueWrite { targets, .. }) if targets.len() == 1)
    ));
    for whole_sql in [false, true] {
        facts.precise_records = whole_sql;
        let files = FxHashMap::from_iter([(
            path.clone(),
            File {
                facts: &facts,
                module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
                executors: executors.iter().copied().collect(),
            },
        )]);
        let (mut old, mut new) = (evaluator(&files), evaluator(&files));
        let mode = Guard::new(true);
        let old_env = old.module_environment(&path);
        let old_visits = mode.visits();
        drop(mode);
        let mode = Guard::new(false);
        let new_env = new.module_environment(&path);
        let new_visits = mode.visits();
        assert_eq!(old_env, new_env);
        assert!(old.scopes == new.scopes);
        macro_rules! equal { ($($field:ident),+ $(,)?) => { $(assert!(old.$field == new.$field, "{}", stringify!($field));)+ }; }
        equal!(
            events,
            modules,
            active_callback_functions,
            active_callback_executions,
            next_builder,
            invalidated_builders,
            builder_updates,
            captured_bindings,
            captured_binding_readers,
            mapped_argument_owners,
            mapped_parameter_indices,
            deleted_argument_slots,
            mapped_arguments,
            fresh_mapped_parameters,
            fresh_mapped_argument_bindings,
            argument_objects,
            argument_extra_slots,
            definite_deleted_argument_slots,
            recreated_argument_slots,
            disconnected_argument_slots
        );
        assert!(old.active_module_initials.is_empty() && new.active_module_initials.is_empty());
        assert!(old.transient_roots.is_empty() && new.transient_roots.is_empty());
        assert!(
            new.scopes[new_env]["retained"]
                == Value::Prefix("/* stable00 */ SELECT 0".into(), true, None)
        );
        assert_eq!(
            new.events.len(),
            2,
            "both saved executor spans must execute"
        );
        for (call, expected) in [
            ("query(retained)", "/* stable00 */ SELECT 0"),
            ("query(\"SELECT 2\")", "SELECT 2"),
        ] {
            let start = source.find(call).expect("saved executor call");
            let span = (start as u32, (start + call.len()) as u32);
            assert!(executors.contains(&span));
            let values = &new.events[&(path.clone(), span)];
            assert_eq!(values.len(), 1, "each saved executor runs once");
            assert!(values[0].1 == Value::Prefix(expected.into(), true, None));
        }
        println!("dirty whole_sql={whole_sql}: legacy binding visits={old_visits}, optimized={new_visits}");
        assert!(old_visits >= 64 * 32);
        assert!(
            new_visits >= 32,
            "every saved branch must inspect its dirty binding"
        );
        assert!(
            new_visits < old_visits / 2,
            "single-key branch writes must avoid scanning the unchanged SQL bindings"
        );
    }
}
