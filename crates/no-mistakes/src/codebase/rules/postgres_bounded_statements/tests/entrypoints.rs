use super::*;

#[test]
fn the_standalone_entry_points_prepare_their_own_facts() {
    let root = fixture_root();
    let files = [root.join("sql/invalid.sql"), root.join("schema.json")];
    let yaml =
        "schemaCatalogPath: schema.json\nsqlInclude: ['sql/invalid.sql']\nexclude: ['src/**']";
    let direct = check_with_files(&root, &config(yaml), &files).unwrap();
    assert_eq!(direct.len(), 3, "{direct:#?}");
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let shared = check_with_files_and_sources(&root, &config(yaml), &files, &sources).unwrap();
    assert_eq!(direct, shared);
}

#[test]
fn sql_that_cannot_be_analyzed_fails_closed_unless_ignored() {
    let root = fixture_root();
    let files = [
        root.join("sql/broken.sql"),
        root.join("src/dynamic.ts"),
        root.join("schema.json"),
    ];
    let base = "schemaCatalogPath: schema.json\nimportSpecifier: '@example/db'\nexecutorNames: [query, read, write]\nsqlInclude: ['sql/broken.sql']\n";
    let found: Vec<_> = check_with_files(&root, &config(base), &files)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.file, finding.message, finding.target.unwrap()))
        .collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|(_, _, target)| target == "unanalyzable"));
    assert!(found[0]
        .1
        .contains("SQL could not be analyzed for bounded statements"));
    assert!(found[1]
        .1
        .contains("not statically recoverable for bounded statements"));
    let ignored = format!("{base}unanalyzableSql: ignore\n");
    assert!(check_with_files(&root, &config(&ignored), &files)
        .unwrap()
        .is_empty());
}
