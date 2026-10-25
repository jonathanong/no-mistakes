use super::*;

#[test]
fn literal_ambient_modules_prove_only_unique_classes_owned_by_the_importing_project() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("ambient-module-targets/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        ["src/main.ts:3 UnusedError"],
        "{report}"
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/opaque.ts:5 unanalyzable",
            "src/opaque.ts:6 unanalyzable",
            "src/opaque.ts:7 unanalyzable",
            "src/opaque.ts:8 unanalyzable",
        ],
        "{report}"
    );
}
