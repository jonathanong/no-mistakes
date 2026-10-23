use super::*;

#[test]
fn parameter_type_annotations_keep_runtime_and_type_only_import_provenance() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("import-binding-type-projection"));
    let config = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            imports: true,
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let sites: Vec<_> = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.file == root.join("entry.ts") && site.source_callee == "run")
        .collect();
    assert_eq!(sites.len(), 3, "{sites:#?}");
    for caller in ["runtime", "declared", "inline"] {
        let site = sites
            .iter()
            .find(|site| site.caller.as_deref() == Some(caller))
            .unwrap();
        assert!(
            site.typed_from
                .contains(&("./types".to_string(), "Executor".to_string())),
            "{caller}: annotation resolution must preserve explicit imported provenance: {site:#?}",
        );
    }
}
