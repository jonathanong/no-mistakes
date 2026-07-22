use super::*;

fn assert_framework_fallbacks(root: &std::path::Path, expected: bool) {
    for framework in ["vitest", "playwright"] {
        assert_eq!(
            json(&plan(
                root,
                framework,
                &["--base", "HEAD~1", "--head", "HEAD", "--json"]
            ))["fallback_triggered"],
            expected,
            "{framework}"
        );
    }
}

#[test]
fn revision_config_changes_invalidate_only_the_affected_framework() {
    for (variant, vitest_fallback, playwright_fallback) in [
        ("format-only.yml", false, false),
        ("vitest-change.yml", true, false),
        ("playwright-change.yml", false, true),
        ("project-change.yml", true, false),
        ("graph-route-change.yml", true, true),
    ] {
        let fixture = fixture();
        let root = fixture.path();
        git_init(root);
        copy_config(root, variant);
        git_commit(root, variant);

        let vitest = json(&plan(
            root,
            "vitest",
            &["--base", "HEAD~1", "--head", "HEAD", "--json"],
        ));
        let playwright = json(&plan(
            root,
            "playwright",
            &["--base", "HEAD~1", "--head", "HEAD", "--json"],
        ));
        assert_eq!(vitest["fallback_triggered"], vitest_fallback, "{variant}");
        assert_eq!(
            playwright["fallback_triggered"], playwright_fallback,
            "{variant}"
        );
    }
}

#[test]
fn routed_project_root_changes_invalidate_the_shared_graph() {
    let fixture = fixture();
    let root = fixture.path();
    copy_config(root, "routed-root-base.yml");
    git_init(root);
    copy_config(root, "routed-root-change.yml");
    git_commit(root, "route root change");
    assert_framework_fallbacks(root, true);
}

#[test]
fn graph_rule_options_invalidate_but_unrelated_rules_do_not() {
    let unrelated_fixture = fixture();
    let unrelated_root = unrelated_fixture.path();
    copy_config(unrelated_root, "unrelated-rule-base.yml");
    git_init(unrelated_root);
    copy_config(unrelated_root, "unrelated-rule-change.yml");
    git_commit(unrelated_root, "unrelated lint rule");
    assert_framework_fallbacks(unrelated_root, false);

    let graph_fixture = fixture();
    let graph_root = graph_fixture.path();
    copy_config(graph_root, "http-rule-base.yml");
    git_init(graph_root);
    copy_config(graph_root, "http-rule-change.yml");
    git_commit(graph_root, "http graph option");
    assert_framework_fallbacks(graph_root, true);

    let filter_fixture = fixture();
    let filter_root = filter_fixture.path();
    copy_config(filter_root, "unrelated-rule-base.yml");
    git_init(filter_root);
    copy_config(filter_root, "dynamic-import-filter-rule-change.yml");
    git_commit(filter_root, "test filter rule");
    assert_framework_fallbacks(filter_root, true);
}

#[test]
fn dynamic_import_rule_targeted_project_filters_invalidate() {
    let fixture = fixture();
    let root = fixture.path();
    copy_config(root, "dynamic-project-filter-base.yml");
    git_init(root);
    copy_config(root, "dynamic-project-filter-change.yml");
    git_commit(root, "dynamic import project filter");
    assert_framework_fallbacks(root, true);
}

#[test]
fn dynamic_import_rule_targeted_runner_config_invalidates_cross_framework() {
    let cross_fixture = fixture();
    let cross_root = cross_fixture.path();
    copy_config(cross_root, "dynamic-vitest-policy-base.yml");
    git_init(cross_root);
    copy_config(cross_root, "dynamic-vitest-policy-change.yml");
    git_commit(cross_root, "dynamic rule vitest policy");
    assert_framework_fallbacks(cross_root, true);

    let scoped_fixture = fixture();
    let scoped_root = scoped_fixture.path();
    copy_config(scoped_root, "dynamic-vitest-policy-base.yml");
    git_init(scoped_root);
    copy_config(scoped_root, "dynamic-vitest-policy-playwright-change.yml");
    git_commit(scoped_root, "unreferenced playwright change");
    let vitest = json(&plan(
        scoped_root,
        "vitest",
        &["--base", "HEAD~1", "--head", "HEAD", "--json"],
    ));
    let playwright = json(&plan(
        scoped_root,
        "playwright",
        &["--base", "HEAD~1", "--head", "HEAD", "--json"],
    ));
    assert_eq!(vitest["fallback_triggered"], false);
    assert_eq!(playwright["fallback_triggered"], true);
}

#[test]
fn mixed_git_and_inline_config_changes_invalidate_both_frameworks() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "playwright-change.yml");
    git_commit(root, "playwright config change");
    copy_config(root, "vitest-and-playwright-change.yml");

    let args = [
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
        "--diff-command",
        "git diff -- .no-mistakes.yml",
        "--json",
    ];
    assert_eq!(
        json(&plan(root, "vitest", &args))["fallback_triggered"],
        true
    );
    assert_eq!(
        json(&plan(root, "playwright", &args))["fallback_triggered"],
        true
    );
}

#[test]
fn inline_config_diff_wins_over_equal_base_and_head_revisions() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "vitest-change.yml");
    let args = [
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--diff-command",
        "git diff -- .no-mistakes.yml",
        "--json",
    ];
    assert_eq!(
        json(&plan(root, "vitest", &args))["fallback_triggered"],
        true
    );
    assert_eq!(
        json(&plan(root, "playwright", &args))["fallback_triggered"],
        false
    );
}

#[test]
fn inline_diff_compares_complete_framework_configuration() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "vitest-change.yml");
    let command = "git diff -- .no-mistakes.yml";
    assert_eq!(
        json(&plan(
            root,
            "vitest",
            &["--diff-command", command, "--json"]
        ))["fallback_triggered"],
        true
    );
    assert_eq!(
        json(&plan(
            root,
            "playwright",
            &["--diff-command", command, "--json"]
        ))["fallback_triggered"],
        false
    );
}

#[test]
fn inline_diff_handles_added_deleted_and_modified_renamed_configs() {
    let added_fixture = fixture();
    let added_root = added_fixture.path();
    std::fs::remove_file(added_root.join(".no-mistakes.yml")).unwrap();
    git_init(added_root);
    copy_config(added_root, "format-only.yml");
    git(added_root, &["add", ".no-mistakes.yml"]);
    assert_eq!(
        json(&plan(
            added_root,
            "vitest",
            &[
                "--diff-command",
                "git diff --cached -- .no-mistakes.yml",
                "--json"
            ]
        ))["fallback_triggered"],
        true
    );

    let deleted_fixture = fixture();
    let deleted_root = deleted_fixture.path();
    git_init(deleted_root);
    std::fs::remove_file(deleted_root.join(".no-mistakes.yml")).unwrap();
    assert_eq!(
        json(&plan(
            deleted_root,
            "vitest",
            &[
                "--diff-command",
                "git diff -- .no-mistakes.yml",
                "--global-config-fallback",
                "true",
                "--json"
            ]
        ))["fallback_triggered"],
        true
    );

    let renamed_fixture = fixture();
    let renamed_root = renamed_fixture.path();
    git_init(renamed_root);
    std::fs::remove_file(renamed_root.join(".no-mistakes.yml")).unwrap();
    std::fs::copy(
        renamed_root.join("configs/vitest-change.yml"),
        renamed_root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    git(renamed_root, &["add", "-A"]);
    let command = "git diff --cached --find-renames -- .no-mistakes.yml .no-mistakes.yaml";
    assert_eq!(
        json(&plan(
            renamed_root,
            "vitest",
            &["--diff-command", command, "--json"]
        ))["fallback_triggered"],
        true
    );
    assert_eq!(
        json(&plan(
            renamed_root,
            "playwright",
            &["--diff-command", command, "--json"]
        ))["fallback_triggered"],
        false
    );
}

#[test]
fn changed_file_only_and_unparseable_history_keep_conservative_fallback() {
    let fixture = fixture();
    let root = fixture.path();
    assert_eq!(
        json(&plan(
            root,
            "vitest",
            &["--changed-file", ".no-mistakes.yml", "--json"]
        ))["fallback_triggered"],
        true
    );
    copy_config(root, "malformed.yml");
    git_init(root);
    copy_config(root, "format-only.yml");
    git_commit(root, "repair config");
    assert_eq!(
        json(&plan(
            root,
            "vitest",
            &["--base", "HEAD~1", "--head", "HEAD", "--json"]
        ))["fallback_triggered"],
        true
    );
}

#[test]
fn renamed_config_with_equal_values_does_not_invalidate_frameworks() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    std::fs::rename(
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    git_commit(root, "rename config extension");
    for framework in ["vitest", "playwright"] {
        assert_eq!(
            json(&plan(
                root,
                framework,
                &["--base", "HEAD~1", "--head", "HEAD", "--json"]
            ))["fallback_triggered"],
            false,
            "{framework}"
        );
    }
}
