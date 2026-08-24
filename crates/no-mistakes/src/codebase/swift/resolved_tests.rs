use super::*;

const BASE: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/base.json");
const FORMATTING_ONLY: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/formatting-only.json");
const VERSION_CHANGED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/version-changed.json");
const REVISION_CHANGED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/revision-changed.json");
const CHECKSUM_CHANGED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/checksum-changed.json");
const ADDED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/added.json");
const REMOVED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/removed.json");
const MALFORMED: &str =
    include_str!("../../../../../fixtures/test-plan/swift-resolved/fixture/malformed.json");
const UNSUPPORTED_SCHEMA: &str = include_str!(
    "../../../../../fixtures/test-plan/swift-resolved/fixture/unsupported-schema.json"
);

#[test]
fn pins_diff_on_version_revision_checksum_add_remove_and_ignore_formatting() {
    let base = parse_resolved_pins(BASE).unwrap();
    let same = parse_resolved_pins(FORMATTING_ONLY).unwrap();
    assert!(diff_resolved_pins(&base, &same).is_empty());
    for changed in [VERSION_CHANGED, REVISION_CHANGED, CHECKSUM_CHANGED] {
        assert_eq!(
            diff_resolved_pins(&base, &parse_resolved_pins(changed).unwrap()),
            ["example"]
        );
    }
    assert_eq!(
        diff_resolved_pins(&base, &parse_resolved_pins(ADDED).unwrap()),
        ["added"]
    );
    assert_eq!(
        diff_resolved_pins(&base, &parse_resolved_pins(REMOVED).unwrap()),
        ["removed"]
    );
}

#[test]
fn pins_reject_malformed_and_unsupported_resolved_schemas() {
    assert_eq!(
        parse_resolved_pins(MALFORMED).unwrap_err(),
        SwiftResolvedDiagnostic::Malformed
    );
    assert_eq!(
        parse_resolved_pins(UNSUPPORTED_SCHEMA).unwrap_err(),
        SwiftResolvedDiagnostic::UnsupportedSchema
    );
}
