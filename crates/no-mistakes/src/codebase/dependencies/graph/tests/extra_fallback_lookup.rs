#[test]
fn fallback_lookup_forwards_scan_helpers_through_primary_and_fallback() {
    struct Files(TsFactMap, Vec<PathBuf>);
    impl TsFactLookup for Files {
        fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
            self.0.get(path)
        }
        fn graph_files(&self) -> Option<&[PathBuf]> {
            Some(&self.1)
        }
    }

    let path = PathBuf::from("/repo/a.ts");
    let files = vec![path.clone()];
    let primary = Files(
        TsFactMap::from([(path.clone(), TsFileFacts::default())]),
        files.clone(),
    );
    let fallback = TsFactMap::new();
    let visible: crate::fx::PathSet = files.clone().into_iter().collect();
    let lookup = FallbackTsFactLookup::new(&primary, &fallback, false, &files, &visible);
    assert!(lookup.covers_ts_fact_plan(TsFactPlan::imports()));
    assert!(lookup.get_playwright_test_files(None).is_none());
    assert!(
        lookup
            .get_or_compute_app_selector_occurrences(&cache_settings(), false, &|| Ok(Vec::new()))
            .unwrap()
            .is_empty()
    );
    assert!(
        lookup
            .get_or_compute_app_text_targets(&cache_settings(), &|| Ok(Vec::new()))
            .unwrap()
            .is_empty()
    );
    assert!(
        lookup
            .get_or_compute_route_reachable_files(&cache_settings(), &|| Ok(Default::default()))
            .unwrap()
            .is_empty()
    );

    let mismatched: crate::fx::PathSet = [PathBuf::from("/repo/b.ts")].into_iter().collect();
    let lookup = FallbackTsFactLookup::new(&primary, &fallback, true, &files, &mismatched);
    assert!(
        lookup
            .get_or_compute_app_selector_occurrences(&cache_settings(), true, &|| Ok(Vec::new()))
            .unwrap()
            .is_empty()
    );
    assert!(lookup.get_ts_facts(&path).is_some());
}
