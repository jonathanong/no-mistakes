use super::*;

fn declarations(files: &[&str], config: &str) -> Diagnostics {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("vitest-project-policy").join("declared"),
    );
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(config))).unwrap();
    diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap()
}

#[test]
fn runtime_projects_keep_declaration_checks_without_incomplete_results() {
    let output = declarations(
        &[
            "vitest.config.ts",
            "helpers/projects.mts",
            "helpers/new-helper.mts",
            "helpers/clean.mts",
        ],
        ".no-mistakes.yml",
    );
    assert!(output.incomplete.is_empty());
    assert_eq!(output.findings.len(), 16, "{:?}", output.findings);
    assert!(output
        .findings
        .iter()
        .any(|finding| finding.file == "helpers/projects.mts" && finding.line == 5));
    assert!(output
        .findings
        .iter()
        .any(|finding| finding.file == "helpers/new-helper.mts"));
    assert!(!output
        .findings
        .iter()
        .any(|finding| finding.file == "helpers/clean.mts"));
    let root_lines = output
        .findings
        .iter()
        .filter(|finding| finding.file == "vitest.config.ts")
        .map(|finding| finding.line)
        .collect::<Vec<_>>();
    assert_eq!(root_lines, vec![4, 5, 6, 7, 8, 19, 20, 21, 22, 23]);
}

#[test]
fn helper_globs_only_use_prepared_visible_inventory() {
    let output = declarations(
        &["vitest.config.ts", "helpers/clean.mts"],
        ".no-mistakes.yml",
    );
    assert!(output
        .findings
        .iter()
        .all(|finding| finding.file == "vitest.config.ts"));
    assert!(output.incomplete.is_empty());
}

#[test]
fn declaration_suppressions_use_the_declaring_file_and_property_line() {
    for case in ["declared-line", "declared-next-line", "declared-file"] {
        let output = run(case);
        assert!(output.findings.is_empty(), "{case}: {:?}", output.findings);
        assert!(output.incomplete.is_empty(), "{case}");
    }
}

#[test]
fn additive_declarations_preserve_effective_policy_outcomes() {
    let files = ["vitest.config.ts", "helpers/projects.mts"];
    let before = declarations(&files, "effective.yml");
    let after = declarations(&files, "combined.yml");
    assert_eq!(before.incomplete, after.incomplete);
    assert!(!before.incomplete.is_empty());
    assert!(before.findings.is_empty());
    assert!(!after.findings.is_empty());
}

#[test]
fn source_only_policies_report_unavailable_sources() {
    for (config, failures) in [
        ("missing-declared-sources.yml", 2),
        ("malformed-declared-sources.yml", 1),
    ] {
        let output = declarations(&["malformed.ts"], config);
        assert!(output.findings.is_empty());
        assert_eq!(output.incomplete.len(), failures);
        assert!(output
            .incomplete
            .iter()
            .all(|failure| failure.property.as_deref() == Some("declared")));
    }
}

mod review;

#[test]
fn effective_and_declaration_checks_share_one_parse_per_source() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("vitest-project-policy").join("declared"),
    );
    crate::ast::begin_parse_count_this_thread(&root);
    let _ = declarations(
        &["vitest.config.ts", "helpers/projects.mts"],
        "combined.yml",
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 2, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
}

#[test]
fn invalid_declaration_options_are_rejected() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("vitest-project-policy").join("declared"),
    );
    let files = vec![root.join("vitest.config.ts")];
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    for (case, expected) in [
        ("array-value", "scalar JSON literals"),
        ("object-value", "scalar JSON literals"),
        ("empty-key", "nonempty direct test property names"),
        ("dotted-key", "nonempty direct test property names"),
        ("invalid-kind", "invalid type"),
        ("removed-required", "unknown field"),
        ("invalid-glob", "unclosed character class"),
    ] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))
                .unwrap();
        let error = diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
            .err()
            .unwrap_or_else(|| panic!("{case} must reject invalid options"));
        assert!(format!("{error:#}").contains(expected), "{case}: {error:#}");
    }
}
