use super::*;

#[test]
fn root_only_options_are_not_inherited_but_explicit_child_properties_remain_visible() {
    let root =
        crate::test_support::rule_fixture_root("vitest-project-policy").join("policy-boundaries");
    for (case, expected) in [
        ("root-only-inheritance", 0),
        ("root-only-inheritance-explicit", 3),
    ] {
        let files = vec![root.join(format!("{case}.ts"))];
        let sources = SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        ));
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))
                .unwrap();
        let result =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
        assert!(
            result.incomplete.is_empty(),
            "{case}: {:?}",
            result.incomplete
        );
        assert_eq!(
            result.findings.len(),
            expected,
            "{case}: {:?}",
            result.findings
        );
        if expected > 0 {
            for property in ["name", "projects", "globalSetup"] {
                assert!(
                    result.findings.iter().any(|finding| finding
                        .message
                        .contains(&format!("test.{property} is forbidden"))),
                    "{property}: {:?}",
                    result.findings
                );
            }
        }
    }
}
