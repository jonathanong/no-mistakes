use super::*;
use crate::codebase::postgres::query_annotation::evaluate::module_cache::initials::SharedInitials;

#[test]
fn shared_unknown_calls_preserve_intrinsic_trust_without_adding_builder_invalidation() {
    for precise in [false, true] {
        let mut files = prepared_fixture("helper-tracing-shared-initials", &["opaque"]);
        for (_, file) in &mut files {
            file.query_annotation.truncate(1);
            file.embedded_sql.truncate(1);
            file.query_annotation[0].1.in_scope = Some(false);
            file.query_annotation[0].1.precise_records = precise;
        }
        let resolve = |_: &str, _: &std::path::Path| None;
        let mut profiles = super::super::prepare::profiles(&files, &resolve);
        let profile = profiles.pop().unwrap();
        let path = profile.files.keys().next().unwrap();
        let initials = SharedInitials::new();
        let run = |shared| {
            let mut evaluator = super::super::evaluator::new(&profile.files, &resolve, false);
            evaluator.environment(crate::fx::FxHashMap::from_iter([(
                "builder".into(),
                query_annotation::evaluate::Value::Prefix(
                    "/* live */ SELECT 1".into(),
                    true,
                    Some(42),
                ),
            )]));
            if shared {
                evaluator.modules.share_initials(Arc::clone(&initials));
            }
            evaluator.module_environment(path);
            assert!(evaluator.untrusted_array_intrinsics);
            assert_eq!(evaluator.invalidated_builders.contains(&42), precise);
            evaluator.scopes[0].clone()
        };
        let baseline = run(false);
        for _ in 0..2 {
            assert!(baseline == run(true));
        }
    }
}

#[test]
fn contextual_closures_are_rejected_before_standalone_evaluation() {
    for name in ["returned-closure", "method-call", "branch"] {
        contextual_module(name);
    }
}

fn contextual_module(name: &str) {
    let mut files = prepared_fixture("helper-tracing-shared-initials", &[name]);
    for (_, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
        file.query_annotation[0].1.in_scope = Some(false);
    }
    let resolve = |_: &str, _: &std::path::Path| None;
    let mut profiles = super::super::prepare::profiles(&files, &resolve);
    let profile = profiles.pop().unwrap();
    let path = profile.files.keys().next().unwrap();
    let initials = SharedInitials::new();
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        for _ in 0..2 {
            let mut evaluator = super::super::evaluator::new(&profile.files, &resolve, false);
            evaluator.modules.share_initials(Arc::clone(&initials));
            evaluator.module_environment(path);
        }
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.module"], 2);
    assert!(!work.contains_key("postgres.annotation.shared_module_initializations"));
}

#[test]
fn shared_template_and_sequence_scopes_isolate_writes_and_remap_leaf_closures() {
    let mut files = prepared_fixture("helper-tracing-shared-initials", &["operators"]);
    for (_, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
        file.query_annotation[0].1.in_scope = Some(false);
    }
    let resolve = |_: &str, _: &std::path::Path| None;
    let mut profiles = super::super::prepare::profiles(&files, &resolve);
    let profile = profiles.pop().unwrap();
    let path = profile.files.keys().next().unwrap();
    let initials = SharedInitials::new();
    let mut first = super::super::evaluator::new(&profile.files, &resolve, false);
    first.modules.share_initials(Arc::clone(&initials));
    let environment = first.module_environment(path);
    assert_eq!(environment, 0);
    assert!(
        matches!(first.scopes[environment].get("prefix"), Some(query_annotation::evaluate::Value::Prefix(text, true, None)) if text == "/* operators */ SELECT 1")
    );
    first.scopes[environment].insert(
        "prefix".into(),
        query_annotation::evaluate::Value::Prefix("/* changed */ SELECT 2".into(), true, None),
    );
    assert_eq!(first.module_environment(path), environment);
    for preexisting in [false, true] {
        let mut next = super::super::evaluator::new(&profile.files, &resolve, false);
        if preexisting {
            next.environment(crate::fx::FxHashMap::from_iter([(
                "prefix".into(),
                query_annotation::evaluate::Value::Unknown,
            )]));
        }
        next.modules.share_initials(Arc::clone(&initials));
        let module = next.module_environment(path);
        let Some(query_annotation::evaluate::Value::Function(_, _, captured)) =
            next.scopes[module].get("queryText")
        else {
            panic!("saved arrow must retain its module closure");
        };
        assert_eq!(*captured, module);
        assert!(
            matches!(next.scopes[*captured].get("prefix"), Some(query_annotation::evaluate::Value::Prefix(text, true, None)) if text == "/* operators */ SELECT 1")
        );
    }
}

#[test]
fn excluded_self_contained_module_initializes_once_and_keeps_closure_bindings() {
    let mut files = prepared_fixture(
        "helper-tracing-shared-initials",
        &["first", "second", "helper", "unused"],
    );
    for (path, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
        if path.ends_with("helper.mts") || path.ends_with("unused.mts") {
            file.query_annotation[0].1.in_scope = Some(false);
        }
    }
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let path = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&path).then_some(path)
    };
    let run = |shared, threads| {
        let observer = InvocationObserver::new(true);
        let mut profiles = super::super::prepare::profiles(&files, &resolve);
        assert_eq!(profiles.len(), 1);
        let profile = profiles.pop().unwrap();
        assert_eq!(profile.roots.len(), 2);
        let initials = SharedInitials::new();
        let events = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                use rayon::prelude::*;
                let mut results = profile
                    .roots
                    .par_iter()
                    .map(|path| {
                        with_observer(Some(Arc::clone(&observer)), || {
                            let mut evaluator = super::super::evaluator::new(
                                &profile.files,
                                &resolve,
                                profile.untrusted_array_intrinsics,
                            );
                            if shared {
                                evaluator.modules.share_initials(Arc::clone(&initials));
                            }
                            evaluator.run(path);
                            (path.clone(), evaluator.events)
                        })
                    })
                    .collect::<Vec<_>>();
                results.sort_by(|left, right| left.0.cmp(&right.0));
                results
            });
        (events, observer.snapshot().work)
    };
    let (baseline, baseline_work) = run(false, 1);
    assert_eq!(baseline_work["postgres.annotation.module"], 4);
    for threads in [1, 4] {
        let (cached, work) = run(true, threads);
        assert!(
            baseline == cached,
            "shared initialization changed execution events"
        );
        assert_eq!(work["postgres.annotation.module"], 3);
        assert_eq!(work["postgres.annotation.shared_module_initializations"], 1);
        assert!(cached.iter().all(|(_, events)| events.values().flatten().any(|(_, value)| {
            matches!(value, query_annotation::evaluate::Value::Prefix(text, true, _) if text == "/* shared */ SELECT 1")
        })));
    }
}
