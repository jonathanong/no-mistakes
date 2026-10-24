use super::*;

#[test]
fn nested_uncertainty_uses_each_policy_and_definite_findings_equal_max() {
    let contracts = || {
        vec![
            contract("flat", "required"),
            contract("scalar", "enum"),
            contract("nested", "nested"),
        ]
    };
    let max = run(vec![rule(Effort::Max, Unanalyzable::Report, contracts())]);
    assert_eq!(max.findings.len(), 2);
    for (policy, findings, advisories) in [
        (Unanalyzable::Report, 3, 0),
        (Unanalyzable::Advise, 2, 1),
        (Unanalyzable::Ignore, 2, 0),
    ] {
        let low = run(vec![rule(Effort::Low, policy, contracts())]);
        assert_eq!(low.findings.len(), findings, "{:?}", low.findings);
        assert_eq!(low.advisories.len(), advisories);
        let definite: Vec<_> = low
            .findings
            .iter()
            .filter(|finding| !finding.message.contains("unanalyzable at effort low"))
            .cloned()
            .collect();
        assert_eq!(definite, max.findings);
        let shape = low
            .findings
            .iter()
            .chain(&low.advisories)
            .find(|finding| finding.message.contains("unanalyzable at effort low"));
        if let Some(shape) = shape {
            assert_eq!(shape.target.as_deref(), Some("nested-to-nested"));
            assert!(shape.message.contains("unproven:"));
            assert!(shape.message.contains("actual payloads"));
            assert!(shape.message.contains("effort max"));
            assert_eq!(shape.line, 1);
        }
    }
}

#[test]
fn mixed_applications_keep_policies_and_custom_messages_separate() {
    let mut low = rule(
        Effort::Low,
        Unanalyzable::Advise,
        vec![contract("nested", "nested")],
    );
    low.message = Some("Coordinate services".into());
    let max = rule(
        Effort::Max,
        Unanalyzable::Report,
        vec![contract("flat", "required"), contract("nested", "nested")],
    );
    let result = run(vec![low, max]);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.advisories.len(), 1);
    assert!(result.advisories[0]
        .message
        .starts_with("Coordinate services: contract"));
    assert!(result.findings[0].message.contains("incompatible"));
}

#[test]
fn a_nested_consumer_is_not_silently_accepted() {
    let result = run(vec![rule(
        Effort::Low,
        Unanalyzable::Report,
        vec![contract("flat", "nested")],
    )]);
    assert_eq!(result.findings.len(), 1);
    assert!(result.findings[0].message.contains("consumer schema"));
}

#[test]
fn malformed_flat_schemas_stay_hard_errors_under_ignore() {
    for name in [
        "invalid",
        "badProperties",
        "badChild",
        "badItems",
        "notSchema",
    ] {
        for contract in [contract(name, "scalar"), contract("scalar", name)] {
            let result = run(vec![rule(
                Effort::Low,
                Unanalyzable::Ignore,
                vec![contract],
            )]);
            assert_eq!(result.findings.len(), 1);
            assert!(result.findings[0].message.contains("unproven"));
            assert!(result.advisories.is_empty());
        }
    }
}

#[test]
fn an_empty_contract_name_stays_a_hard_error() {
    let mut empty = contract("nested", "nested");
    empty.name.clear();
    let result = run(vec![rule(Effort::Low, Unanalyzable::Ignore, vec![empty])]);
    assert_eq!(result.findings.len(), 1);
    assert!(result.findings[0].message.contains("name must be nonempty"));
}
