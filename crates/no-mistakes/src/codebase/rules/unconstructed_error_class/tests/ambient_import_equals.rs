use super::*;

#[test]
fn external_import_equals_cannot_turn_an_augmentation_into_global_class_proof() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("ambient-import-equals/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        Vec::<String>::new(),
        "{report}"
    );
    // The external require-style import makes its declaration an augmentation;
    // the internal namespace alias preserves the other file's ambient module.
    assert_eq!(
        reported_channel(&report, "advisories"),
        ["src/main.ts:4 unanalyzable"],
        "{report}"
    );
}
