use super::*;
use crate::codebase::postgres::query_annotation::{self, evaluate::File, modules::Module};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashMap;
use references::check_reads;

#[test]
fn readonly_alternatives_skip_inherited_heap_work_and_keep_exact_sql_values() {
    for precise in [false, true] {
        check_scalar_mode(precise, "annotation-scalar-alternatives/state.mts");
    }
    check_scalar_mode(true, "annotation-readonly-garbage/root.mts");
}

fn check_scalar_mode(precise: bool, fixture: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance")
        .join(fixture);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@data-stores/psql", &[]);
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    // Precise arrays retain heap state before either read-only branch.
    facts.precise_records = precise;
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &Path| -> Option<PathBuf> { None },
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
    let observer = InvocationObserver::new(true);
    let root = with_observer(Some(observer.clone()), || {
        evaluator.module_environment(&path)
    });
    if fixture.starts_with("annotation-readonly-garbage/") {
        // Both allocation waves must execute; an unsupported body or an
        // Unknown result would make this RSS control meaningless.
        assert!(evaluator.next_builder >= 514);
        let expected = Value::Joined(
            vec![
                Value::Prefix("/* selected */ SELECT 2".into(), true, None),
                Value::Prefix("/* selected */ SELECT 3".into(), true, None),
            ]
            .into(),
        );
        assert!(evaluator.scopes[root]["selected63"] == expected);
        assert_eq!(
            observer.snapshot().work["postgres.annotation.readonly_alternatives"],
            64
        );
        return;
    }
    if precise {
        assert!(!evaluator.argument_objects.is_empty());
    }
    assert!(evaluator.next_builder > 0);
    let values = vec![
        Value::Prefix("/* selected */ SELECT 2".into(), true, None),
        Value::Prefix("/* selected */ SELECT 3".into(), true, None),
    ]
    .into();
    let expected = if precise {
        Value::Joined(values)
    } else {
        Value::Aggregate(values)
    };
    assert!(evaluator.scopes[root]["selected"] == expected);
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.readonly_alternatives"], 6);
    assert!(!work.contains_key("postgres.annotation.prefix_compare_arm"));
    // Missing local names can resolve a module; calls, module loads and
    // mutation always keep the full fallback, even beside an admissible arm.
    for arms in [
        vec![Expr::Name("missing".into())],
        vec![Expr::Member(
            Box::new(Expr::ModuleLoad("./other.mts".into())),
            "field".into(),
        )],
        vec![
            Expr::Name("left".into()),
            Expr::ModuleLoad("./other.mts".into()),
        ],
        vec![Expr::Opaque(vec![])],
        vec![Expr::Array(vec![])],
        vec![Expr::Call {
            callee: Box::new(Expr::Unknown),
            args: vec![],
            start: 0,
            end: 0,
        }],
        vec![Expr::SlotWrite {
            receiver: Box::new(Expr::Unknown),
            index: 0,
            value: Box::new(Expr::Text("changed".into())),
        }],
    ] {
        assert!(evaluator
            .readonly_alternatives(&arms, &path, &root, 16, true)
            .is_none());
    }
    check_reads(&mut evaluator, &path, root, precise);
    terminal::check_terminal(&mut evaluator, &path, root, precise);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn readonly_alternatives_fixture_preserves_the_complete_public_report() {
    for fixture in [
        "annotation-scalar-alternatives",
        "annotation-readonly-garbage",
    ] {
        check_public_fixture(fixture);
    }
}

#[cfg(feature = "test-instrumentation")]
fn check_public_fixture(fixture: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance")
        .join(fixture);
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected-report.json")).unwrap())
            .unwrap();
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&off).unwrap(),
        expected
    );
    assert_eq!(off, on);
    assert!(disabled.work.is_empty());
    assert!(enabled.work["postgres.annotation.readonly_alternatives"] >= 2);
}

mod references;

mod terminal;
