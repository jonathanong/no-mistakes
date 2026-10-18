use super::*;

#[test]
fn parallel_distinct_projections_share_canonical_successes_and_failures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/signature-impact")
        .canonicalize()
        .unwrap();
    let common = [
        root.join("consumer.mts"),
        root.join("utils.mts"),
        root.join("missing.mts"),
    ];
    let projections = (0..16)
        .map(|index| {
            common
                .iter()
                .cloned()
                .chain([root.join(format!("absent-{index}.mts"))])
                .collect::<crate::fx::PathSet>()
        })
        .collect::<Vec<_>>();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let barrier = std::sync::Barrier::new(projections.len());
    std::thread::scope(|scope| {
        for visible in &projections {
            scope.spawn(|| {
                barrier.wait();
                let normalized = session.normalized_visible(visible);
                assert!(normalized.contains_visible(&root.join("utils.mts")));
                assert!(normalized.contains_visible(&root.join("missing.mts")));
            });
        }
    });
    let work = observer.snapshot().work;
    // Different universes normalize separately, while overlapping successful
    // and failed canonicalization attempts retain one request-wide identity.
    assert_eq!(work["resolver.visible_normalizations"], 16);
    assert_eq!(work["resolver.visible_canonicalize_calls"], 19);
}
