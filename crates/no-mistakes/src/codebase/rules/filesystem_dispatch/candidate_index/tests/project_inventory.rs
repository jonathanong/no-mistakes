use super::*;

#[test]
fn project_scoped_tsconfig_file_coverage_keeps_repository_inventory_tsconfig() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let project_root = root.join("src");
    let tsconfig = root.join("tsconfig.json");
    let source = project_root.join("index.ts");
    let files = vec![source.clone()];
    let inventory = Arc::new(vec![tsconfig.clone(), source.clone()]);
    let config = NoMistakesConfig {
        projects: [(
            "app".to_string(),
            Project {
                root: Some(project_root.to_string_lossy().into_owned()),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        rules: vec![
            RuleDef {
                rule: TSCONFIG_FILE_COVERAGE.to_string(),
                projects: vec!["app".to_string()],
                ..Default::default()
            },
            RuleDef {
                rule: crate::codebase::rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES.to_string(),
                projects: vec!["app".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let index = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &config,
        &files,
        &files,
        &[],
        Some(inventory),
    )
    .unwrap();

    let mut expected = vec![source.clone(), tsconfig];
    expected.sort();
    assert_eq!(index.candidates(TSCONFIG_FILE_COVERAGE), expected);
    assert_eq!(
        index.candidates(crate::codebase::rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES),
        std::slice::from_ref(&source)
    );
}
