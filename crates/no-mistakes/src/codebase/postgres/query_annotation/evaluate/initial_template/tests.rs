use super::super::{Capture, Value};
use super::*;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

fn fixture() -> (PathBuf, query_annotation::QueryAnnotationFileFacts) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/module-captures/pure.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap();
    (path, facts)
}

#[test]
fn module_capture_checks_frame_identity_not_only_source_owner() {
    let files = fx_map();
    let mut evaluator = super::super::new(&files, |_: &str, _: &Path| None, false);
    let path = PathBuf::from("owner.mts");
    let module = evaluator.environment(fx_map());
    evaluator.modules.insert(path.clone(), module);
    let local = evaluator.environment(fx_map());
    assert_eq!(evaluator.capture(&path, module), Capture::Module);
    assert_eq!(
        evaluator.capture(&path, local),
        Capture::Lexical(local.into())
    );
    assert_eq!(
        Capture::Module.environment(&path, &evaluator.modules),
        module
    );
}

#[test]
#[should_panic(expected = "module capture retains its registered frame")]
fn live_module_capture_requires_its_registered_owner() {
    Capture::Module.environment(Path::new("missing.mts"), &fx_map());
}

#[test]
fn independent_module_scopes_share_across_root_local_frame_ids_and_detach_on_write() {
    let (path, facts) = fixture();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
            initial_template: Default::default(),
        },
    )]);
    let resolve =
        |_: &str, _: &Path| -> Option<PathBuf> { panic!("pure template cannot resolve imports") };
    let mut first = super::super::new(&files, resolve, false);
    first.environment(fx_map());
    let left = first.module_environment(&path);
    let mut second = super::super::new(&files, resolve, false);
    let right = second.module_environment(&path);
    assert_ne!(left, right);
    assert!(std::ptr::eq(&*first.scopes[left], &*second.scopes[right]));
    let Value::Function(_, owner, capture) = &first.scopes[left]["read"] else {
        panic!("module function")
    };
    assert_eq!(*capture, Capture::Module);
    assert_eq!(capture.environment(owner, &first.modules), left);
    assert_eq!(capture.environment(owner, &second.modules), right);
    first.scopes[left].insert("statement".into(), Value::Unknown);
    assert!(!std::ptr::eq(&*first.scopes[left], &*second.scopes[right]));
    assert!(matches!(
        second.scopes[right]["statement"],
        Value::Prefix(_, true, None)
    ));
    // An eager cyclic dependency may already have written the reserved frame.
    // A cached pristine scope must not overwrite that activation's mutation.
    let mut guarded = super::super::new(&files, resolve, false);
    let guarded_env =
        guarded.environment(FxHashMap::from_iter([("statement".into(), Value::Unknown)]));
    guarded.modules.insert(path.clone(), guarded_env);
    assert!(!guarded.apply_initial_template(&path, guarded_env));
    assert!(matches!(
        guarded.scopes[guarded_env]["statement"],
        Value::Unknown
    ));
}

#[test]
fn independent_steps_and_globals_leave_no_identity_or_effect_state() {
    let (path, facts) = fixture();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
            initial_template: Default::default(),
        },
    )]);
    let mut canonical = super::super::new(
        &files,
        |_: &str, _: &Path| -> Option<PathBuf> {
            panic!("independent initializer must not resolve")
        },
        false,
    );
    let mut env = canonical.environment(fx_map());
    canonical.modules.insert(path.clone(), env);
    canonical.steps(&facts.roots, &path, &mut env, 16, true);
    canonical.initialize_module_globals(&path, env);
    assert!(construct(&files, &path, None).unwrap() == canonical.scopes[env]);
    assert_eq!(canonical.scopes.len(), 1);
    assert_eq!(canonical.modules.len(), 1);
    assert_eq!(canonical.next_builder, 0);
    assert!(!canonical.untrusted_array_intrinsics);
    assert!(canonical.active_callback_functions.is_none());
    assert!(canonical.active_callback_executions.is_none());
    macro_rules! empty {
        ($($field:ident),+ $(,)?) => { $(assert!(canonical.$field.is_empty(), stringify!($field));)+ };
    }
    empty!(
        events,
        active_module_initials,
        transient_roots,
        invalidated_builders,
        builder_updates,
        captured_bindings,
        captured_binding_readers,
        mapped_argument_owners,
        mapped_parameter_indices,
        deleted_argument_slots,
        argument_objects,
        argument_extra_slots,
        definite_deleted_argument_slots,
        recreated_argument_slots,
        disconnected_argument_slots,
        fresh_mapped_parameters,
        fresh_mapped_argument_bindings,
        mapped_arguments
    );
}

#[test]
fn reserved_frame_guard_preserves_capture_readers_mapped_writes_and_freshness() {
    let files = fx_map();
    let mut evaluator = super::super::new(&files, |_: &str, _: &Path| None, false);
    let env = evaluator.environment(fx_map());
    assert!(evaluator.template_context_pristine(env));
    evaluator.scopes[env].insert("statement".into(), Value::Unknown);
    assert!(!evaluator.template_context_pristine(env));
    evaluator.scopes[env].clear();
    evaluator.captured_bindings.insert(env, Default::default());
    assert!(!evaluator.template_context_pristine(env));
    evaluator.captured_bindings.clear();
    let reader = evaluator.environment(fx_map());
    evaluator
        .captured_bindings
        .insert(reader, FxHashMap::from_iter([("statement".into(), env)]));
    evaluator.captured_binding_readers.insert(
        env,
        FxHashMap::from_iter([(
            "statement".into(),
            crate::fx::FxHashSet::from_iter([reader]),
        )]),
    );
    assert!(!evaluator.template_context_pristine(env));
    evaluator.captured_bindings.clear();
    evaluator.captured_binding_readers.clear();
    evaluator.mapped_arguments.insert(env, Vec::new());
    assert!(!evaluator.template_context_pristine(env));
    evaluator.mapped_arguments.clear();
    evaluator
        .fresh_mapped_parameters
        .insert(env, Default::default());
    assert!(!evaluator.template_context_pristine(env));
    evaluator.fresh_mapped_parameters.clear();
    assert!(evaluator.template_context_pristine(env));
}

#[test]
fn canonical_capture_preserves_function_equality_within_one_module_world() {
    let (path, facts) = fixture();
    let Expr::Function(function) = &facts.globals["read"] else {
        panic!("stored function")
    };
    let files = fx_map();
    let mut evaluator = super::super::new(&files, |_: &str, _: &Path| None, false);
    let module = evaluator.environment(fx_map());
    evaluator.modules.insert(path.clone(), module);
    let first_local = evaluator.environment(fx_map());
    let second_local = evaluator.environment(fx_map());
    let other_owner = path.with_file_name("other.mts");
    let numeric = [
        (&path, module),
        (&path, module),
        (&path, first_local),
        (&path, first_local),
        (&path, second_local),
        (&other_owner, module),
    ];
    let converted = numeric
        .iter()
        .map(|(owner, env)| {
            Value::Function(
                Arc::clone(function),
                (*owner).clone(),
                evaluator.capture(owner, *env),
            )
        })
        .collect::<Vec<_>>();
    for (left, (left_owner, left_env)) in numeric.iter().enumerate() {
        for (right, (right_owner, right_env)) in numeric.iter().enumerate() {
            assert_eq!(
                converted[left] == converted[right],
                left_owner == right_owner && left_env == right_env
            );
        }
    }
}

#[test]
fn concurrent_demand_builds_one_profile_local_template() {
    let (path, facts) = fixture();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
            initial_template: Default::default(),
        },
    )]);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let evaluate = || {
        crate::diagnostics::with_observer(Some(observer.clone()), || {
            let mut evaluator = super::super::new(&files, |_: &str, _: &Path| None, false);
            let env = evaluator.module_environment(&path);
            evaluator.scopes[env].clone()
        })
    };
    let (first, second) = std::thread::scope(|threads| {
        let first = threads.spawn(evaluate);
        let second = threads.spawn(evaluate);
        (first.join().unwrap(), second.join().unwrap())
    });
    assert!(std::ptr::eq(&*first, &*second));
    let work = observer.snapshot().work;
    assert_eq!(
        work["postgres.annotation.initial_template.template_builds"],
        1
    );
    assert_eq!(work["postgres.annotation.module_initializations"], 1);
    assert_eq!(
        work["postgres.annotation.initial_template.module_activations"],
        2
    );
    assert_eq!(
        work["postgres.annotation.initial_template.applied_activations"],
        2
    );
}

// The saved-fixture observer entrypoint is supplied by this feature.
#[cfg(feature = "test-instrumentation")]
#[test]
fn two_real_executor_roots_initialize_their_shared_pure_helper_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-module-initial-scopes");
    let (output, diagnostics) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["rules"], serde_json::json!([]));
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots"], 2);
    // Two distinct roots plus one shared helper: all three initial scopes are
    // built once, even though the helper is demanded from both root evaluators.
    assert_eq!(
        work["postgres.annotation.initial_template.template_builds"],
        3
    );
    assert_eq!(work["postgres.annotation.module_initializations"], 3);
    assert!(work["postgres.annotation.initial_template.module_activations"] > 3);
    assert_eq!(
        work["postgres.annotation.initial_template.applied_activations"],
        work["postgres.annotation.initial_template.module_activations"],
    );
}

#[test]
fn lazy_branch_returned_functions_keep_the_registered_module_owner() {
    use crate::codebase::ts_source::facts::{
        collect_file_facts_from_program, TsFactContext, TsFactPlan,
    };
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/module-captures");
    let entry = base.join("entry.mts");
    let shared = base.join("shared.mts");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let prepared = [&entry, &shared]
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).unwrap();
            let facts = crate::ast::with_program(path, &source, |program, _| {
                (
                    query_annotation::collect(program, &source, &options),
                    collect_file_facts_from_program(
                        path,
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
                    ),
                )
            })
            .unwrap();
            (path.clone(), facts)
        })
        .collect::<Vec<_>>();
    let files = prepared
        .iter()
        .map(|(path, (facts, ts))| {
            (
                path.clone(),
                File {
                    facts,
                    module: Arc::new(Module::new(ts.clone().into())),
                    executors: Default::default(),
                    initial_template: Default::default(),
                },
            )
        })
        .collect::<FxHashMap<_, _>>();
    let mut evaluator = super::super::new(
        &files,
        |specifier: &str, _: &Path| (specifier == "./shared.mts").then(|| shared.clone()),
        false,
    );
    let root = evaluator.module_environment(&entry);
    assert!(
        !evaluator.modules.contains_key(&shared),
        "ordinary imports stay lazy"
    );
    let Value::Function(choose, _, _) = evaluator.scopes[root]["choose"].clone() else {
        panic!("saved branch function")
    };
    let mut local = evaluator.environment((*evaluator.scopes[root]).clone());
    evaluator.scopes[local].insert("flag".into(), Value::Unknown);
    let returned = evaluator.steps(&choose.body, &entry, &mut local, 12, false);
    fn captures(value: &Value, owner: &Path, output: &mut Vec<Capture>) {
        match value {
            Value::Function(_, path, capture) if path == owner => output.push(*capture),
            Value::Promise(inner) | Value::Evaluated(inner, _) => captures(inner, owner, output),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                for value in values {
                    captures(value, owner, output);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    captures(&returned, &shared, &mut found);
    assert!(
        !found.is_empty(),
        "the branch result retains its imported function"
    );
    assert!(found.iter().all(|capture| *capture == Capture::Module));
    let registered = evaluator.modules[&shared];
    for capture in found {
        assert_eq!(capture.environment(&shared, &evaluator.modules), registered);
    }
    assert!(matches!(
        evaluator.scopes[registered]["read"],
        Value::Function(_, _, Capture::Module)
    ));
}

#[test]
fn eligibility_rejects_work_that_depends_on_activation_state() {
    let (_, baseline) = fixture();
    let check = |facts: &query_annotation::QueryAnnotationFileFacts| {
        eligibility(&File {
            facts,
            module: Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
            initial_template: Default::default(),
        })
    };
    assert_eq!(check(&baseline), Ok(()));
    let mut facts = baseline.clone();
    facts.roots.clear();
    assert_eq!(
        check(&facts),
        Err("postgres.annotation.initial_template.rejected_empty")
    );
    facts.roots = vec![Step::Bind(
        "dependent".into(),
        Expr::Name("imported".into()),
    )];
    assert_eq!(
        check(&facts),
        Err("postgres.annotation.initial_template.rejected_initializer")
    );
    facts.roots = vec![Step::Effect(Expr::Primitive)];
    assert_eq!(
        check(&facts),
        Err("postgres.annotation.initial_template.rejected_control_or_effect")
    );
    facts = baseline.clone();
    facts.globals.insert("unmodeled".into(), Expr::Unknown);
    assert_eq!(
        check(&facts),
        Err("postgres.annotation.initial_template.rejected_global")
    );
}

#[test]
fn lexical_handles_round_trip_zero_and_the_largest_representable_index() {
    for index in [0, 7, usize::MAX - 1] {
        let capture = Capture::from(index);
        assert_eq!(
            capture.environment(Path::new("local.mts"), &fx_map()),
            index
        );
    }
}

#[test]
#[should_panic(expected = "lexical frame index fits a nonzero handle")]
fn lexical_handles_reject_an_unrepresentable_frame_index() {
    let _ = Capture::from(usize::MAX);
}

#[test]
fn captures_preserve_the_existing_frame_word_width() {
    assert_eq!(
        std::mem::size_of::<Capture>(),
        std::mem::size_of::<Environment>()
    );
}
