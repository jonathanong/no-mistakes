use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "structured-config-policy";

fn cfg(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "configs", name)
}

fn root(kind: &str) -> std::path::PathBuf {
    rules_path(RULE, kind, ".")
}

#[test]
fn structured_config_policy_json_matches_snapshot() {
    let fixture = root("fixture");
    let nested = root("nested-plugins");
    let selector = root("equals-file-selector");
    let ancestor = root("ancestor-override-subset");
    assert_saved_fixture(
        "structured_config_policy",
        &[
            // `<<: *defaults` is expanded, so the anchored banned key is present.
            ("merge_key", root("merge-key"), cfg("merge-key.yml")),
            (
                "required_banned",
                fixture.clone(),
                cfg("required-banned.yml"),
            ),
            (
                "value_assertions",
                fixture.clone(),
                cfg("value-assertions.yml"),
            ),
            ("unknown_kind", fixture.clone(), cfg("unknown-kind.yml")),
            ("invalid_yaml", fixture, cfg("invalid-yaml.yml")),
            ("jsonc_banned", root("jsonc"), cfg("jsonc-banned.yml")),
            ("jsonc_any_pass", root("jsonc"), cfg("jsonc-any-pass.yml")),
            ("jsonc_any_fail", root("jsonc"), cfg("jsonc-any-fail.yml")),
            (
                "prefix_single_file",
                root("prefix-single-file"),
                cfg("prefix-single-file.yml"),
            ),
            (
                "overrides_bind",
                root("overrides-bind"),
                cfg("overrides-bind.yml"),
            ),
            (
                "equals_indexed",
                selector.clone(),
                cfg("equals-indexed.yml"),
            ),
            ("equals_match_any", selector, cfg("equals-match-any.yml")),
            (
                "equals_missing_key",
                nested.clone(),
                cfg("equals-missing-key.yml"),
            ),
            (
                "equals_missing_file",
                nested.clone(),
                cfg("equals-missing-file.yml"),
            ),
            (
                "equals_empty_file",
                nested.clone(),
                cfg("equals-empty-file.yml"),
            ),
            // A custom message must not replace a comparison-file parse error.
            (
                "equals_parse_custom",
                nested.clone(),
                cfg("equals-parse-custom.yml"),
            ),
            ("plugins_drift", nested.clone(), cfg("plugins-drift.yml")),
            ("equals_parse_error", nested, cfg("equals-parse-error.yml")),
            (
                "ancestor_custom_keys",
                ancestor.clone(),
                cfg("ancestor-custom-keys.yml"),
            ),
            (
                "ancestor_per_child",
                ancestor.clone(),
                cfg("ancestor-per-child.yml"),
            ),
            (
                "ancestor_exclude",
                ancestor.clone(),
                cfg("ancestor-exclude.yml"),
            ),
            (
                "ancestor_sibling",
                ancestor.clone(),
                cfg("ancestor-sibling.yml"),
            ),
            (
                "ancestor_precedence",
                ancestor.clone(),
                cfg("ancestor-precedence.yml"),
            ),
            (
                "ancestor_malformed",
                ancestor.clone(),
                cfg("ancestor-malformed.yml"),
            ),
            (
                "ancestor_shapes",
                ancestor.clone(),
                cfg("ancestor-shapes.yml"),
            ),
            (
                "ancestor_target",
                ancestor.clone(),
                cfg("ancestor-target.yml"),
            ),
            // Rule include drops file.ts from the scanned list; inventory still sees it.
            (
                "ancestor_include_configs",
                ancestor.clone(),
                cfg("ancestor-include-configs.yml"),
            ),
            (
                "ancestor_custom_message",
                ancestor.clone(),
                cfg("ancestor-custom-message.yml"),
            ),
            // A custom message must not replace an ancestor parse error.
            (
                "ancestor_parse_custom",
                ancestor.clone(),
                cfg("ancestor-parse-custom.yml"),
            ),
            (
                "ancestor_when_skips",
                ancestor.clone(),
                cfg("ancestor-when-skips.yml"),
            ),
            (
                "ancestor_cycle_escape",
                ancestor.clone(),
                cfg("ancestor-cycle-escape.yml"),
            ),
            (
                "ancestor_curated",
                ancestor.clone(),
                cfg("ancestor-curated.yml"),
            ),
            (
                "ancestor_extends_inventory",
                ancestor,
                cfg("ancestor-extends-inventory.yml"),
            ),
        ],
    );
}
