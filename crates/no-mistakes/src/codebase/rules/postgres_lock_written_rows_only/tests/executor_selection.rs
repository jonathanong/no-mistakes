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
        .filter(|finding| finding.line <= 110 && finding.target.as_deref() == Some(CHECKS[0]))
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
    for line in [114, 120, 126] {
        assert!(
            findings.iter().any(|finding| finding.line == line
                && matches!(
                    finding.target.as_deref(),
                    Some("row-lock-unwritten-relation" | "unanalyzable")
                )),
            "{line}: {findings:?}"
        );
    }
    assert!(
        findings
            .iter()
            .all(|finding| finding.line > 21 && finding.line < 133),
        "{findings:?}"
    );
}

#[test]
fn executor_options_reject_conflicts_and_preserve_independent_receivers() {
    let root = fixture("options-coverage");
    let path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/resource-options.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let mut findings = check(&root, &config).unwrap();
    findings.sort_by_key(|finding| finding.line);
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.target.as_deref()))
            .collect::<Vec<_>>(),
        [
            (4, Some(CHECKS[0])),
            (18, Some(CHECKS[0])),
            (43, Some(CHECKS[0])),
        ],
        "{findings:?}"
    );
}

#[test]
fn executor_option_metadata_rejects_conflicting_overrides_and_ended_aliases() {
    use crate::codebase::postgres::embedded::{
        extract_embedded_sql_from_source, EmbeddedSqlOptions,
    };
    let path = fixture("options-coverage").join("src/query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["beginTransaction".into()], &[])
        .with_executor_option_properties(&["query".into()]);
    options.track_transaction_helpers = true;
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    for line in [7, 8, 9, 10, 11, 12, 14, 27, 46] {
        let call = facts.calls.iter().find(|call| call.line == line).unwrap();
        assert!(!call.executor_transaction_known, "{call:?}");
        assert!(call.transaction_id.is_none(), "{call:?}");
    }
    for (lock, write) in [(31, 39), (50, 53)] {
        let lock = facts.calls.iter().find(|call| call.line == lock).unwrap();
        let write = facts.calls.iter().find(|call| call.line == write).unwrap();
        assert!(write.executor_transaction_known, "{write:?}");
        assert!(lock.transaction_id.is_some(), "{lock:?}");
        assert_eq!(lock.transaction_id, write.transaction_id);
    }
    options.track_transaction_helpers = false;
    let disabled = extract_embedded_sql_from_source(&path, &source, &options);
    let call = disabled.calls.iter().find(|call| call.line == 7).unwrap();
    assert!(!call.executor_transaction_known);
    assert!(call.transaction_id.is_none());
}
