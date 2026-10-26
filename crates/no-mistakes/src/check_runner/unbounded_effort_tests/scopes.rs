use super::*;

#[test]
fn unbounded_effort_executor_alias_unknown_uses_call_line_suppression() {
    let report = run(".no-mistakes.yml", false);
    let findings = report
        .rules
        .iter()
        .filter(|row| row.file == "src/executor-alias.ts")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert!(findings[0].message.contains("unanalyzable at effort low"));
    let advice = run(".no-mistakes-advise.yml", false);
    let advice = advice
        .advisories
        .iter()
        .filter(|row| row.file == "src/executor-alias.ts")
        .collect::<Vec<_>>();
    assert_eq!(advice.len(), 1, "{advice:?}");
    assert_eq!(advice[0].line, 4);
    assert!(!run(".no-mistakes-ignore.yml", false)
        .rules
        .iter()
        .any(|row| row.file == "src/executor-alias.ts"));
}

#[test]
fn unbounded_effort_narrows_files_profiles_and_input_synthesis() {
    let root = root().parent().unwrap().join("effort-narrow");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all_with_sql_verdicts(root, None, None, false, false, false).unwrap()
    };
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    let snapshot = observer.snapshot();
    assert_eq!(
        snapshot.work.get("parse.files"),
        Some(&1),
        "{:?}",
        snapshot.work
    );
    assert_eq!(
        snapshot.work.get("postgres.sql_identifier.syntax_files"),
        Some(&1)
    );
    assert_eq!(
        snapshot.work.get("postgres.sql_identifier.syntax_inputs"),
        Some(&1)
    );
}

#[test]
fn unbounded_effort_scoped_literals_are_clear_and_loaders_remain_unknown() {
    let report = run(".no-mistakes-scoped.yml", false);
    // The saved fixture's historical label records the former factory false positive.
    // Its configured immutable factory and directly typed literal calls are now proven.
    for (file, expected) in [
        ("src/scoped.ts", Vec::new()),
        ("src/loaders.ts", vec![1, 4, 8]),
    ] {
        let lines = report
            .rules
            .iter()
            .filter(|row| row.file == file)
            .map(|row| row.line)
            .collect::<Vec<_>>();
        assert_eq!(lines, expected, "{file}: {:?}", report.rules);
    }
    assert!(report
        .rules
        .iter()
        .all(|row| row.message.contains("unanalyzable at effort low")));
    assert_eq!(
        run(".no-mistakes-inactive.yml", false).rules,
        run(".no-mistakes.yml", false).rules
    );
}
#[test]
fn unbounded_effort_project_scope_prunes_other_sources_before_parse() {
    let root = root().parent().unwrap().join("effort-narrow");
    let config = root.join(".no-mistakes-project.yml");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all_with_sql_verdicts(root, Some(config), None, false, false, false).unwrap()
    };
    assert!(report.rules.is_empty());
    assert_eq!(observer.snapshot().work.get("parse.files"), Some(&1));
}

#[test]
fn unbounded_effort_loose_member_selection_does_not_select_unrelated_loads() {
    let report = run(".no-mistakes-loose.yml", false);
    assert_eq!(report.rules.len(), 1, "{:?}", report.rules);
    assert_eq!(report.rules[0].file, "src/loose.ts");
    assert_eq!(report.rules[0].line, 4);
}

#[test]
fn unbounded_effort_namespace_members_keep_max_query_only_selection() {
    let report = run(".no-mistakes-read-only.yml", false);
    assert_eq!(report.rules, run(".no-mistakes-read-max.yml", false).rules);
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    assert!(report.advisories.is_empty());
}
