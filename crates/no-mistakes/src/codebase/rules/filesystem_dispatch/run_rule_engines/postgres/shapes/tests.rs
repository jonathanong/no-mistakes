use super::*;
#[test]
fn standalone_dispatch_prepares_the_same_shape_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/prepared"),
    );
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = [root.join("src/builders.ts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    assert_eq!(
        shape_policy(&root, &config, &files, &sources, None)
            .unwrap()
            .len(),
        2
    );
}
