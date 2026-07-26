// Included into `napi_api::tests`; shares its fixture helpers and imports.

include!("tests_planning/vitest_config_extends.rs");

#[test]
fn tests_plan_json_returns_complete_deterministic_changed_file_inventory_when_no_tests_match() {
    let root = fixture_root("test-plan-config");
    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["unchanged-order.ts", "deleted.ts", "unchanged-order.ts"],
            "diff": "\
diff --git a/deleted.ts b/deleted.ts
deleted file mode 100644
diff --git a/old-name.ts b/new-name.ts
similarity index 100%
rename from old-name.ts
rename to new-name.ts
diff --git a/copy-source.ts b/copied.ts
similarity index 100%
copy from copy-source.ts
copy to copied.ts
diff --git a/space name.ts b/space name.ts
diff --git a/日本語.ts b/日本語.ts
diff --git a/-leading.ts b/-leading.ts
"
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
    assert_eq!(
        plan["changed_files"],
        json!([
            "-leading.ts",
            "copied.ts",
            "copy-source.ts",
            "deleted.ts",
            "new-name.ts",
            "old-name.ts",
            "space name.ts",
            "unchanged-order.ts",
            "日本語.ts"
        ])
    );
}

#[test]
fn tests_plan_json_rejects_a_malformed_quoted_path_in_an_explicit_diff() {
    let error = tests_plan_json_impl(
        json!({
            "root": fixture_root("test-plan-config"),
            "diff": "diff --git \"a/unsupported\\q.ts\" \"b/unsupported\\q.ts\"\n"
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("quoted git path contains an unsupported escape"));
}

#[test]
fn tests_plan_json_base_head_preserves_tabs_and_newlines_in_changed_file_inventory() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    crate::test_support::git_init(root);
    let paths = ["tab\tname.ts", "line\nbreak.ts"];
    for path in paths {
        std::fs::write(root.join(path), "export const value = 1;\n").unwrap();
    }
    crate::test_support::git_commit_all(root, "base");
    for path in paths {
        std::fs::write(root.join(path), "export const value = 2;\n").unwrap();
    }
    crate::test_support::git_commit_all(root, "head");

    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "base": "HEAD~1",
            "head": "HEAD"
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["changed_files"], json!(["line\nbreak.ts", "tab\tname.ts"]));
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn tests_plan_json_preserves_manual_symlink_identity_while_analyzing_its_target() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    crate::test_support::git_init(root);
    std::fs::write(root.join("real.ts"), "export const value = 1;\n").unwrap();
    std::fs::write(
        root.join("real.test.ts"),
        "import { value } from './real';\ntest('value', () => value);\n",
    )
    .unwrap();
    std::os::unix::fs::symlink("real.ts", root.join("alias.ts")).unwrap();
    crate::test_support::git_commit_all(root, "fixture");

    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["alias.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["changed_files"], json!(["alias.ts"]));
    assert_eq!(plan["selected_tests"][0]["test_file"], "real.test.ts");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["changed_file"],
        "real.ts"
    );
}

#[test]
fn tests_plan_json_union_applies_vitest_setup_fallback() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-setup-dependencies");
    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["config/setup-selector.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["warnings"].as_array().unwrap().iter().any(|warning| {
        warning["type"] == "vitest-setup-dynamic"
    }));
    assert_eq!(
        plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|test| test["test_file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"]
    );
}

#[test]
fn tests_plan_json_tracks_commonjs_dynamic_setup_helper() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-setup-dependencies");
    let output = tests_plan_json_impl(
        json!({
            "framework": "vitest",
            "root": root,
            "changedFiles": ["config/dynamic-commonjs-values.cjs"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(
        plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|test| test["test_file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["commonjs-closure-owner/commonjs-closure.test.ts"]
    );
}

#[test]
fn tests_plan_json_setup_fallback_spends_dependency_group_budget() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-setup-dependencies");
    let output = tests_plan_json_impl(
        json!({
            "framework": "vitest",
            "root": root,
            "config": "dependency-limit.no-mistakes.yml",
            "changedFiles": ["setup/conditional-a.ts", "config/setup-selector.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(plan["groups"].as_array().unwrap().len(), 1, "{plan:#}");
    assert_eq!(plan["groups"][0]["type"], "dependencies");
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["selected"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
}

#[test]
fn tests_plan_why_comment_and_graph_exports_return_reports() {
    let root = fixture_root("test-plan-config");
    let plan_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": ["source.ts"],
        "limitFiles": 1
    })
    .to_string();
    let output = tests_plan_json_impl(plan_options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"][0]["targets"][0]["runner"], "vitest");

    let fallback_limit_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": ["web/app/page.tsx"],
        "limitFiles": 1
    })
    .to_string();
    let fallback_limit_output = tests_plan_json_impl(fallback_limit_options).unwrap();
    let fallback_limit: serde_json::Value = serde_json::from_str(&fallback_limit_output).unwrap();

    assert_eq!(fallback_limit["fallback_triggered"], true);
    assert_eq!(
        fallback_limit["selected_tests"].as_array().unwrap().len(),
        1
    );
    assert_eq!(fallback_limit["groups"][0]["limit"], 1);

    let no_global_fallback_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": [".no-mistakes.yml"],
        "globalConfigFallback": false
    })
    .to_string();
    let no_global_fallback_output = tests_plan_json_impl(no_global_fallback_options).unwrap();
    let no_global_fallback: serde_json::Value =
        serde_json::from_str(&no_global_fallback_output).unwrap();

    assert_eq!(no_global_fallback["fallback_triggered"], false);
    assert_eq!(
        no_global_fallback["selected_tests"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let legacy_plan_options = json!({
        "root": root,
        "changedFiles": ["source.ts"],
    })
    .to_string();
    let legacy_output = tests_plan_json_impl(legacy_plan_options).unwrap();
    let legacy_plan: serde_json::Value = serde_json::from_str(&legacy_output).unwrap();

    assert_eq!(legacy_plan["fallback_triggered"], false);
    assert!(legacy_plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|test| test["test_file"] == "source.test.mts"));

    let comment = tests_comment_markdown_impl(json!({ "planJson": plan }).to_string()).unwrap();
    assert!(comment.contains("Selected Tests"));

    let plan_path = PathBuf::from(&root).join("plan.json");
    let path_comment =
        tests_comment_markdown_impl(json!({ "plan": plan_path.display().to_string() }).to_string())
            .unwrap();
    assert!(path_comment.contains("source.test.mts"));

    let graph = tests_graph_json_impl(json!({ "planJson": output }).to_string()).unwrap();
    let graph: serde_json::Value = serde_json::from_str(&graph).unwrap();
    assert!(!graph["nodes"].as_array().unwrap().is_empty());

    let mermaid = tests_graph_mermaid_impl(
        json!({ "planJson": serde_json::from_str::<serde_json::Value>(&output).unwrap() })
            .to_string(),
    )
    .unwrap();
    assert!(mermaid.starts_with("graph TD"));

    let why_options = json!({
        "root": fixture_root("test-plan-config"),
        "test": "source.test.mts",
        "changed": "source.ts"
    })
    .to_string();
    let why = tests_why_json_impl(why_options).unwrap();
    let why: serde_json::Value = serde_json::from_str(&why).unwrap();
    assert!(!why["source.ts"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_json_exposes_target_scoped_configured_triggers() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let output = tests_plan_json_impl(
        serde_json::json!({
            "framework": "vitest",
            "root": root,
            "changedFiles": ["migrations/001.sql"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"][0]["test_file"], "src/db/db.test.ts");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via"],
        serde_json::json!(["configured-trigger"])
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["project"],
        "database"
    );
}
