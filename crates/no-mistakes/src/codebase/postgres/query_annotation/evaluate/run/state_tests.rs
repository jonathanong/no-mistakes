use super::super::File;
use super::state_test_support::{assert_same_state, known_prefixes, new};
use crate::codebase::{
    postgres::{
        extract_embedded_sql_from_program,
        query_annotation::{self, modules::Module},
        EmbeddedSqlOptions,
    },
    ts_source::facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
};
use crate::fx::FxHashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn assert_fixture_state(name: &str, precise: bool) {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-entrypoint-checkpoints")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let (mut facts, ts, embedded) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(program, &source, &options),
            collect_file_facts_from_program(
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
            ),
            extract_embedded_sql_from_program(&path, program, &source, &options),
        )
    })
    .unwrap();
    facts.precise_records = precise;
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(ts.into())),
            executors: embedded.call_spans.iter().copied().collect(),
        },
    )]);
    let resolve = |_: &str, _: &Path| -> Option<PathBuf> { None };
    let mut old = new(&files, resolve, false);
    let mut candidate = new(&files, resolve, false);
    let tasks = super::tasks::Tasks::new(&files[&path]).count();
    match name {
        "zero.mts" | "zero-dense.mts" => assert_eq!(tasks, 0),
        "one.mts" | "mapped-one.cjs" | "array-one.mts" => assert_eq!(tasks, 1),
        "many.mts" | "captured.mts" | "mapped-many.cjs" | "array-many.mts" => assert!(tasks >= 2),
        _ => {}
    }
    let mut initial = new(&files, resolve, false);
    initial.module_environment(&path);
    initial.prune_snapshot_state();
    if matches!(
        name,
        "one.mts" | "many.mts" | "mapped-one.cjs" | "mapped-many.cjs"
    ) {
        assert!(
            !initial.argument_objects.is_empty(),
            "{name} precise={precise}: fixture must retain real argument object state"
        );
    }
    if name.starts_with("mapped-") {
        assert!(!initial.mapped_arguments.is_empty());
        assert!(!initial.mapped_argument_owners.is_empty());
        assert!(!initial.mapped_parameter_indices.is_empty());
        assert!(!initial.fresh_mapped_parameters.is_empty());
        assert!(!initial.fresh_mapped_argument_bindings.is_empty());
    }
    // Ordinary return aggregation does not expose this returned callee
    // as one Function. Preserve its old conservative events while
    // requiring live copied-frame ownership in both algorithm modes.
    if name == "captured.mts" {
        assert!(initial
            .captured_bindings
            .values()
            .any(|bindings| bindings.contains_key("value")));
        assert!(initial
            .captured_binding_readers
            .values()
            .any(|names| names.contains_key("value")));
    }
    drop(initial);
    super::operations::reset();
    old.run_reference(&path);
    let old_operations = super::operations::snapshot();
    assert_eq!(
        old_operations,
        super::operations::Operations {
            captures: 1,
            restores: tasks,
            fresh: tasks
        }
    );
    super::operations::reset();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || candidate.run(&path));
    assert_eq!(
        super::operations::snapshot(),
        super::operations::Operations {
            captures: usize::from(tasks > 1),
            restores: tasks.saturating_sub(1),
            fresh: tasks,
        }
    );
    let work = observer.snapshot().work;
    let value = |name| work.get(name).copied().unwrap_or(0);
    assert_eq!(
        value("postgres.annotation.run.entrypoints.zero"),
        u64::from(tasks == 0)
    );
    assert_eq!(
        value("postgres.annotation.run.entrypoints.one"),
        u64::from(tasks == 1)
    );
    assert_eq!(
        value("postgres.annotation.run.entrypoints.many"),
        u64::from(tasks > 1)
    );
    assert_eq!(value("postgres.annotation.run.entrypoints"), tasks as u64);
    assert_eq!(
        value("postgres.annotation.run.checkpoint_captures"),
        u64::from(tasks > 1)
    );
    assert_eq!(
        value("postgres.annotation.run.checkpoint_restores"),
        tasks.saturating_sub(1) as u64
    );
    assert_eq!(
        value("postgres.annotation.run.fresh_index_rebuilds"),
        tasks as u64
    );
    assert!(!candidate.events.is_empty(), "{name} precise={precise}");
    let prefixes = candidate
        .events
        .values()
        .flatten()
        .flat_map(|(_, value)| known_prefixes(value))
        .collect::<Vec<_>>();
    // The precise packer-call boundary is conservative in the frozen public
    // API too. Capture callbacks retain ownership but have unknown call SQL.
    let packer_unknown = precise && matches!(name, "one.mts" | "many.mts");
    assert_eq!(
        prefixes.iter().any(|text| text.starts_with("/*")),
        !packer_unknown,
        "{name} precise={precise}: annotation witnesses {prefixes:?}"
    );
    let missing_expected =
        !packer_unknown && name != "captured.mts" && (name != "precise-lookahead.mts" || precise);
    assert_eq!(
        prefixes.iter().any(|text| text.starts_with("SELECT")),
        missing_expected,
        "{name} precise={precise}: missing-annotation witnesses {prefixes:?}"
    );
    assert_same_state(&old, &candidate);
}

macro_rules! fixture_case {
    ($test:ident, $file:literal, $precise:literal) => {
        #[test]
        fn $test() {
            assert_fixture_state($file, $precise);
        }
    };
}
fixture_case!(ordinary_zero, "zero.mts", false);
fixture_case!(ordinary_zero_dense, "zero-dense.mts", false);
fixture_case!(ordinary_one, "one.mts", false);
fixture_case!(ordinary_many, "many.mts", false);
fixture_case!(ordinary_precise_aliases, "precise-aliases.mts", false);
fixture_case!(ordinary_precise_lookahead, "precise-lookahead.mts", false);
fixture_case!(ordinary_captured, "captured.mts", false);
fixture_case!(ordinary_mapped_one, "mapped-one.cjs", false);
fixture_case!(ordinary_mapped_many, "mapped-many.cjs", false);
fixture_case!(precise_zero, "zero.mts", true);
fixture_case!(precise_zero_dense, "zero-dense.mts", true);
fixture_case!(precise_one, "one.mts", true);
fixture_case!(precise_many, "many.mts", true);
fixture_case!(precise_precise_aliases, "precise-aliases.mts", true);
fixture_case!(precise_precise_lookahead, "precise-lookahead.mts", true);
fixture_case!(precise_captured, "captured.mts", true);

fixture_case!(precise_array_one, "array-one.mts", true);
fixture_case!(precise_array_many, "array-many.mts", true);
