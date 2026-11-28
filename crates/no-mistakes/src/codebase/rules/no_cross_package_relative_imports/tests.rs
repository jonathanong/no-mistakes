use super::RULE_ID;

#[test]
fn workspace_relative_imports_reuse_the_rust_check_graph_and_existing_facts() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    use crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan;
    use crate::codebase::rules::{run_check, run_check_with_facts, try_canonical_graph_plan};
    use crate::codebase::ts_source::discover_files;
    use crate::config::v2::load_v2_config;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let config = load_v2_config(&root, None).unwrap();
    let plan = try_canonical_graph_plan(&config).unwrap().unwrap();
    let (graph, graph_context) = ts_fact_plan_and_context_for_plan(&root, plan);
    let facts = collect_check_facts(
        &root,
        discover_files(&root, &[]),
        CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let standalone = run_check(&root, None, None).unwrap();
    assert_eq!(standalone.len(), 7);
    assert_eq!(
        run_check_with_facts(&root, None, None, &facts).unwrap(),
        standalone
    );
}

#[test]
fn workspace_relative_imports_reports_malformed_selected_source_to_rust_callers() {
    use crate::codebase::rules::run_check;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let config = root.join("incomplete.yml");
    let error = run_check(&root, Some(&config), None).unwrap_err();
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .unwrap();
    assert!(failure
        .file
        .as_deref()
        .is_some_and(|file| file.ends_with("packages/app/src/gaps/broken.mts")));
}

#[test]
fn vitest_import_calls_distinguish_vitest_bindings_and_computed_args() {
    use crate::codebase::dependencies::extract::{
        FunctionCall, ImportedBinding, ImportedBindingKind, InvocationKind,
    };
    use crate::codebase::dependencies::graph::{vitest_import_calls, VitestReceiver};
    use crate::codebase::ts_source::facts::TsFileFacts;

    let call = |callee: &str, line: u32, arg: Option<&str>, scope: Option<usize>| FunctionCall {
        caller: None,
        caller_id: None,
        syntactic_caller: None,
        callee: callee.to_owned(),
        line,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: crate::codebase::dependencies::extract::CallTargetIdentity::Unknown,
        callee_binding_scope: scope,
        static_arg: arg.map(str::to_owned),
        static_cwd: None,
    };
    let facts = TsFileFacts {
        imported_bindings: vec![
            ImportedBinding {
                specifier: "vitest".to_owned(),
                local: "mock".to_owned(),
                imported: "vi".to_owned(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
            ImportedBinding {
                specifier: "vitest".to_owned(),
                local: "vitest".to_owned(),
                imported: "*".to_owned(),
                kind: ImportedBindingKind::Namespace,
                is_type_only: false,
            },
            ImportedBinding {
                specifier: "vitest".to_owned(),
                local: "vitestAlias".to_owned(),
                imported: "vitest".to_owned(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
            ImportedBinding {
                specifier: "./helpers".to_owned(),
                local: "other".to_owned(),
                imported: "vi".to_owned(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            },
        ],
        function_calls: vec![
            call("vi.importActual", 1, Some("./a.ts"), None),
            call("mock.importMock", 2, Some("../mod"), None),
            call("vitest.vi.importActual", 3, None, None),
            call("vitestAlias.importActual", 10, Some("./b.ts"), None),
            call("vitest.vitest.importMock", 11, Some("./c.ts"), None),
            call("vi.importActual", 4, Some("./shadowed.ts"), Some(1)),
            call("other.importActual", 5, Some("./helper.ts"), None),
            call("<unknown>.importMock", 8, Some("./factory.ts"), None),
            call("vitest.other.importActual", 9, Some("./ignored.ts"), None),
            call("vi.spyOn", 6, Some("./ignored.ts"), None),
            call("plainCall", 7, Some("./ignored.ts"), None),
        ],
        ..Default::default()
    };
    let found: Vec<_> = vitest_import_calls(&facts)
        .map(|item| (item.line, item.specifier, item.computed, item.receiver))
        .collect();
    assert_eq!(
        found,
        vec![
            (1, "./a.ts", false, VitestReceiver::Direct),
            (2, "../mod", false, VitestReceiver::Direct),
            (3, "<computed>", true, VitestReceiver::Direct),
            (10, "./b.ts", false, VitestReceiver::Direct),
            (11, "./c.ts", false, VitestReceiver::Direct),
            (
                4,
                "./shadowed.ts",
                false,
                VitestReceiver::Unproven {
                    imported_from: None
                }
            ),
            (
                5,
                "./helper.ts",
                false,
                VitestReceiver::Unproven {
                    imported_from: Some("./helpers")
                }
            ),
            (
                8,
                "./factory.ts",
                false,
                VitestReceiver::Unproven {
                    imported_from: None
                }
            ),
        ]
    );
}

#[test]
fn vitest_helper_imports_follow_the_unanalyzable_policy() {
    use crate::codebase::rules::run_check;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let rows = |name: &str| {
        let mut findings = run_check(&root, Some(&root.join(name)), None).unwrap();
        findings.sort_by_key(|finding| (finding.line, finding.import.clone()));
        findings
            .into_iter()
            .map(|finding| {
                (
                    finding.line,
                    finding.import.unwrap_or_default(),
                    finding.message,
                )
            })
            .collect::<Vec<_>>()
    };

    let crossing = "../../shared/src/status.mts";
    let receiver = "Unanalyzable import: the receiver of importActual or importMock is unresolved. Import vi directly from Vitest.";
    assert_eq!(
        rows("helper-vitest-report.yml"),
        vec![
            (5, crossing.to_owned(), receiver.to_owned()),
            (6, crossing.to_owned(), receiver.to_owned()),
            (8, crossing.to_owned(), receiver.to_owned()),
            (9, "<computed>".to_owned(), receiver.to_owned()),
        ]
    );
    assert!(rows("helper-vitest-ignore.yml").is_empty());
    assert!(rows("helper-vitest-default.yml").is_empty());

    let direct = rows("helper-vitest-direct.yml");
    let fix = "Use a declared package import through the target package's public exports instead of a relative path across the workspace package boundary.";
    assert_eq!(
        direct,
        vec![
            (2, crossing.to_owned(), fix.to_owned()),
            (
                7,
                "<computed>".to_owned(),
                "Unanalyzable import: the specifier is computed. Use a literal import specifier."
                    .to_owned(),
            ),
            (
                9,
                "../../shared/src/missing-topic.mts".to_owned(),
                "Unanalyzable import: the relative specifier has no unique file target. Run resolve-check and fix resolution.".to_owned(),
            ),
            (10, crossing.to_owned(), fix.to_owned()),
            (11, crossing.to_owned(), fix.to_owned()),
        ]
    );
}

#[test]
fn direct_vitest_call_checks_prepared_crossing_before_lexical_same_package_return() {
    use super::vitest::{record_direct_vitest_target, VitestSite};
    use crate::codebase::dependencies::graph::ImportTarget;
    use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
    use crate::codebase::workspaces::{IndexedWorkspaceMap, WorkspacePackage};
    use crate::config::v2::Unanalyzable;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let workspace = IndexedWorkspaceMap::from_packages(vec![
        WorkspacePackage {
            name: "@sample/app".to_owned(),
            dir: root.join("packages/app"),
            entry: None,
            exports: None,
            imports: None,
        },
        WorkspacePackage {
            name: "@sample/shared".to_owned(),
            dir: root.join("packages/shared"),
            entry: None,
            exports: None,
            imports: None,
        },
    ]);
    let file = root.join("packages/app/compat/direct-vi.test.ts");
    let owner = workspace.nearest_package(&file).unwrap();
    let site = VitestSite {
        file: &file,
        relative_file: "packages/app/compat/direct-vi.test.ts",
        owner,
        message: "crossing",
        policy: Unanalyzable::Report,
    };
    let mut diagnostics = AnalysisDiagnostics::default();
    let shared_target = ImportTarget::File(root.join("packages/shared/src/status.mts"));
    record_direct_vitest_target(
        &root,
        &workspace,
        &site,
        14,
        "./shared-link/status.mts",
        Some(&shared_target),
        &mut diagnostics,
    );
    assert_eq!(diagnostics.findings.len(), 1);
    assert_eq!(
        diagnostics.findings[0].import.as_deref(),
        Some("./shared-link/status.mts")
    );
    assert_eq!(
        diagnostics.findings[0].target.as_deref(),
        Some("packages/shared/src/status.mts")
    );
    assert!(diagnostics.advisories.is_empty());

    let local_target = ImportTarget::File(root.join("packages/app/compat/vi-helper.ts"));
    let mut local_diagnostics = AnalysisDiagnostics::default();
    record_direct_vitest_target(
        &root,
        &workspace,
        &site,
        15,
        "./vi-helper",
        Some(&local_target),
        &mut local_diagnostics,
    );
    assert!(local_diagnostics.findings.is_empty());
    assert!(local_diagnostics.advisories.is_empty());
}
