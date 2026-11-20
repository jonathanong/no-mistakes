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
    assert!(empty[0].message.contains("**/node_modules/**"));
    assert!(empty[0].message.contains("**/.git/**"));
    assert_eq!(empty[0].rule, RULE_ID);
    assert!(output("exclude-empty-array").incomplete.is_empty());

    let missing = &output("exclude-missing-glob").findings;
    assert_eq!(missing.len(), 1);
    assert!(missing[0].message.contains("**/.git/**"));
    assert!(!missing[0].message.contains("**/node_modules/**"));

    let text = &output("exclude-non-array").findings;
    assert_eq!(text.len(), 1);
    assert!(text[0].message.contains("test.exclude must include"));

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
fn empty_required_exclude_globs_are_rejected() {
    let Err(error) = diagnose("exclude-empty-glob") else {
        panic!("empty glob must be rejected");
    };
    assert!(format!("{error:#}").contains("requiredExcludeGlobs"));
}
