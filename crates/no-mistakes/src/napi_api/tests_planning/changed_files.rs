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
