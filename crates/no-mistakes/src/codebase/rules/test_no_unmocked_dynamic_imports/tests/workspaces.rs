use super::*;

#[test]
fn implicit_workspace_and_project_catalogs_supply_effective_setup_contexts() {
    for kind in ["workspace", "projects"] {
        let root = crate::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-workspaces")
                .join(kind),
        );
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            crate::codebase::ts_source::discover_files(&root, &[]),
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                dynamic_imports: true,
                source: true,
                ..Default::default()
            },
        );
        for findings in [
            check(&root, &config, None).unwrap(),
            check_with_facts(&root, &config, None, &facts).unwrap(),
            crate::codebase::rules::run_check_with_facts(&root, None, None, &facts).unwrap(),
        ] {
            assert_eq!(
                findings
                    .iter()
                    .map(|finding| finding.file.as_str())
                    .collect::<Vec<_>>(),
                vec!["custom/other.case.mts"],
                "{kind}: {findings:?}"
            );
        }
    }
}
