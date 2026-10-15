use super::*;

fn alternatives_fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/effects-alternatives/fixture"),
    )
}

fn alternative_report(
    root: &Path,
    entry: &str,
    kind: &str,
    categories: &[String],
) -> EffectsReport {
    run(root, None, None, kind, Path::new(entry), categories, None).unwrap()
}

#[test]
fn physical_effect_occurrence_projects_every_matched_category() {
    let fixture = alternatives_fixture();
    for (projection, categories) in [
        ("all", vec![]),
        ("read", vec!["read".into()]),
        ("write", vec!["write".into()]),
    ] {
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(fixture.path().join(format!("expected-{projection}.json")))
                .unwrap(),
        )
        .unwrap();
        let report = alternative_report(fixture.path(), "src/entry.mts", "database", &categories);
        assert_eq!(serde_json::to_value(&report).unwrap(), expected);
        let reverse =
            alternative_report(fixture.path(), "src/reverse.mts", "database", &categories);
        assert_eq!(reverse.call_sites.len(), 1);
        assert_eq!(reverse.call_sites[0].line, report.call_sites[0].line);
        assert_eq!(reverse.call_sites[0].callee, report.call_sites[0].callee);
        assert_eq!(
            reverse.call_sites[0].alternatives,
            report.call_sites[0].alternatives
        );
        assert_eq!(reverse.by_category, report.by_category);
    }
}

#[test]
fn spelling_and_leaf_configuration_share_one_occurrence_without_duplicate_categories() {
    let fixture = alternatives_fixture();
    let mixed = alternative_report(fixture.path(), "src/entry.mts", "mixed", &[]);
    assert_eq!(mixed.call_sites.len(), 1);
    assert_eq!(mixed.call_sites[0].callee, "primary.read");
    assert_eq!(mixed.call_sites[0].category.as_deref(), Some("read"));
    let alternatives = mixed.call_sites[0].alternatives.as_ref().unwrap();
    assert_eq!(
        alternatives
            .iter()
            .map(|candidate| candidate.callee.as_str())
            .collect::<Vec<_>>(),
        ["primary.read", "receiver.read", "replica.write"]
    );
    assert_eq!(
        mixed.by_category,
        BTreeMap::from([
            ("read".into(), 1),
            ("uncategorized".into(), 1),
            ("write".into(), 1)
        ])
    );
    let read = alternative_report(fixture.path(), "src/entry.mts", "mixed", &["read".into()]);
    assert_eq!(read.call_sites.len(), 1);
    assert_eq!(read.call_sites[0].callee, "primary.read");
    assert!(read.call_sites[0].alternatives.is_none());
    assert_eq!(read.by_category, BTreeMap::from([("read".into(), 1)]));
    let same = alternative_report(fixture.path(), "src/same-category.mts", "database", &[]);
    assert_eq!(same.call_sites.len(), 1);
    assert_eq!(same.call_sites[0].alternatives.as_ref().unwrap().len(), 2);
    assert_eq!(same.by_category, BTreeMap::from([("read".into(), 1)]));
    let unknown = alternative_report(fixture.path(), "src/unknown.mts", "database", &[]);
    assert_eq!(unknown.call_sites.len(), 1);
    assert_eq!(unknown.call_sites[0].callee, "primary.read");
    assert!(unknown.call_sites[0].alternatives.is_none());
}

#[path = "alternative_provenance.rs"]
mod provenance;
#[path = "alternative_reachability.rs"]
mod reachability;
