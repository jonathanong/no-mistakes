use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::cell::Cell;

thread_local! {
    static MODE: Cell<(bool, usize)> = const { Cell::new((false, 0)) };
}
pub(super) fn original_prune() -> bool {
    MODE.with(|mode| mode.get().0)
}
pub(super) fn record_reuse() {
    MODE.with(|mode| {
        let (old, count) = mode.get();
        mode.set((old, count + 1));
    });
}
struct Guard((bool, usize));
impl Guard {
    fn new(original: bool) -> Self {
        Self(MODE.with(|mode| mode.replace((original, 0))))
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        MODE.with(|mode| mode.set(self.0));
    }
}
fn resolve(_: &str, _: &Path) -> Option<PathBuf> {
    None
}
type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;
fn evaluator<'a>(files: &'a FxHashMap<PathBuf, File<'a>>) -> TestEvaluator<'a> {
    Evaluator {
        initial_template_observer: crate::diagnostics::current(),
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
fn evaluate(evaluator: &mut TestEvaluator<'_>, path: &Path, original: bool) -> (u64, usize) {
    let _mode = Guard::new(original);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _observer = crate::diagnostics::InvocationGuard::install(observer.clone());
    evaluator.module_environment(path);
    let walks = observer
        .snapshot()
        .work
        .get("postgres.annotation.reachability_walks")
        .copied()
        .unwrap_or(0);
    (walks, MODE.with(|mode| mode.get().1))
}
fn compare(name: &str) -> ((u64, usize), (u64, usize)) {
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
            initial_template: Default::default(),
            facts: &facts,
            module: std::sync::Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
        },
    )]);
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    let before = evaluate(&mut old, &path, true);
    let after = evaluate(&mut new, &path, false);
    assert!(old.scopes == new.scopes, "all scope values");
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
fn deleted_state_reuses_the_last_walk_and_preserves_every_mutation_table() {
    let (old, new) = compare("deleted-state.cjs");
    assert_eq!(old.1, 0, "verbatim old prune path");
    assert_eq!(
        new.1, 32,
        "all32 saved conditionals hit the proved boundary"
    );
    assert_eq!(
        old.0 - new.0,
        32,
        "remove actual walks, not diagnostic bookkeeping"
    );
}
#[test]
fn opaque_invalidations_retain_the_original_walk_fallback() {
    let (old, new) = compare("invalidated-state.mts");
    assert_eq!(new.1, 0, "opaque mutations invalidate the no-write proof");
    assert_eq!(old.0, new.0);
}
#[test]
fn final_live_guard_keeps_captures_and_moved_transient_environments_conservative() {
    use query_annotation::evaluate::temporaries::TransientRoot;
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    evaluator.scopes.push(Scope::default());
    let identity = FxHashMap::from_iter([(0, 0)]);
    let moved = FxHashMap::from_iter([(1, 0)]);
    assert!(evaluator.can_reuse_final_live(1, true, true, &identity));
    assert!(!evaluator.can_reuse_final_live(1, false, true, &identity));
    assert!(!evaluator.can_reuse_final_live(1, true, false, &identity));
    assert!(!evaluator.can_reuse_final_live(1, true, true, &moved));
    evaluator.transient_roots = vec![
        TransientRoot::Argument(7),
        TransientRoot::Builder(9),
        TransientRoot::Environment(0),
    ];
    assert!(evaluator.can_reuse_final_live(1, true, true, &identity));
    evaluator
        .transient_roots
        .push(TransientRoot::Environment(1));
    assert!(!evaluator.can_reuse_final_live(1, true, true, &identity));
    evaluator.transient_roots.clear();
    evaluator
        .captured_bindings
        .insert(0, FxHashMap::from_iter([("x".into(), 0)]));
    assert!(!evaluator.can_reuse_final_live(1, true, true, &identity));
    evaluator.captured_bindings.clear();
    evaluator.scopes[0].insert("arguments".into(), Value::Arguments(7));
    evaluator.argument_objects.insert(7, vec![Value::Unknown]);
    evaluator.begin_alternative_module_snapshot(0);
    evaluator.record_alternative_module_initial(Path::new("guard.mts"), 0);
    assert!(!evaluator.can_reuse_final_live(1, true, true, &identity));
}
