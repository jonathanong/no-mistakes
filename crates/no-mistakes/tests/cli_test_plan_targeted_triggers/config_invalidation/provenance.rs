use super::*;

#[test]
fn manual_config_change_does_not_borrow_unrelated_git_endpoints() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "vitest-change.yml");
    let args = [
        "--changed-file",
        ".no-mistakes.yml",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
        "--global-config-fallback",
        "true",
        "--json",
    ];
    for framework in ["vitest", "playwright"] {
        assert_eq!(
            json(&plan(root, framework, &args))["fallback_triggered"],
            true,
            "{framework}"
        );
    }
}

#[test]
fn manual_config_change_overlapping_git_config_change_fails_open() {
    let fixture = fixture();
    let root = fixture.path();
    git_init(root);
    copy_config(root, "vitest-change.yml");
    git_commit(root, "vitest config change");
    copy_config(root, "vitest-and-playwright-change.yml");
    let args = [
        "--changed-file",
        ".no-mistakes.yml",
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
        "--global-config-fallback",
        "true",
        "--json",
    ];
    for framework in ["vitest", "playwright"] {
        assert_eq!(
            json(&plan(root, framework, &args))["fallback_triggered"],
            true,
            "{framework}"
        );
    }
}

#[test]
fn manual_config_requires_a_same_path_structured_diff_endpoint() {
    let fixture = fixture();
    let root = fixture.path();
    std::fs::rename(
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    git_init(root);
    std::fs::copy(
        root.join("configs/vitest-change.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();

    for (manual_path, expected_playwright_fallback) in
        [(".no-mistakes.yaml", false), (".no-mistakes.yml", true)]
    {
        let args = [
            "--changed-file",
            manual_path,
            "--diff-command",
            "git diff -- .no-mistakes.yaml",
            "--global-config-fallback",
            "true",
            "--json",
        ];
        assert_eq!(
            json(&plan(root, "playwright", &args))["fallback_triggered"],
            expected_playwright_fallback,
            "{manual_path}"
        );
    }
}
