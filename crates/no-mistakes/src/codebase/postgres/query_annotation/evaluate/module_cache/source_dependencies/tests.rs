use super::*;

#[test]
fn cache_hits_replay_actual_dependencies_after_snapshot_restore() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-source-prerequisites")
        .canonicalize().unwrap();
    let broken = root.join("broken.mts");
    let module = root.join("cached.mts");
    let caller = root.join("cached-entry.mts");
    let sources = SourceDependencies::new(Arc::new([broken.clone()].into_iter().collect()));
    sources.begin(&module);
    sources.consult(&broken);
    sources.finish(&module);
    sources.begin(&caller);
    sources.consult(&broken);
    sources.finish(&caller);
    assert!(sources.0.as_ref().unwrap().lock().unwrap().modules[&caller].contains(&broken));
    let snapshot = sources.clone();
    sources
        .0
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .consulted
        .clear();
    snapshot.begin(&caller);
    snapshot.replay(&module);
    snapshot.finish(&caller);
    assert_eq!(sources.failures(), [broken.clone()].into_iter().collect());
    let state = sources.0.as_ref().unwrap().lock().unwrap();
    assert_eq!(state.modules[&caller], [broken].into_iter().collect());
}

#[test]
fn known_unfailed_and_unconsulted_sources_do_not_enter_dependencies() {
    let paths = crate::codebase::postgres::query_annotation::module_ids::tests::saved_paths();
    let sources = SourceDependencies::new(Arc::new([paths[0].clone()].into_iter().collect()));
    sources.begin(&paths[1]);
    sources.consult(&paths[1]);
    sources.finish(&paths[1]);
    sources.replay(&paths[1]);
    assert!(sources.failures().is_empty());
    assert!(SourceDependencies::default().failures().is_empty());
    assert!(SourceDependencies::new(Arc::default())
        .failures()
        .is_empty());
}
