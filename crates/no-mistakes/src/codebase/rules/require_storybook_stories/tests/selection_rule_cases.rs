use super::*;

#[test]
fn colocated_tests_do_not_cover_components_without_option() {
    let root = fixture("colocated-tests");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        findings
            .iter()
            .filter_map(|finding| finding.target.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "components/MockTs.tsx#MockTs",
            "components/MockTsx.tsx#MockTsx",
            "components/NestedOnly.tsx#NestedOnly",
            "components/PlainTs.tsx#PlainTs",
            "components/PlainTsx.tsx#PlainTsx",
            "components/SpecOnly.tsx#SpecOnly",
        ]
    );
}

#[test]
fn explicit_include_globs_match_direct_children_only() {
    let root = fixture("direct-child-scope");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include: ["components/ui/*.tsx"]
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn ignore_index_and_private_files_skips_selected_source_files() {
    let root = fixture("private-index-skip");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include: ["components/ui/*.tsx"]
ignore_index_and_private_files: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn test_files_are_excluded_from_component_selection() {
    let root = fixture("test-files-excluded");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include: ["components/**/*.tsx"]
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn type_only_story_imports_do_not_count_as_coverage() {
    let root = fixture("type-only-story");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include: ["components/**/*.tsx"]
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "components/Card.tsx");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Card.tsx#Card")
    );
}

#[test]
fn same_file_siblings_are_not_implicitly_covered() {
    let root = fixture("same-file-sibling");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Card.tsx#Sibling")
    );
}

#[test]
fn helper_imports_do_not_count_as_direct_story_coverage() {
    let root = fixture("helper-import");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Hidden.tsx#Hidden")
    );
}

#[test]
fn story_imports_resolve_component_reexports() {
    let root = fixture("reexport");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn transitive_coverage_uses_project_relative_keys() {
    let root = fixture("project-root");
    let findings = check(
        &root,
        &config_with_project_root(
            "web",
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn component_and_file_opt_outs_use_no_mistakes_comments() {
    let root = fixture("comments");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn config_opt_outs_need_reasons_and_existing_targets() {
    let root = fixture("missing");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
allow_components:
  "components/Missing.tsx#Missing": ""
  "components/Gone.tsx#Gone": "no longer exists"
allow_files:
  "components/Card.tsx": "covered by story"
  "components/nope/**": "gone"
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("must include a reason")));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "components/nope/**"
            && finding.message.contains("does not match")));
}

#[test]
fn default_export_assignments_are_selected() {
    let root = fixture("default-export");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_default_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn dynamic_import_targets_are_not_required_by_include_all() {
    let root = fixture("dynamic");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}
