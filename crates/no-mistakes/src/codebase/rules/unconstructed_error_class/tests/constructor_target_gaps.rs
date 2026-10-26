use super::*;

fn saved_root() -> PathBuf {
    fixture()
        .parent()
        .unwrap()
        .join("constructor-target-gaps/fixture")
}

#[test]
fn package_runtime_files_preserve_constructor_and_unused_error_child_proof() {
    let fixture = crate::test_support::materialize_saved_fixture(&saved_root());
    let root = fixture.path().to_path_buf();
    for policy in [".no-mistakes.yml", "advise.yml", "ignore.yml"] {
        for effort in [Some("low"), None, Some("max")] {
            let mut options = serde_json::json!({"root": root, "config": root.join(policy)});
            if let Some(effort) = effort {
                options["effort"] = effort.into();
            }
            let output = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                options.to_string(),
            ))
            .unwrap();
            let report: serde_json::Value = serde_json::from_str(&output).unwrap();
            // Each constructed Error facade is used; only its unused child is
            // dead. The inverse root subpath package must never win ownership.
            assert_eq!(
                reported_channel(&report, "rules"),
                [
                    "app/children.ts:3 UnusedConfigError",
                    "app/children.ts:5 UnusedFilterError",
                    "app/children.ts:7 UnusedPoolError",
                    "app/conditional-import.mts:2 UnusedImportError",
                    "app/conditional-require.cts:3 UnusedRequireError",
                ],
                "{policy}, {effort:?}: {report}"
            );
            let expected = if policy == "ignore.yml" {
                vec![]
            } else {
                // The annotation promises an Error constructor, but its opaque
                // initializer cannot prove the runtime constructor identity.
                vec![
                    "app/children.ts:14 unanalyzable",
                    "app/conditional-unknown.ts:2 unanalyzable",
                    "app/conditional-unknown.ts:3 unanalyzable",
                ]
            };
            assert_eq!(
                reported_channel(&report, "advisories"),
                expected,
                "{policy}, {effort:?}: {report}"
            );
        }
    }
}

#[test]
fn conditional_declarations_share_fact_ownership_without_widening_ordinary_relationships() {
    use crate::codebase::analysis_session::AnalysisSession;
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles, NodeId};
    use crate::codebase::ts_resolver::TsConfig;
    use crate::codebase::ts_source::facts::{
        collect_ts_facts_with_session_and_context, TsFactContext,
    };
    let fixture = crate::test_support::materialize_saved_fixture(&saved_root());
    let root = fixture.path().to_path_buf();
    let session = AnalysisSession::new(Some(crate::diagnostics::InvocationObserver::new(true)));
    let visible = session.visible_paths(&root);
    let files = GraphFiles::from_files(visible.paths_for(&root).as_ref().clone());
    let plan = GraphBuildPlan {
        imports: true,
        calls: true,
        extends: true,
        ..Default::default()
    };
    let facts = collect_ts_facts_with_session_and_context(
        &session,
        files.indexable(),
        plan.ts_fact_plan(),
        &TsFactContext {
            root: root.clone(),
            ..Default::default()
        },
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        paths: vec![(
            "@facade".into(),
            vec!["packages/facade/src/index.ts".into()],
        )],
        ..Default::default()
    };
    let build = |extends| {
        DepGraph::build_with_plan_files_config_facts_and_session(
            &root,
            &config,
            GraphBuildPlan { extends, ..plan },
            &files,
            None,
            Some(&facts),
            session.clone(),
        )
        .unwrap()
    };
    let ordinary = build(false);
    let ancestry = build(true);
    let repeated = build(true);
    assert_eq!(
        ordinary.resolved_call_sites(),
        ancestry.resolved_call_sites()
    );
    assert_eq!(ancestry.class_declarations(), repeated.class_declarations());
    for path in files.indexable() {
        assert_eq!(
            ordinary.dependencies_of_node(&NodeId::file(path)),
            ancestry.dependencies_of_node(&NodeId::file(path)),
            "{}",
            path.display()
        );
    }
    let work = session.work_snapshot();
    for suffix in [
        "node_modules/@types/pg/index.d.mts",
        "node_modules/@types/pg/index.d.ts",
    ] {
        let path = root.join(suffix);
        assert_eq!(work.source_reads.get(&path), Some(&1), "{suffix}");
        assert_eq!(work.parse_attempts.get(&path), Some(&1), "{suffix}");
    }
    assert!(!work
        .parse_attempts
        .contains_key(&root.join("node_modules/pg/index.js")));
}

#[test]
fn require_syntax_in_an_esm_file_does_not_claim_the_import_branch_constructor() {
    let fixture = crate::test_support::materialize_saved_fixture(&saved_root());
    let root = fixture.path();
    let output = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root": root, "config": root.join("require-only.yml")}).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(reported_channel(&report, "rules").is_empty(), "{report}");
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "controls/require-only.mts:3 unanalyzable",
            "controls/require-only.mts:5 unanalyzable",
        ],
        "{report}"
    );
}
