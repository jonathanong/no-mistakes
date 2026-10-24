use super::*;

#[test]
fn prepared_story_errors_use_authoritative_causes_and_sources() {
    let root = normalize_path(&fixture("malformed-stories"));
    let story = normalize_path(&root.join("web/stories/Broken.stories.tsx"));
    let disabled =
        normalize_path(&fixture("malformed-selected").join("components/FileDisabled.tsx"));
    let mut shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![story.clone(), disabled.clone()],
        CheckFactPlan {
            storybook: true,
            source: true,
            ..Default::default()
        },
    );
    let stories = std::collections::BTreeSet::from([story.clone()]);
    let causes = [
        (String::new(), "could not be analyzed,"),
        ("  ".into(), "could not be analyzed,"),
        (
            "cause\nprivate detail".into(),
            "could not be analyzed (cause),",
        ),
        (
            format!("failed to read {}: unavailable", story.display()),
            "could not be analyzed (failed to read web/stories/Broken.stories.tsx: unavailable),",
        ),
    ];
    for (cause, expected) in causes {
        std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap())
            .unwrap()
            .parse_error = Some(cause.clone());
        let (incomplete, findings) =
            diagnostics::malformed_stories(&root, &shared, &stories, false);
        assert!(incomplete);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains(expected), "{findings:#?}");
        assert!(!findings[0].message.contains("()"));
        assert!(!findings[0].message.contains("private detail"));
        assert!(!findings[0].message.contains(root.to_str().unwrap()));
        assert_eq!(
            shared.ts.get(&story).unwrap().parse_error.as_ref(),
            Some(&cause)
        );
    }
    // The on-disk story has no exemption; prepared text alone is authoritative.
    let source = shared.ts.get(&disabled).unwrap().source.clone();
    std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap())
        .unwrap()
        .source = source;
    let (incomplete, findings) = diagnostics::malformed_stories(&root, &shared, &stories, false);
    assert!(incomplete);
    assert!(findings.is_empty());
    assert_eq!(
        diagnostics::malformed_stories(&root, &shared, &stories, true)
            .1
            .len(),
        1
    );
    std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap())
        .unwrap()
        .source = None;
    assert_eq!(
        diagnostics::malformed_stories(&root, &shared, &stories, false)
            .1
            .len(),
        1
    );
}

#[test]
fn unreadable_story_reports_repo_relative_read_error_through_public_checks() {
    let root = fixture("unreadable-story");
    let cfg = crate::config::v2::load_v2_config(&root, None).unwrap();
    let direct = check(&root, &cfg, None).unwrap();
    assert_eq!(direct.len(), 1, "{direct:#?}");
    assert_eq!(direct[0].file, "Card.stories.tsx");
    assert_eq!(direct[0].target.as_deref(), Some("unanalyzable"));
    assert!(direct[0]
        .message
        .contains("failed to read Card.stories.tsx:"));
    assert!(!direct[0].message.contains(root.to_str().unwrap()));
    assert_eq!(
        direct,
        crate::codebase::rules::run_check(&root, None, None).unwrap()
    );
    let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(report["rules"], serde_json::to_value(direct).unwrap());
}

#[test]
fn malformed_recovered_story_facts_cannot_prove_coverage_or_namespace_errors() {
    let root = normalize_path(&fixture("malformed-stories"));
    let story = normalize_path(&root.join("web/stories/Card.stories.tsx"));
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let mut shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            source: true,
            ..Default::default()
        },
    );
    let stories = std::collections::BTreeSet::from([story.clone()]);
    let resolver = empty_resolver(&root);
    let keys = coverage::all_react_component_keys(&root, &shared);
    assert!(
        !coverage::directly_covered_components(&root, &shared, &stories, &resolver, &keys)
            .is_empty()
    );
    let facts = std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap()).unwrap();
    facts.parse_error = Some("recovered parse failure".into());
    facts.storybook.as_mut().unwrap().used_runtime_imports[0].namespace = true;
    assert!(
        findings::namespace_import_findings(&root, &root, &shared, &stories, &resolver).is_empty()
    );
    std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap())
        .unwrap()
        .storybook
        .as_mut()
        .unwrap()
        .used_runtime_imports[0]
        .namespace = false;
    assert!(
        coverage::directly_covered_components(&root, &shared, &stories, &resolver, &keys)
            .is_empty()
    );
    assert!(
        !diagnostics::malformed_stories(&root, &shared, &stories, false)
            .1
            .is_empty()
    );
    std::sync::Arc::get_mut(shared.ts.get_mut(&story).unwrap())
        .unwrap()
        .parse_error = None;
    assert!(!diagnostics::malformed_stories(&root, &shared, &stories, false).0);
    assert!(!diagnostics::malformed_stories(&root, &CheckFactMap::default(), &stories, false).0);
}
