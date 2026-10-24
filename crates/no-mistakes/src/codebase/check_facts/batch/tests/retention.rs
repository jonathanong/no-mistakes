use super::*;

#[test]
fn dispatched_batch_programs_retire_before_returning_to_the_ambient_worker() {
    let fixture = fixture();
    let root = normalize(fixture.path());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();

    pool.install(|| {
        crate::ast::with_request_parse_cache(|| {
            let ambient = root.join("ambient.ts");
            let path = root.join("current.ts");
            let outer = AnalysisSession::disabled();
            // Cache ambient.ts under its own key so the public call must dispatch
            // current.ts to a worker while the ambient request cache is active.
            warm_ambient(&outer, &root, &ambient);
            let cache = crate::ast::current_request_parse_cache().unwrap();
            assert!(cache.contains_path(&ambient));
            assert!(!cache.contains_path(&path));

            let session = AnalysisSession::disabled();
            let inputs = requests(&session, &root, &path);
            let sources = inputs[0].sources.clone();
            crate::ast::begin_parse_count_this_thread(&root);
            let facts = collect_check_fact_batch_with_session(&session, inputs);
            let counts = crate::ast::finish_parse_count(&root);

            assert_eq!(facts.len(), 2);
            for map in &facts {
                assert_current(&map.ts[&path]);
            }
            assert!(facts[1].ts[&path].symbols.is_some());
            assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
            assert_eq!(
                counts.len(),
                1,
                "only the demanded current program may parse: {counts:#?}"
            );
            assert_eq!(sources.physical_read_count(), 1);
            assert_eq!(
                crate::ast::request_parse_cache_len(),
                1,
                "only the original ambient program may remain"
            );
            assert!(cache.contains_path(&ambient));
            assert!(
                !cache.contains_path(&path),
                "the dispatched program must not enter the ambient cache"
            );
            assert_ambient_restored(&outer, &ambient);
        })
    });
}
