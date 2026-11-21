use super::*;

fn diagnose(case: &str) -> Result<Diagnostics> {
    let folder = if case == "exclude-suppressed" {
        "exclude-globs-suppressed"
    } else {
        "exclude-globs"
    };
    let root = crate::test_support::rule_fixture_root("vitest-project-policy").join(folder);
    let mut files = vec![root.join(format!("{case}.ts"))];
    if folder == "exclude-globs" {
        files.push(root.join("settings.ts"));
        files.push(root.join("helper.ts"));
    }
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))?;
    diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
}

fn output(case: &str) -> Diagnostics {
    diagnose(case).unwrap_or_else(|error| panic!("{case} should analyze: {error:#}"))
}

fn quiet(case: &str) {
    let report = output(case);
    assert!(report.findings.is_empty(), "{case}: {:?}", report.findings);
    assert!(
        report.incomplete.is_empty(),
        "{case}: {:?}",
        report.incomplete
    );
}

#[test]
fn configured_globs_match_literal_exclude_arrays_only() {
    for case in [
        "exclude-present",
        "exclude-omitted",
        "exclude-no-test",
        "exclude-explicit-empty",
        "exclude-local-binding",
        "exclude-helper",
        "exclude-proven-spread",
        "exclude-conditional-spread",
        "exclude-conditional-proven",
        "exclude-ternary-complete-or-empty",
        "exclude-ternary-complete-or-complete",
        "exclude-and-complete",
        "exclude-computed-both",
        "exclude-logical-spread",
        "exclude-logical-and-exclude",
        "exclude-project-present",
        "exclude-project-silent",
        "exclude-option-off",
        "exclude-suppressed",
    ] {
        quiet(case);
    }
}

#[test]
fn replaced_exclude_arrays_must_contain_each_configured_glob() {
    let empty = &output("exclude-empty-array").findings;
    assert_eq!(empty.len(), 1);
    assert_eq!(
        empty[0].message,
        "Vitest project <root>: add **/node_modules/**, **/.git/** to test.exclude"
    );
    assert!(!empty[0].message.contains("declare literal test properties"));
    assert_eq!(empty[0].rule, RULE_ID);
    assert!(output("exclude-empty-array").incomplete.is_empty());

    let missing = &output("exclude-missing-glob").findings;
    assert_eq!(missing.len(), 1);
    assert_eq!(
        missing[0].message,
        "Vitest project <root>: add **/.git/** to test.exclude"
    );

    let text = &output("exclude-non-array").findings;
    assert_eq!(text.len(), 1);
    assert_eq!(
        text[0].message,
        "Vitest project <root>: add **/node_modules/**, **/.git/** to test.exclude"
    );

    let project = &output("exclude-project-empty").findings;
    assert_eq!(project.len(), 1);
    assert!(project[0].message.contains("Vitest project orders:"));

    let override_project = &output("exclude-project-override").findings;
    assert_eq!(override_project.len(), 1);
    assert!(override_project[0]
        .message
        .contains("Vitest project orders:"));
    assert!(override_project[0].message.contains("**/node_modules/**"));

    let inherited = &output("exclude-project-inherit").findings;
    assert_eq!(inherited.len(), 2);
    assert!(inherited
        .iter()
        .any(|finding| finding.message.contains("<root>")));
    assert!(inherited
        .iter()
        .any(|finding| finding.message.contains("Vitest project widgets:")));
}

#[test]
fn non_literal_exclude_values_stay_incomplete() {
    for case in [
        "exclude-dynamic",
        "exclude-partial-spread",
        "exclude-dynamic-config",
        "exclude-conditional-unknown",
        "exclude-logical-or-exclude",
        "exclude-logical-nullish-exclude",
        "exclude-identifier-spread",
        "exclude-call-spread",
    ] {
        let report = output(case);
        assert!(report.findings.is_empty(), "{case}: {:?}", report.findings);
        assert!(
            report
                .incomplete
                .iter()
                .any(|failure| failure.property.as_deref() == Some("test.exclude")),
            "{case}: {:?}",
            report.incomplete
        );
    }
    let projects = output("exclude-dynamic-projects");
    assert!(projects.findings.is_empty(), "{:?}", projects.findings);
    assert!(projects
        .incomplete
        .iter()
        .any(|failure| failure.property.as_deref() == Some("test.projects")));
    let entry = output("exclude-string-project");
    assert!(entry.findings.is_empty(), "{:?}", entry.findings);
    assert!(entry
        .incomplete
        .iter()
        .any(|failure| failure.property.as_deref() == Some("test")));
}

#[test]
fn conditional_required_properties_stay_incomplete_when_a_branch_omits_them() {
    let report = output("exclude-conditional-required");
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(report
        .incomplete
        .iter()
        .any(|failure| failure.property.as_deref() == Some("isolate")));
    assert!(report
        .incomplete
        .iter()
        .all(|failure| failure.property.as_deref() != Some("test.exclude")));
}

#[test]
fn a_forbidden_key_on_only_some_branches_stays_incomplete() {
    let report = output("exclude-conditional-forbidden");
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(report.incomplete.iter().any(|failure| {
        failure.property.as_deref() == Some("poolOptions")
            && failure.message.contains("not statically known")
    }));
    assert!(report
        .incomplete
        .iter()
        .all(|failure| failure.property.as_deref() != Some("test.exclude")));
}

#[test]
fn a_forbidden_key_on_every_branch_stays_a_finding() {
    let report = output("exclude-conditional-forbidden-known");
    assert!(report.incomplete.is_empty(), "{:?}", report.incomplete);
    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0]
        .message
        .contains("test.poolOptions is forbidden"));
}

#[test]
fn folded_exclude_paths_name_each_missing_glob() {
    for case in [
        "exclude-conditional-exclude",
        "exclude-ternary-partial-or-empty",
        "exclude-ternary-complete-or-partial",
        "exclude-and-partial",
        "exclude-ternary-partial-or-call",
        "exclude-earlier-then-partial",
        "exclude-computed-then-partial",
    ] {
        let report = output(case);
        assert!(
            report.incomplete.is_empty(),
            "{case}: {:?}",
            report.incomplete
        );
        assert_eq!(report.findings.len(), 1, "{case}: {:?}", report.findings);
        assert!(
            report.findings[0]
                .message
                .contains("add **/.git/** to test.exclude"),
            "{case}: {}",
            report.findings[0].message
        );
        assert!(
            !report.findings[0].message.contains("**/node_modules/**"),
            "{case}"
        );
    }
}

#[test]
fn non_literal_spreads_leave_only_exclude_incomplete() {
    for case in [
        "exclude-call-make-extra",
        "exclude-ternary-complete-or-call",
        "exclude-computed-or-empty",
    ] {
        let report = output(case);
        assert!(report.findings.is_empty(), "{case}: {:?}", report.findings);
        assert_eq!(
            report.incomplete.len(),
            1,
            "{case}: {:?}",
            report.incomplete
        );
        assert_eq!(
            report.incomplete[0].property.as_deref(),
            Some("test.exclude"),
            "{case}"
        );
    }
}

#[test]
fn nested_omitted_exclude_uses_the_enclosing_array() {
    let report = output("exclude-nested-omitted");
    assert!(report.incomplete.is_empty(), "{:?}", report.incomplete);
    assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
    assert!(
        report.findings[0]
            .message
            .contains("add **/node_modules/**, **/.git/** to test.exclude"),
        "{}",
        report.findings[0].message
    );
}

#[test]
fn truthy_object_and_keeps_a_definite_exclude_array() {
    let report = output("exclude-logical-and-known");
    assert!(report.incomplete.is_empty(), "{:?}", report.incomplete);
    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0].message.contains("**/.git/**"));
    assert!(!report.findings[0].message.contains("**/node_modules/**"));
}

#[test]
fn empty_required_exclude_globs_are_rejected() {
    let Err(error) = diagnose("exclude-empty-glob") else {
        panic!("empty glob must be rejected");
    };
    assert!(format!("{error:#}").contains("requiredExcludeGlobs"));
}
