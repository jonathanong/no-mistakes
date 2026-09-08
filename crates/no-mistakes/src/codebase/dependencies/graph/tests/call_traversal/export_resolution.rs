use super::*;

#[test]
fn call_export_resolution_keeps_missing_facts_and_nonvisible_stars_unknown() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("call-traversal"));
    let source = root.join("src/empty-star-barrel.mts");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::from_files(vec![source.clone()]);
    let inputs = call_resolution_inputs(&root, &tsconfig, &graph_files);
    let resolver = ImportResolver::new(&tsconfig);

    let missing_facts = TsFactMap::new();
    let indexes = CallableResolutionIndexes::default();
    assert!(matches!(
        resolve_exported_callable(
            &inputs,
            &missing_facts,
            &resolver,
            &source,
            "missing",
            &indexes,
            &mut Vec::new(),
        ),
        ExportedCallableResolution::Unknown
    ));
    assert!(matches!(
        indexes
            .exports
            .get(&(source.clone(), "missing".to_string()))
            .as_deref(),
        Some(ExportedCallableResolution::Unknown)
    ));

    let star_facts = TsFactMap::from([(
        source.clone(),
        TsFileFacts {
            star_reexport_specifiers: vec!["./empty-star-source.mts".to_string()],
            ..TsFileFacts::default()
        },
    )]);
    assert!(matches!(
        resolve_exported_callable(
            &inputs,
            &star_facts,
            &resolver,
            &source,
            "missing",
            &CallableResolutionIndexes::default(),
            &mut Vec::new(),
        ),
        ExportedCallableResolution::Unknown
    ));
}
