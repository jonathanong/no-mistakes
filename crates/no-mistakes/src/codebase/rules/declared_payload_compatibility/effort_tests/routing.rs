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
fn consumer_only_nested_shape_anchors_both_channels_to_the_consumer_file() {
    let files = [
        root().join("schemas.json"),
        root().join("consumer-nested.json"),
    ];
    let sources = super::super::super::source_store_for_files(&files);
    for policy in [Unanalyzable::Report, Unanalyzable::Advise] {
        let mut pair = contract("flat", "nested");
        pair.consumer.file = "consumer-nested.json".into();
        pair.consumer.pointer = "/payload".into();
        let config = NoMistakesConfig {
            rules: vec![rule(Effort::Low, policy, vec![pair])],
            ..NoMistakesConfig::default()
        };
        let output =
            diagnostics_with_files_and_sources(&root(), &config, &files, &sources, false).unwrap();
        let expected = usize::from(policy == Unanalyzable::Report);
        assert_eq!(output.findings.len(), expected);
        assert_eq!(output.advisories.len(), 1 - expected);
        let finding = output
            .findings
            .iter()
            .chain(&output.advisories)
            .next()
            .unwrap();
        assert_eq!(finding.file, "consumer-nested.json");
        assert_eq!(finding.line, 1);
        assert!(finding.message.contains("consumer schema"));
        assert!(finding.message.contains("consumer-nested.json/payload"));
        assert!(finding.message.contains("$[\"profile\"]"));
    }
    assert_eq!(sources.physical_read_count(), 2);
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
