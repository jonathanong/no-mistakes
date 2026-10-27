use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "version-pin-consistency";

fn case(
    label: &'static str,
    scenario: &'static str,
    config: &'static str,
) -> (&'static str, PathBuf, PathBuf) {
    (
        label,
        rules_path(RULE, "fixture", scenario),
        rules_path(RULE, "configs", config),
    )
}

#[test]
fn version_pin_consistency_json_matches_snapshot() {
    let cases = vec![
        case("pass", "pass", "lychee.yml"),
        case("mismatch", "fail", "lychee.yml"),
        case("skip_absent", "skip-absent", "lychee.yml"),
        case("empty_options", "pass", "empty.yml"),
        case("missing_source_key", "missing-source-key", "lychee.yml"),
        case("non_string_pin", "non-string-pin", "lychee.yml"),
        case("missing_capture", "missing-capture", "lychee.yml"),
        case("invalid_toml", "invalid-toml", "lychee.yml"),
        case("json_source", "json-source", "json-source.yml"),
        case("yaml_source", "yaml-source", "yaml-source.yml"),
        case("nested_dotted_key", "nested-dotted-key", "nested-node.yml"),
        case(
            "one_capture_required",
            "matching-short-anchor",
            "no-capture.yml",
        ),
        case("anchor_only", "fail", "include-anchor.yml"),
        case("source_only", "fail", "include-source.yml"),
        case("exclude_anchor", "fail", "exclude-anchor.yml"),
        case("dot_slash", "fail", "dot-slash.yml"),
        case("disable_file", "disable-file", "lychee.yml"),
        case(
            "invalid_pattern",
            "matching-short-anchor",
            "invalid-pattern.yml",
        ),
        case("two_groups", "matching-short-anchor", "two-groups.yml"),
        case("empty_anchor", "pin-mismatch", "empty-anchor.yml"),
        case("empty_label", "pin-mismatch", "empty-label.yml"),
        case("pin_kind_true", "pin-kind-true", "lychee.yml"),
        case("pin_kind_null", "pin-kind-null", "yaml-source.yml"),
        case("pin_kind_array", "pin-kind-array", "yaml-source.yml"),
        case("pin_kind_object", "pin-kind-object", "yaml-source.yml"),
        case("undotted_scalar", "undotted-scalar", "undotted-scalar.yml"),
        case("empty_toml", "empty-toml", "lychee.yml"),
        case("invalid_yaml", "invalid-yaml", "yaml-source.yml"),
        case("jsonc_source", "jsonc-source", "jsonc-source.yml"),
        case("tagged_yaml", "tagged-yaml", "yaml-source.yml"),
        case(
            "optional_capture",
            "optional-capture",
            "optional-capture.yml",
        ),
        case("missing_key_line", "missing-key-line", "lychee.yml"),
        case(
            "undotted_mapping",
            "undotted-mapping",
            "undotted-mapping.yml",
        ),
        case("include_miss", "fail", "include-miss.yml"),
        case(
            "invalid_pattern_anchor_only",
            "matching-short-anchor",
            "invalid-pattern-anchor.yml",
        ),
        case(
            "anchor_only_missing_key",
            "missing-source-key",
            "include-anchor.yml",
        ),
        case(
            "anchor_only_non_string",
            "non-string-pin",
            "include-anchor.yml",
        ),
        case("empty_source_file", "fail", "empty-source.yml"),
        case("unicode_key", "unicode-key", "unicode-key.yml"),
        case(
            "source_only_missing_key",
            "missing-source-key",
            "include-source.yml",
        ),
        case(
            "source_only_non_string",
            "non-string-pin",
            "include-source.yml",
        ),
        case(
            "exclude_anchor_missing_key",
            "missing-source-key",
            "exclude-anchor.yml",
        ),
        case("mapping_key_line", "mapping-key-line", "node-versions.yml"),
        case("multibyte_key", "multibyte-key", "node-versions.yml"),
        case(
            "disable_mapping_key",
            "disable-mapping-key",
            "node-versions.yml",
        ),
        case("later_stale_capture", "later-stale-capture", "lychee.yml"),
        case(
            "all_matching_captures",
            "all-matching-captures",
            "lychee.yml",
        ),
        // `../outside.toml` is outside the fixture; its pin must not appear.
        case("parent_escape", "external-pin/repo", "parent-escape.yml"),
        case("literal_dotted", "literal-dotted", "dotted-remainder.yml"),
        case(
            "disable_dotted_key",
            "disable-dotted-key",
            "dotted-remainder.yml",
        ),
        case(
            "unavailable_report",
            "unavailable-source",
            "unavailable-report.yml",
        ),
        case(
            "unavailable_advise",
            "unavailable-source",
            "unavailable-advise.yml",
        ),
        case(
            "unavailable_ignore",
            "unavailable-source",
            "unavailable-ignore.yml",
        ),
        case(
            "unavailable_missing_report",
            "unavailable-source",
            "unavailable-missing-report.yml",
        ),
        case(
            "unavailable_missing_advise",
            "unavailable-source",
            "unavailable-missing-advise.yml",
        ),
        case(
            "unavailable_missing_ignore",
            "unavailable-source",
            "unavailable-missing-ignore.yml",
        ),
        case(
            "unavailable_unselected",
            "unavailable-source",
            "unavailable-unselected.yml",
        ),
        case(
            "unavailable_mixed_report",
            "unavailable-source",
            "unavailable-mixed-report.yml",
        ),
        case(
            "unavailable_mixed_advise",
            "unavailable-source",
            "unavailable-mixed-advise.yml",
        ),
        case(
            "unavailable_mixed_ignore",
            "unavailable-source",
            "unavailable-mixed-ignore.yml",
        ),
        case(
            "unreadable_source_for_anchor",
            "unavailable-source",
            "unavailable-unreadable-source-anchor.yml",
        ),
        case(
            "suppress_disabled_report",
            "unavailable-source",
            "unavailable-suppress-disabled-report.yml",
        ),
        case(
            "suppress_disabled_advise",
            "unavailable-source",
            "unavailable-suppress-disabled-advise.yml",
        ),
        case(
            "suppress_line_report",
            "unavailable-source",
            "unavailable-suppress-line-report.yml",
        ),
        case(
            "suppress_line_advise",
            "unavailable-source",
            "unavailable-suppress-line-advise.yml",
        ),
        case(
            "selected_versions_report",
            "unavailable-source",
            "unavailable-selected-versions-report.yml",
        ),
        case(
            "selected_versions_advise",
            "unavailable-source",
            "unavailable-selected-versions-advise.yml",
        ),
        case(
            "selected_versions_ignore",
            "unavailable-source",
            "unavailable-selected-versions-ignore.yml",
        ),
        case(
            "selected_empty_report",
            "unavailable-source",
            "unavailable-selected-empty-report.yml",
        ),
        case(
            "selected_empty_advise",
            "unavailable-source",
            "unavailable-selected-empty-advise.yml",
        ),
        case(
            "selected_empty_ignore",
            "unavailable-source",
            "unavailable-selected-empty-ignore.yml",
        ),
        case(
            "failed_dependency",
            "unavailable-source",
            "unavailable-failed-dependency.yml",
        ),
        case(
            "source_utf8_report",
            "unavailable-source",
            "unavailable-source-utf8-report.yml",
        ),
        case(
            "source_utf8_advise",
            "unavailable-source",
            "unavailable-source-utf8-advise.yml",
        ),
        case(
            "source_utf8_ignore",
            "unavailable-source",
            "unavailable-source-utf8-ignore.yml",
        ),
        case(
            "anchor_utf8_report",
            "unavailable-source",
            "unavailable-anchor-utf8-report.yml",
        ),
        case(
            "anchor_utf8_advise",
            "unavailable-source",
            "unavailable-anchor-utf8-advise.yml",
        ),
        case(
            "anchor_utf8_ignore",
            "unavailable-source",
            "unavailable-anchor-utf8-ignore.yml",
        ),
    ];
    assert_saved_fixture("version_pin_consistency", &cases);
}
