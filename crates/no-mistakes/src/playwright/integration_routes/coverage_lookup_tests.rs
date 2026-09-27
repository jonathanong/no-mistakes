#[test]
fn integration_routes_lookup_preserves_primary_errors_and_uses_supplemental_links() {
    use crate::codebase::dependencies::graph::{FallbackTsFactLookup, TsFactLookup};
    struct Empty;
    impl TsFactLookup for Empty {
        fn get_ts_facts(
            &self,
            _: &std::path::Path,
        ) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
            None
        }
    }
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let source = &settings.route_coverage_sources[0];
    let facts = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let mut supplemental = crate::codebase::ts_source::facts::TsFactMap::default();
    supplemental.integration_route_links = facts.integration_route_links;
    let visible = snapshot
        .paths_for(&root)
        .iter()
        .cloned()
        .collect::<crate::fx::PathSet>();
    assert!(Empty.integration_route_links(source).is_none());
    let lookup = FallbackTsFactLookup::new(&Empty, &supplemental, false, &[], &visible);
    assert!(lookup
        .integration_route_links(source)
        .unwrap()
        .as_ref()
        .unwrap()
        .iter()
        .any(|link| link.occurrence.value == "/healthz"));
    let mut primary = crate::codebase::check_facts::CheckFactMap::default();
    primary
        .integration_route_links
        .insert(source.clone(), Err("primary registration failure".into()));
    let lookup = FallbackTsFactLookup::new(&primary, &supplemental, true, &[], &visible);
    assert_eq!(
        lookup
            .integration_route_links(source)
            .unwrap()
            .as_ref()
            .unwrap_err(),
        "primary registration failure"
    );
}
#[test]
fn integration_routes_alias_mutations_and_destructuring_reject_only_affected_receivers() {
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let facts = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    assert!(
        facts
            .ts
            .get(&root.join("integration/aliased.ts"))
            .unwrap()
            .integration_route_occurrences
            .is_empty(),
        "either spelling of a mutated export invalidates its helper identity"
    );
    let retained = &facts
        .ts
        .get(&root.join("integration/receiver-patterns.ts"))
        .unwrap()
        .integration_route_occurrences;
    assert_eq!(
        retained
            .iter()
            .map(|fact| fact.occurrence.value.as_str())
            .collect::<Vec<_>>(),
        ["/other"]
    );
}
#[test]
fn integration_routes_registered_hooks_match_consumer_initialization_without_uncalled_credit() {
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let facts = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let hook = &facts
        .ts
        .get(&root.join("integration/hook-owned.ts"))
        .unwrap()
        .integration_route_occurrences;
    assert_eq!(
        hook.iter()
            .map(|fact| fact.occurrence.value.as_str())
            .collect::<Vec<_>>(),
        ["/hook-all", "/hook-page", "/hook-ready", "/local-ready"]
    );
    assert!(facts
        .ts
        .get(&root.join("integration/unexecuted-initializers.ts"))
        .unwrap()
        .integration_route_occurrences
        .is_empty());
}
