use super::*;

#[test]
fn low_modes_alias_and_max_profiles_keep_their_policy_semantics() {
    let report = run(".no-mistakes-report.yml");
    let advise = run(".no-mistakes-advise.yml");
    let ignore = run(".no-mistakes-ignore.yml");
    let legacy = run(".no-mistakes-legacy-alias.yml");
    assert_eq!(
        report.rules, legacy.rules,
        "legacy policy alias changed findings"
    );
    assert_eq!(report.advisories, legacy.advisories);
    let source = std::fs::read_to_string(root().join("src/query.mts")).unwrap();
    let missing = marked_lines(&source, "EXPECT_MISSING");
    let unresolved = call_lines(&report.rules, "src/query.mts")
        .into_iter()
        .filter(|line| !missing.contains(line))
        .collect::<Vec<_>>();
    assert_eq!(call_lines(&advise.rules, "src/query.mts"), missing);
    assert_eq!(call_lines(&advise.advisories, "src/query.mts"), unresolved);
    assert!(advise
        .advisories
        .iter()
        .all(|row| row.file == "src/query.mts"));
    assert!(
        advise
            .advisories
            .iter()
            .all(|row| row.message.contains("query(\"/* name */ \" + sql)")),
        "advisories must carry the static rewrite: {:?}",
        advise.advisories
    );
    assert_eq!(call_lines(&ignore.rules, "src/query.mts"), missing);
    assert!(ignore.rules.iter().all(|finding| finding
        .message
        .contains("query execution must start with a /* name */ annotation")));
    assert!(ignore.advisories.is_empty(), "{:?}", ignore.advisories);

    let default_max = run(".no-mistakes-default-max.yml");
    let explicit_max = run(".no-mistakes-explicit-max.yml");
    assert_eq!(default_max.rules, explicit_max.rules);
    assert_eq!(default_max.advisories, explicit_max.advisories);
    assert!(
        !default_max.rules.is_empty(),
        "max must retain evaluator findings"
    );
    let root = root();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let mixed = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes-mixed.yml")),
            None,
            false,
            false,
            false,
        )
        .unwrap()
    };
    assert!(
        mixed.rules.iter().any(|row| {
            row.file == "src/query.mts" && row.message.contains("unanalyzable at effort low")
        }),
        "the low application must still report: {:?}",
        mixed.rules
    );
    for finding in &default_max.rules {
        assert!(
            mixed.rules.contains(finding),
            "max finding was lost: {finding:?}"
        );
    }
    let work = observer.snapshot().work;
    assert!(
        work.get("postgres.annotation.profiles")
            .copied()
            .unwrap_or(0)
            > 0,
        "the max application must request profiles: {work:#?}"
    );
}
