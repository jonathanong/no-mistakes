use super::*;
use crate::codebase::rules::source_store_for_files;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    specifier: String,
    importer: String,
    expected: Option<String>,
    consumer: Option<String>,
}

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("unconstructed-error-class")
        .parent()
        .unwrap()
        .join("package-resolution/fixture")
}

#[test]
fn saved_declaration_resolution_cases_preserve_unknowns_and_source_ownership() {
    let root = fixture();
    let cases_store = source_store_for_files(&[]);
    let text = cases_store.read_path(&root.join("cases.json")).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&text).unwrap();
    for case in cases {
        let sources = source_store_for_files(&[]);
        let importer = root.join(&case.importer);
        let consumer = case.consumer.as_ref().map(|path| root.join(path));
        let context = consumer
            .as_deref()
            .or_else(|| importer.parent())
            .unwrap_or(&root);
        let actual = resolve_in_context(&case.specifier, &importer, context, &sources);
        let expected = case
            .expected
            .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)));
        assert_eq!(actual, expected, "{}", case.name);
        let reads = sources.physical_read_count();
        let parses = sources.json_parse_count();
        assert_eq!(
            resolve_in_context(&case.specifier, &importer, context, &sources),
            actual,
            "{}",
            case.name
        );
        assert_eq!(
            sources.physical_read_count(),
            reads,
            "{} reread sources",
            case.name
        );
        assert_eq!(
            sources.json_parse_count(),
            parses,
            "{} reparsed manifests",
            case.name
        );
    }
}
