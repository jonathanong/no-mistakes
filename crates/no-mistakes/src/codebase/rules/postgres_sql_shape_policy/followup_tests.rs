use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

#[test]
fn cooked_and_delayed_sql_initializers_use_physical_source_lines() {
    let root = fixture("review-followups");
    let path = root.join("src/source-map.ts");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 6, 8, 10, 12]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 6, 8, 10]
    );
}

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

#[test]
fn count_negation_and_ungrouped_having_preserve_existence_semantics() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/negated-count.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 5]
    );
    for finding in findings {
        assert_eq!(
            finding.message.contains("use NOT EXISTS"),
            [2, 3].contains(&finding.line),
            "{finding:?}"
        );
    }
}

#[test]
fn quoted_lowercase_count_keeps_the_builtin_schema_and_case_restrictions() {
    let root = fixture("review-followups");
    let paths = [root.join("sql/quoted-count.sql")];
    let findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: ['sql/**/*.sql']\nbannedShapes: [count-for-existence]\n"),
        &paths,
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3],
        "{findings:#?}"
    );
}

#[test]
fn empty_grouping_sets_remain_global_existence_probes() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/empty-grouping.sql")],
    )
    .unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5],
        "{findings:#?}"
    );
}

#[test]
fn filter_facts_are_unique_and_lenient_spans_preserve_suppression() {
    let root = fixture("review-followups");
    let sql = std::fs::read_to_string(root.join("sql/filter-once.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|s| s.not_in_subqueries.len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|s| s.count_existence_checks.len())
            .sum::<usize>(),
        1
    );
    let path = root.join("sql/lenient-locations.sql");
    let config = config_yaml(&format!("{BANNED}\nunanalyzableSql: ignore"));
    let mut findings = check_with_files(&root, &config, std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [3, 4],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(findings.iter().map(|f| f.line).collect::<Vec<_>>(), [4]);
}
