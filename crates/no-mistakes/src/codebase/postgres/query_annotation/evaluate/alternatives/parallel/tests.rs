use super::*;
use crate::codebase::postgres::query_annotation;
mod events;
mod fixtures;
mod invoke;
mod observer;
mod sharing;
mod sql;
mod state;
use super::super::callback_tests::private_arms::support;
mod roundtrip;
mod translation;

#[test]
fn saved_closed_factories_execute_on_distinct_workers_and_match_serial_state() {
    for name in ["no-deletion", "deletion"] {
        let (path, _, file) = support::fixture(name);
        let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
        let mut serial = support::evaluator(&files);
        let root = serial.module_environment(&path);
        assert_eq!(
            support::functions(&serial.scopes[root]["selected"]).len(),
            2
        );
        for jobs in [1, 2, 4] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(jobs)
                .build()
                .unwrap();
            let probe = super::probe::Probe::new(jobs > 1);
            pool.install(|| {
                super::probe::installed(probe.clone(), || {
                    let mut parallel = support::evaluator(&files);
                    parallel.enable_parallel_arms();
                    let root = parallel.module_environment(&path);
                    if name == "mapped.cjs" {
                        let returned = parallel.scopes[root]["selected"].clone();
                        invoke::callbacks(&mut parallel, returned);
                    }
                    state::equal(&serial, &parallel);
                })
            });
            probe.assert(jobs > 1);
        }
    }
    #[cfg(feature = "test-instrumentation")]
    support::assert_report();
}

#[test]
fn dispatch_falls_back_for_low_budget_executor_spans_and_precise_mode() {
    let (path, _, mut file) = support::fixture("no-deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let env = evaluator.module_environment(&path);
    let Expr::Sequence(parts) = &file.query_annotation[0].1.globals["selected"] else {
        panic!()
    };
    let Expr::Alternatives(arms) = &parts[1] else {
        panic!()
    };
    assert!(gate::eligible(&evaluator, arms, &path, env, 32));
    assert!(!gate::eligible(&evaluator, arms, &path, env, 7));
    let bounded = vec![arms[0].clone(); gate::MAX_FACTORY_ARMS];
    assert!(gate::eligible(&evaluator, &bounded, &path, env, 32));
    let excessive = vec![arms[0].clone(); gate::MAX_FACTORY_ARMS + 1];
    assert!(!gate::eligible(&evaluator, &excessive, &path, env, 32));
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap()
        .install(|| {
            let probe = super::probe::Probe::new(false);
            super::probe::installed(probe.clone(), || {
                evaluator.enable_parallel_arms();
                evaluator.alternatives(&excessive, &path, &env, 32, false);
            });
            probe.assert(false);
        });
    evaluator.begin_alternative_module_snapshot(evaluator.scopes.len());
    assert!(!gate::eligible(&evaluator, arms, &path, env, 32));
    evaluator.active_module_initials.clear();
    let arms = arms.clone();
    let Expr::Call { start, end, .. } = &arms[0] else {
        panic!()
    };
    let span = (*start, *end);
    drop(evaluator);
    drop(files);
    let mut files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    files.get_mut(&path).unwrap().executors.insert(span);
    let mut evaluator = support::evaluator(&files);
    let env = evaluator.module_environment(&path);
    assert!(!gate::eligible(&evaluator, &arms, &path, env, 32));
    drop(evaluator);
    drop(files);
    file.query_annotation[0].1.precise_records = true;
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let env = evaluator.module_environment(&path);
    assert!(!gate::eligible(&evaluator, &arms, &path, env, 32));
}

#[test]
fn saved_sparse_self_cycles_translate_with_recreated_deleted_slots() {
    for name in ["cyclic", "mapped.cjs"] {
        saved_cycle(name);
    }
}
fn saved_cycle(name: &str) {
    let (path, _, file) = fixtures::fixture(name);
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut serial = support::evaluator(&files);
    let root = serial.module_environment(&path);
    assert_eq!(
        support::functions(&serial.scopes[root]["selected"]).len(),
        2
    );
    if name == "mapped.cjs" {
        assert!(!serial.mapped_argument_owners.is_empty());
        assert!(!serial.mapped_parameter_indices.is_empty());
        assert!(!serial.disconnected_argument_slots.is_empty());
        let returned = serial.scopes[root]["selected"].clone();
        invoke::callbacks(&mut serial, returned);
    }
    sql::cyclic(&serial, name != "mapped.cjs");
    assert_eq!(serial.argument_extra_slots.len(), 2);
    for (id, slots) in &serial.argument_extra_slots {
        assert!(matches!(slots.get(&3), Some(Value::Arguments(cycle)) if cycle == id));
    }
    for jobs in [1, 2, 4] {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap()
            .install(|| {
                let probe = super::probe::Probe::new(jobs > 1);
                super::probe::installed(probe.clone(), || {
                    let mut parallel = support::evaluator(&files);
                    parallel.enable_parallel_arms();
                    let root = parallel.module_environment(&path);
                    if name == "mapped.cjs" {
                        let returned = parallel.scopes[root]["selected"].clone();
                        invoke::callbacks(&mut parallel, returned);
                    }
                    state::equal(&serial, &parallel);
                });
                probe.assert(jobs > 1);
            });
    }
}

#[test]
fn saved_unsafe_factory_shapes_and_imports_never_dispatch() {
    let (path, _, file) = fixtures::fixture("rejected");
    let facts = &file.query_annotation[0].1;
    let files = support::files(&path, facts, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let root = evaluator.module_environment(&path);
    for name in [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
    ] {
        let Expr::Sequence(parts) = &facts.globals[name] else {
            panic!("saved conditional")
        };
        let Expr::Alternatives(arms) = &parts[1] else {
            panic!("saved alternatives")
        };
        assert!(!gate::eligible(&evaluator, arms, &path, root, 32), "{name}");
    }
}
