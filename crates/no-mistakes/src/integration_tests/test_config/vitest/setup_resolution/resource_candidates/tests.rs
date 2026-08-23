use super::*;

#[test]
fn resource_candidates_follow_graph_path_bases() {
    let root = Path::new("/repo");
    let setup = Path::new("/repo/packages/unit/setup.ts");
    for (path, expected) in [
        (
            ResourcePath {
                value: "resources/data.json".to_string(),
                base: ResourcePathBase::AnalysisRoot,
            },
            PathBuf::from("/repo/resources/data.json"),
        ),
        (
            ResourcePath {
                value: "./local.json".to_string(),
                base: ResourcePathBase::SourceModule,
            },
            PathBuf::from("/repo/packages/unit/local.json"),
        ),
        (
            ResourcePath {
                value: "/tmp/absolute.json".to_string(),
                base: ResourcePathBase::AnalysisRoot,
            },
            PathBuf::from("/tmp/absolute.json"),
        ),
    ] {
        assert_eq!(resolve(root, setup, path), expected);
    }
}
