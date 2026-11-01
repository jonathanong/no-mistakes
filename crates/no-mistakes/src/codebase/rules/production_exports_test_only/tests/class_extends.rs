use super::findings;

#[test]
fn production_subclass_is_a_production_use() {
    let found = findings("class-extends-production");
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_only_subclass_still_reports_the_base() {
    let found = findings("class-extends-tests");
    let mut names: Vec<_> = found
        .iter()
        .map(|row| row.target.as_deref().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        ["Base", "OnlyHidden", "Shadow", "unanalyzable"],
        "{found:#?}"
    );
}
