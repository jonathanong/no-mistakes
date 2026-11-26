use super::*;
use crate::codebase::structured_value::parse_structured_value_located;
use std::path::Path;

#[test]
fn partial_mappings_keep_explicit_paths_without_assuming_inherited_values() {
    let partial = fixture();

    assert!(conditions_known(
        &partial,
        &[PolicyWhen {
            key: "enabled".into()
        }]
    ));
    assert!(!conditions_known(
        &partial,
        &[PolicyWhen {
            key: "runtime.owner".into()
        }]
    ));
    assert_eq!(
        key(partial.value(), "runtime.rows", Some(&partial))
            .flatten()
            .and_then(Value::as_sequence)
            .map(Vec::len),
        Some(2)
    );

    for (selector, known) in [
        ("runtime.rows.0.owner", true),
        ("runtime.rows.1.owner", false),
        ("runtime.rows.2.owner", true),
        ("runtime.rows.[].owner", false),
        ("runtime.rows.[].missing", false),
        ("runtime.rows.[].[]", true),
        ("runtime", false),
    ] {
        let assertion = ValueAssertion {
            key: selector.into(),
            ..Default::default()
        };
        assert_eq!(assertion_known(&partial, &assertion), known, "{selector}");
    }

    let ancestors = ValueAssertion {
        kind: Some(AssertionKind::AncestorOverrideSubset),
        ..Default::default()
    };
    assert!(assertion_known(&partial, &ancestors));
    let ancestors_with_unknown_rules = ValueAssertion {
        rules_key: "runtime.rules".into(),
        ..ancestors
    };
    assert!(!assertion_known(&partial, &ancestors_with_unknown_rules));

    let shape = ValueAssertion {
        key: "runtime".into(),
        kind: Some(AssertionKind::Boolean),
        ..Default::default()
    };
    assert!(assertion_known(&partial, &shape));
}

fn fixture() -> std::sync::Arc<PartialYaml> {
    parse_structured_value_located(
        Path::new("partial-paths.yml"),
        include_str!(
            "../../../../../../../test-cases/structured-value/merge-keys/partial-paths.yml"
        ),
    )
    .unwrap_err()
    .partial
    .unwrap()
}

#[test]
fn partial_extends_only_reports_a_known_invalid_declaration() {
    let value = fixture();
    for selector in ["absent", "runtime.owner", "enabled"] {
        let assertion = ValueAssertion {
            kind: Some(AssertionKind::AncestorOverrideSubset),
            extends_key: selector.into(),
            ..Default::default()
        };
        let finding = super::super::ancestor_override_subset::validate_partial_extends(
            &value,
            "partial-paths.yml",
            &assertion,
        );
        assert!(finding.is_none(), "{selector}: {finding:?}");
    }
}
