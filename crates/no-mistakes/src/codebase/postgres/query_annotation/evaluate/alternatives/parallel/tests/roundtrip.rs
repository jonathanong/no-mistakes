use super::super::Outcome;
use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};
mod assertions;
mod seed;
#[test]
fn saved_factory_private_heap_delta_filters_and_translates_every_namespace() {
    let (path, _, file) = support::fixture("no-deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut baseline = support::evaluator(&files);
    let root = baseline.module_environment(&path);
    let Expr::Sequence(parts) = &file.query_annotation[0].1.globals["selected"] else {
        panic!()
    };
    let Expr::Alternatives(arms) = &parts[1] else {
        panic!()
    };
    let (base_env, base_id) = (baseline.scopes.len(), baseline.next_builder);
    for (env_gap, id_gap) in [(0, 0), (2, 7)] {
        let mut fork = baseline.fork_arm();
        let value = fork.expr(&arms[0], &path, &root, 32, false);
        assert!(matches!(value, Value::Function(_, _, env) if env == base_env));
        seed::metadata(&mut fork, base_env, base_id);
        let scope = fork.scopes[base_env].clone();
        let dense = fork.argument_objects[&base_id].clone();
        let outcome = Outcome::take(fork, value, base_env, base_id);
        let mut target = baseline.fork_arm();
        target
            .scopes
            .extend((0..env_gap).map(|_| Default::default()));
        target.next_builder += id_gap;
        let observer = InvocationObserver::new(true);
        let returned = with_observer(Some(observer.clone()), || outcome.install(&mut target));
        let env = base_env + env_gap;
        let id = base_id + id_gap;
        assert!(matches!(returned, Value::Function(_, _, returned_env) if returned_env == env));
        assertions::metadata(&target, env, id);
        assert_eq!(
            target.next_builder,
            id + 5,
            "advance full allocated ID range"
        );
        assert!(
            target.builder_updates.get(&1) == baseline.builder_updates.get(&1),
            "incoming fork-only mutation is filtered"
        );
        assert!(target.captured_bindings.get(&0) == baseline.captured_bindings.get(&0));
        assert_eq!(target.argument_objects[&id].len(), 2);
        assert!(matches!(target.argument_objects[&id][0], Value::Arguments(cycle) if cycle == id));
        let copied = observer
            .snapshot()
            .work
            .get("postgres.annotation.dense_arena_slots_copied")
            .copied()
            .unwrap_or(0);
        if env_gap == 0 {
            assert!(target.scopes[env].same_bindings(&scope));
            assert!(target.argument_objects[&id].shared() && dense.shared());
            assert_eq!(copied, 0, "identity install must move shared payloads");
        } else {
            assert!(!target.scopes[env].same_bindings(&scope));
            assert_eq!(
                copied, 2,
                "translated shared dense detach remains physically accounted"
            );
        }
    }
}
