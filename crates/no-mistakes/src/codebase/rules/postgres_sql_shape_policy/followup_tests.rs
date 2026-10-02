use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

#[test]
fn review_followups_preserve_shapes_locations_and_semantics() {
    let root = fixture("review-followups");
    for (name, lines) in [
        ("having", vec![3, 4]),
        ("comments", vec![2]),
        ("unicode", vec![2, 3]),
        ("repeated", vec![2, 4]),
        ("casts", vec![1, 2]),
        ("filter", vec![2]),
        ("mutations", vec![2, 4]),
        ("not-count", vec![]),
        ("polarity", vec![1, 2, 3, 4, 5, 6]),
        ("cte", vec![1, 2]),
        ("operators", vec![2, 4]),
        ("count-edges", vec![1]),
    ] {
        let findings = check_with_files(
            &root,
            &config_yaml(BANNED),
            &[root.join(format!("sql/{name}.sql"))],
        )
        .unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            lines,
            "{name}: {findings:#?}"
        );
        if name == "polarity" {
            for finding in &findings {
                assert_eq!(
                    finding.message.contains("use NOT EXISTS"),
                    (2..=5).contains(&finding.line),
                    "{finding:?}"
                );
            }
        }
    }
}

#[test]
fn comment_and_operator_suppressions_use_actual_expression_lines() {
    let root = fixture("review-followups");
    let file = root.join("sql/suppress.sql");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 2);
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn embedded_shape_locations_are_rebased() {
    let root = fixture("review-followups");
    let findings =
        check_with_files(&root, &config_yaml(BANNED), &[root.join("src/query.ts")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![5, 6],
        "{findings:#?}"
    );
}

#[test]
fn unanalyzable_targets_are_enabled_shapes() {
    let root = fixture("fail-dynamic");
    for shape in ["not-in-subquery", "count-for-existence"] {
        let config = config_yaml(&format!("bannedShapes: [{shape}]"));
        let findings = check_with_files(&root, &config, &[root.join("src/query.ts")]).unwrap();
        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some(shape)));
    }
}
