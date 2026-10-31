use super::findings;

#[test]
fn bundler_script_entry_is_a_production_use() {
    let found = findings("script-entry");
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn missing_script_entry_still_reports_the_default_export() {
    let found = findings("script-absent");
    let mut names: Vec<_> = found
        .iter()
        .map(|row| row.target.as_deref().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["default"], "{found:#?}");
}
