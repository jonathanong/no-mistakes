use super::*;

#[test]
fn constructor_aliases_preserve_nearest_project_scope_and_shadowing() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("scoped-construction-targets/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        [
            "packages/app/classes.ts:3 UnusedError",
            "src/classes.ts:3 UnusedError",
        ],
        "{report}"
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        ["packages/app/main.ts:5 unanalyzable",],
        "{report}"
    );
}
