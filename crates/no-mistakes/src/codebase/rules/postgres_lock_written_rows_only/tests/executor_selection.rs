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
