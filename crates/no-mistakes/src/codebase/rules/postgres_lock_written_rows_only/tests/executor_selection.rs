use super::*;

fn saved_config(root: &Path, name: &str) -> NoMistakesConfig {
    let path = fixture("executor-selection")
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join(format!("executor-{name}.yml"));
    crate::config::v2::load_v2_config(root, Some(&path)).unwrap()
}

#[test]
fn executor_selection_options_preserve_scoped_transaction_factories() {
    let root = fixture("executor-selection");
    for (name, lines) in [
        ("defaults", vec![2, 3, 4, 9]),
        ("extras", vec![2, 3, 4, 5, 9]),
        ("replacement", vec![5, 9]),
        ("excluded-default", vec![3, 4, 5, 9]),
        ("all-excluded-scoped", vec![9]),
        ("project-inherited", vec![3, 4, 5, 9]),
    ] {
        let config = saved_config(&root, name);
        let mut findings: Vec<_> = check(&root, &config)
            .unwrap()
            .into_iter()
            .map(|finding| (finding.line, finding.target.unwrap()))
            .collect();
        findings.sort();
        let expected: Vec<_> = lines
            .into_iter()
            .map(|line| {
                (
                    line,
                    if line == 9 { CHECKS[0] } else { CHECKS[3] }.to_owned(),
                )
            })
            .collect();
        assert_eq!(findings, expected, "{name}");
    }
}

#[test]
fn unused_default_exclusion_keeps_other_default_executors_selected() {
    let root = fixture("executor-unused-exclusion");
    let findings = check(&root, &saved_config(&root, "unused-exclusion")).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[3]));
}

#[test]
fn resource_scopes_and_executor_options_route_locks_without_certifying_unknowns() {
    let root = fixture("resource-options");
    let path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/resource-options.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let mut findings = check(&root, &config).unwrap();
    findings.sort_by_key(|finding| finding.line);
    assert_eq!(findings.len(), 4, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
    for finding in &findings[1..] {
        assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
    }
    assert_eq!(findings[1].line, 10);
    assert_eq!(findings[2].line, 11);
    assert_eq!(findings[3].line, 13);
    assert!(findings.iter().all(|finding| finding.line != 12));
}

#[test]
fn const_executor_options_route_writes_and_reject_mutated_or_unwritten_controls() {
    let root = fixture("const-options");
    let path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/resource-options.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let findings = check(&root, &config).unwrap();
    let mut unwritten = findings
        .iter()
        .filter(|finding| finding.target.as_deref() == Some(CHECKS[0]))
        .map(|finding| finding.line)
        .collect::<Vec<_>>();
    unwritten.sort();
    assert_eq!(
        unwritten,
        [24, 30, 37, 44, 50, 57, 65, 74, 82, 99, 107],
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 91 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| finding.line > 21),
        "{findings:?}"
    );
}
