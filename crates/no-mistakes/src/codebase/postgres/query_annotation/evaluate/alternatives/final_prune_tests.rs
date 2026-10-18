use super::*;
use crate::codebase::postgres::query_annotation::evaluate::File;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
#[path = "final_prune_tests/lifecycle.rs"]
mod lifecycle;

fn resolve(_: &str, _: &Path) -> Option<PathBuf> {
    None
}
pub(super) type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;
fn evaluator<'a>(files: &'a FxHashMap<PathBuf, File<'a>>) -> TestEvaluator<'a> {
    Evaluator {
        files,
        resolve,
        events: Default::default(),
        transactions: Default::default(),
        scopes: Default::default(),
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
#[derive(Debug)]
struct Work {
    reachability_walks: u64,
    full_frame_visits: u64,
    full_old_frame_visits: u64,
}
fn evaluate(evaluator: &mut TestEvaluator<'_>, path: &Path, full: bool) -> Work {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _observer = crate::diagnostics::InvocationGuard::install(observer.clone());
    super::super::generation::with_collection_mode(full, || {
        evaluator.module_environment(path);
    });
    let work = observer.snapshot().work;
    Work {
        reachability_walks: work
            .get("postgres.annotation.reachability_walks")
            .copied()
            .unwrap_or(0),
        full_frame_visits: work
            .get("postgres.annotation.generation_full_frame_visits")
            .copied()
            .unwrap_or(0),
        full_old_frame_visits: work
            .get("postgres.annotation.generation_full_old_frame_visits")
            .copied()
            .unwrap_or(0),
    }
}
fn compare(name: &str) -> (Work, Work) {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-alternative-callback/src")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
        },
    )]);
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    let before = evaluate(&mut old, &path, false);
    let after = evaluate(&mut new, &path, true);
    let old_frames = old.scopes.ids().collect::<FxHashSet<_>>();
    assert_eq!(old_frames, new.scopes.ids().collect::<FxHashSet<_>>());
    for env in old_frames {
        assert!(old.scopes[env] == new.scopes[env], "scope {env}");
    }
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
    assert!(
        new.next_builder >= 32,
        "saved fixture must create real speculative identities"
    );
    (before, after)
}

#[test]
fn young_collection_matches_full_oracle_for_deleted_state_and_preserves_every_mutation_table() {
    let (young, full) = compare("deleted-state.cjs");
    assert_eq!(
        young.full_frame_visits, 0,
        "young collector skips full scans"
    );
    assert!(
        full.full_frame_visits > 0,
        "the full liveness oracle visits retained frames"
    );
    assert!(
        full.full_old_frame_visits > 0,
        "the oracle includes pre-checkpoint frames"
    );
    assert!(full.reachability_walks >= young.reachability_walks);
}
#[test]
fn young_collection_matches_full_oracle_for_opaque_invalidations() {
    let (young, full) = compare("invalidated-state.mts");
    assert_eq!(
        young.full_frame_visits, 0,
        "young collector skips full scans"
    );
    assert!(full.full_frame_visits > 0);
    assert!(full.full_old_frame_visits > 0);
}
