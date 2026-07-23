use super::*;

fn fixture(name: &str) -> (tempfile::TempDir, PathBuf) {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan")
            .join(name),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

#[test]
fn project_scoped_tsconfig_resolves_setup_alias_after_catalog_finalization() {
    let (_fixture, root) = fixture("vitest-project-tsconfig-setup");
    let config_path = root.join(".no-mistakes.yml");
    crate::ast::begin_parse_count(&root);
    let mut shared =
        crate::codebase::dependencies::SharedTraversalContext::prepare_with_framework_plan(
            root.clone(),
            None,
            Some(&config_path),
            crate::codebase::dependencies::graph::GraphBuildPlan::default(),
            crate::codebase::test_discovery::FrameworkPreparationPlan::for_runners([
                crate::codebase::test_discovery::TestRunner::Vitest,
            ]),
        )
        .unwrap();
    let parses = crate::ast::finish_parse_count(&root);
    assert_eq!(
        parses.get(&root.join("vitest.workspace.ts")),
        Some(&2),
        "the unresolved imported alias is reparsed once with the final catalog: {parses:#?}"
    );
    assert_eq!(
        parses.get(&root.join("packages/unit/vitest.config.ts")),
        Some(&2),
        "the standalone project config is reparsed once with the final catalog: {parses:#?}"
    );
    assert_eq!(
        shared.source_store().physical_read_count(),
        10,
        "the final reparse must reuse already-loaded runner configs while reading each fixture source once"
    );
    assert_eq!(
        shared
            .tsconfig_catalog()
            .provenance_for(&root.join("packages/unit/vitest.config.ts"))
            .config
            .as_deref(),
        Some(root.join("packages/unit/tsconfig.json").as_path())
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new(
        shared.tsconfig_catalog(),
        shared.graph_files().visible(),
    );
    assert_eq!(
        crate::codebase::ts_resolver::ImportResolution::resolve(
            &resolver,
            "@setup/list",
            &root.join("packages/unit/vitest.config.ts"),
        ),
        Some(root.join("packages/unit/setup/list.ts"))
    );
    let graph = shared.canonical_graph().unwrap();
    let owner = crate::codebase::dependencies::graph::NodeId::File(
        root.join("packages/unit/tests/owner.test.ts"),
    );
    let setup = crate::codebase::dependencies::graph::EdgeKind::VitestSetup(
        crate::codebase::dependencies::graph::VitestSetupField::SetupFiles,
    );
    assert_eq!(
        graph.dependencies_of_node(&owner),
        Some(&vec![
            (
                crate::codebase::dependencies::graph::NodeId::File(
                    root.join("packages/unit/setup/aliased-default.ts"),
                ),
                setup.clone(),
            ),
            (
                crate::codebase::dependencies::graph::NodeId::File(
                    root.join("packages/unit/setup/aliased-namespace.ts"),
                ),
                setup,
            ),
            (
                crate::codebase::dependencies::graph::NodeId::File(
                    root.join("packages/unit/setup/aliased.ts"),
                ),
                crate::codebase::dependencies::graph::EdgeKind::VitestSetup(
                    crate::codebase::dependencies::graph::VitestSetupField::SetupFiles,
                ),
            ),
        ])
    );
    for setup in ["aliased-default.ts", "aliased-namespace.ts", "aliased.ts"] {
        let mut args = vitest_setup_args(
            root.clone(),
            vec![root.join("packages/unit/setup").join(setup)],
        );
        args.config = Some(config_path.clone());
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["packages/unit/tests/owner.test.ts"],
            "{setup}: {plan:#?}"
        );
        assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
    }
}

#[test]
fn dynamic_setup_expression_does_not_trigger_final_catalog_reparse() {
    let (_fixture, root) = fixture("vitest-dynamic-setup-no-reparse");
    let config_path = root.join(".no-mistakes.yml");
    crate::ast::begin_parse_count(&root);
    let shared =
        crate::codebase::dependencies::SharedTraversalContext::prepare_with_framework_plan(
            root.clone(),
            None,
            Some(&config_path),
            crate::codebase::dependencies::graph::GraphBuildPlan::default(),
            crate::codebase::test_discovery::FrameworkPreparationPlan::for_runners([
                crate::codebase::test_discovery::TestRunner::Vitest,
            ]),
        )
        .unwrap();
    let parses = crate::ast::finish_parse_count(&root);

    assert_eq!(parses.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(
        shared.source_store().physical_read_count(),
        2,
        "the dynamic setup config is read once; no final-catalog reparse occurs"
    );
}

#[test]
fn deleted_runtime_setup_candidate_falls_back_when_only_declaration_remains() {
    let (_fixture, root) = fixture("vitest-declaration-runtime-deleted");
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.config = Some(root.join(".no-mistakes.yml"));
    args.diff_content = Some(
        "diff --git a/setup/runtime.ts b/setup/runtime.ts\n\
--- a/setup/runtime.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const runtimeSetup = true\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["tests/owner.test.ts"]
    );
}
