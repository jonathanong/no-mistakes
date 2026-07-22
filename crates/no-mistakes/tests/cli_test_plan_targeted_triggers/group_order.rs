use super::*;

fn assert_shared_has_all_owners_and_reasons(report: &serde_json::Value) {
    let shared = report["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["test_file"] == "src/shared.test.ts")
        .unwrap();
    assert_eq!(
        shared["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|target| target["project"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["database", "web"]
    );
    assert_eq!(
        shared["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|reason| reason["via"].as_array().unwrap())
            .map(|via| via.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["configured-trigger", "self"]
    );
}

#[test]
fn dependencies_before_direct_preserves_all_owners_and_reasons() {
    let fixture = fixture();
    copy_config(fixture.path(), "dependencies-first.yml");
    let report = json(&plan(
        fixture.path(),
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "src/shared.test.ts",
            "--environment",
            "dependencies-first",
            "--json",
        ],
    ));
    assert_shared_has_all_owners_and_reasons(&report);
}

#[test]
fn zero_budget_dependencies_merges_targeted_reason_into_direct_selection() {
    let fixture = fixture();
    copy_config(fixture.path(), "direct-first.yml");
    let report = json(&plan(
        fixture.path(),
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "src/shared.test.ts",
            "--environment",
            "direct-first",
            "--json",
        ],
    ));
    assert_shared_has_all_owners_and_reasons(&report);
}

#[test]
fn lockfile_seed_merges_with_targeted_shared_test() {
    let fixture = fixture();
    git_init(fixture.path());
    std::fs::copy(
        fixture.path().join("lockfiles/after-pnpm-lock.yaml"),
        fixture.path().join("pnpm-lock.yaml"),
    )
    .unwrap();
    let report = json(&plan(
        fixture.path(),
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--environment",
            "shared",
            "--json",
        ],
    ));
    let shared = report["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["test_file"] == "src/shared.test.ts")
        .unwrap();
    assert_eq!(
        shared["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|target| target["project"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["database", "web"]
    );
    assert!(shared["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason["changed_file"] == "pnpm-lock.yaml"));
    assert!(shared["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason["via"] == serde_json::json!(["configured-trigger"])));
}

#[test]
fn synthesized_dependencies_precedes_budget_consuming_sample() {
    let fixture = fixture();
    copy_config(fixture.path(), "sample-before-synthetic.yml");
    let report = json(&plan(
        fixture.path(),
        "vitest",
        &[
            "--changed-file",
            "migrations/001.sql",
            "--environment",
            "sample-first",
            "--json",
        ],
    ));
    assert_eq!(selected_files(&report), vec!["src/db/db.test.ts"]);
    assert_eq!(
        report["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|group| group["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["dependencies", "sample"]
    );
    assert_eq!(
        report["groups"][0]["selected"],
        serde_json::json!(["src/db/db.test.ts"])
    );
    assert_eq!(report["groups"][1]["selected"], serde_json::json!([]));
    assert_eq!(
        report["selected_tests"][0]["reasons"][0]["via"],
        serde_json::json!(["configured-trigger"])
    );
    assert_eq!(
        report["selected_tests"][0]["targets"][0]["project"],
        "database"
    );
    assert_eq!(report["fallback_triggered"], false);
}
