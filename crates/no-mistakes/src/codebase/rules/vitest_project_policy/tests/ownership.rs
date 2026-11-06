use super::*;

#[test]
fn legacy_borrowed_source_entrypoint_matches_diagnostics_and_custom_message() {
    for case in ["missing", "custom-advisory"] {
        let root = crate::test_support::rule_fixture_root("vitest-project-policy").join(case);
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        let files = vec![root.join("vitest.config.ts")];
        let findings = check_with_files(&root, &config, &files).unwrap();
        assert_eq!(findings, run(case).findings);
        assert_eq!(findings.len(), 1);
        if case == "custom-advisory" {
            assert_eq!(
                findings[0].message,
                "Declare both worker settings so project execution stays consistent."
            );
        }
    }
}

#[test]
fn absent_project_policy_and_unknown_inheritance_keep_distinct_outcomes() {
    let root =
        crate::test_support::rule_fixture_root("vitest-project-policy").join("policy-boundaries");
    for (case, complete) in [
        ("no-projects", true),
        ("neither-pair", true),
        ("root-nonliteral", false),
        ("unknown-inherits", false),
        ("opaque-inherits", false),
    ] {
        let files = vec![root.join(format!("{case}.ts"))];
        let sources = SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        ));
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))
                .unwrap();
        let output =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
        assert!(output.findings.is_empty(), "{case}: {:?}", output.findings);
        assert_eq!(
            output.incomplete.is_empty(),
            complete,
            "{case}: {:?}",
            output.incomplete
        );
        if case == "root-nonliteral" {
            assert!(output
                .incomplete
                .iter()
                .any(|failure| failure.property.as_deref() == Some("maxWorkers")));
        }
    }
}
