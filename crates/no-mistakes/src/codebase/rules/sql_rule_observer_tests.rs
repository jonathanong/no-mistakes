use super::{PostgresFindings, PreparedRuleContext, RuleFinding};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier,
};

const APPLICATIONS: usize = 16;
const RULE: &str = "postgres-explicit-columns";
const METRIC: &str = "tests.sql_rule.application_observers";
#[derive(Clone, Copy)]
enum Runner {
    Glob,
    Included,
}
fn config() -> (PathBuf, NoMistakesConfig) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/sql-policy-effort"),
    );
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("low-report.yml"))).unwrap();
    let application = config
        .rules
        .iter()
        .find(|rule| rule.rule == RULE)
        .unwrap()
        .clone();
    config.rules = (0..APPLICATIONS)
        .map(|index| {
            let mut rule = application.clone();
            rule.name = Some(format!("observed-{index}"));
            rule
        })
        .collect();
    (root, config)
}
fn run(
    runner: Runner,
    context: PreparedRuleContext<'_>,
    barrier: &Barrier,
    missing: &AtomicUsize,
    workers: &AtomicUsize,
) -> anyhow::Result<Vec<RuleFinding>> {
    let scan = |_: &Path, _: &(), _: &[PathBuf], _: &SourceStore, _: &dyn SqlFactContext| {
        // Every pair blocks until another worker steals an application. Do not
        // assert here: assertions after the join cannot strand a barrier peer.
        barrier.wait();
        if let Some(index) = rayon::current_thread_index() {
            workers.fetch_or(1 << index, Ordering::SeqCst);
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(METRIC, 1);
        } else {
            missing.fetch_add(1, Ordering::SeqCst);
        }
        Ok(PostgresFindings::default())
    };
    match runner {
        Runner::Glob => super::run_sql_rule_with_glob_filters::<serde_yaml::Value, ()>(
            context,
            RULE,
            |_| Ok(()),
            |_| (&[], &[]),
            scan,
        ),
        Runner::Included => {
            super::run_included_postgres_rule(context, RULE, |_| Ok(()), |_, _| true, scan)
        }
    }
}
fn count(observer: &crate::diagnostics::InvocationObserver) -> u64 {
    observer.snapshot().work.get(METRIC).copied().unwrap_or(0)
}
fn exercise(runner: Runner) {
    let (root, config) = config();
    assert_eq!(config.rule_applications(RULE).len(), APPLICATIONS);
    let sources = super::source_store_for_files(&[]);
    let facts = CheckFactMap::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let barrier = Barrier::new(2);
    let missing = AtomicUsize::new(0);
    let workers = AtomicUsize::new(0);
    let invoke = || {
        run(
            runner,
            PreparedRuleContext::new(&root, &config, &[], &sources, &facts),
            &barrier,
            &missing,
            &workers,
        )
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let result =
        pool.install(|| crate::diagnostics::with_observer(Some(Arc::clone(&observer)), invoke));
    assert!(result.unwrap().is_empty());
    assert_eq!(
        workers.load(Ordering::SeqCst),
        3,
        "both pool workers must participate"
    );
    assert_eq!(missing.load(Ordering::SeqCst), 0);
    assert_eq!(count(&observer), APPLICATIONS as u64);
    assert!(pool
        .broadcast(|_| crate::diagnostics::current().is_none())
        .into_iter()
        .all(|clear| clear));

    let unrelated = crate::diagnostics::InvocationObserver::new(true);
    let result = pool.install(|| {
        crate::diagnostics::with_observer(Some(Arc::clone(&unrelated)), || {
            let result = crate::diagnostics::with_observer(None, invoke);
            // The disabled request restores its caller's independent observer.
            assert!(crate::diagnostics::current()
                .is_some_and(|active| Arc::ptr_eq(&active, &unrelated)));
            result
        })
    });
    assert!(result.unwrap().is_empty());
    assert_eq!(missing.load(Ordering::SeqCst), APPLICATIONS);
    assert_eq!(count(&observer), APPLICATIONS as u64);
    assert_eq!(count(&unrelated), 0);
    assert!(pool
        .broadcast(|_| crate::diagnostics::current().is_none())
        .into_iter()
        .all(|clear| clear));

    let next = crate::diagnostics::InvocationObserver::new(true);
    let result =
        pool.install(|| crate::diagnostics::with_observer(Some(Arc::clone(&next)), invoke));
    assert!(result.unwrap().is_empty());
    assert_eq!(count(&next), APPLICATIONS as u64);
    assert_eq!(count(&observer), APPLICATIONS as u64);
    assert_eq!(count(&unrelated), 0);
    assert_eq!(missing.load(Ordering::SeqCst), APPLICATIONS);
    assert!(pool
        .broadcast(|_| crate::diagnostics::current().is_none())
        .into_iter()
        .all(|clear| clear));
}
#[test]
fn sql_glob_applications_propagate_observers_and_isolate_disabled_requests() {
    exercise(Runner::Glob);
}
#[test]
fn included_sql_applications_propagate_observers_and_isolate_disabled_requests() {
    exercise(Runner::Included);
}
