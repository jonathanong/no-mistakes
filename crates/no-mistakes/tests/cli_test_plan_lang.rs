use crate::common;

use common::{fixture, run, stdout};

fn selected_files(plan: &serde_json::Value) -> Vec<&str> {
    plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect()
}

fn plan(framework: &str, root: &std::path::Path, changed: &str) -> serde_json::Value {
    let output = run(&[
        "test",
        "plan",
        framework,
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        changed,
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

#[test]
fn test_plan_python_selects_owning_import_test() {
    let root = fixture("python-test-plan");
    let plan = plan("python", &root, "app/users.py");
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(
        selected_files(&plan),
        vec!["app/test_users.py", "app/test_views.py"]
    );
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(target["runner"], "python");
    assert_eq!(target["base_command"], serde_json::json!(["pytest"]));
    assert_eq!(
        target["runner_args"],
        serde_json::json!(["app/test_users.py"])
    );
}

#[test]
fn test_plan_python_route_hop_selects_view_test() {
    let root = fixture("python-test-plan");
    let plan = plan("python", &root, "app/urls.py");
    assert!(selected_files(&plan).contains(&"app/test_views.py"));
}

#[test]
fn test_plan_python_queue_hop_selects_worker_test() {
    let root = fixture("python-test-plan");
    let plan = plan("python", &root, "app/enqueue.py");
    assert!(selected_files(&plan).contains(&"app/test_tasks.py"));
}

#[test]
fn test_plan_python_untraceable_source_triggers_native_fallback() {
    let root = fixture("python-test-plan");
    let plan = plan("python", &root, "app/unused.py");
    assert_eq!(plan["fallback_triggered"], true);
    let selected = selected_files(&plan);
    assert!(selected.contains(&"app/test_users.py"));
    assert!(selected.contains(&"app/test_views.py"));
    assert!(selected.contains(&"app/test_tasks.py"));
}

#[test]
fn test_plan_python_empty_packages_selects_nothing() {
    let root = fixture("python-test-plan");
    let output = run(&[
        "test",
        "plan",
        "python",
        "--root",
        root.to_str().unwrap(),
        "--config",
        root.join("empty.no-mistakes.yml").to_str().unwrap(),
        "--changed-file",
        "app/users.py",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(selected_files(&plan).is_empty());
}

#[test]
fn test_plan_go_selects_package_test() {
    let root = fixture("go-test-plan");
    let plan = plan("go", &root, "pkg/ping.go");
    assert_eq!(selected_files(&plan), vec!["pkg/ping_test.go"]);
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(target["base_command"], serde_json::json!(["go", "test"]));
    assert_eq!(target["runner_args"], serde_json::json!(["./pkg"]));
}

#[test]
fn test_plan_go_queue_hop_selects_worker_test() {
    let root = fixture("go-test-plan");
    let plan = plan("go", &root, "worker/enqueue.go");
    assert!(selected_files(&plan).contains(&"worker/tasks_test.go"));
}

#[test]
fn test_plan_cargo_sibling_and_integration_commands() {
    let root = fixture("cargo-test-plan");
    let plan = plan("cargo", &root, "app/src/lib.rs");
    let selected = selected_files(&plan);
    assert!(selected.contains(&"app/src/tests.rs"));
    assert!(selected.contains(&"app/tests/integration.rs"));
    let commands = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|test| test["targets"].as_array().unwrap())
        .map(|target| {
            let mut parts = target["base_command"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            parts.extend(
                target["runner_args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string()),
            );
            parts.join(" ")
        })
        .collect::<Vec<_>>();
    assert!(commands
        .iter()
        .any(|cmd| cmd == "cargo test --manifest-path app/Cargo.toml -p app --lib"));
    assert!(commands
        .iter()
        .any(|cmd| cmd == "cargo test --manifest-path app/Cargo.toml -p app --test integration"));
}

#[test]
fn test_plan_rails_selects_minitest_and_rspec() {
    let root = fixture("rails-test-plan");
    let users = plan("rails", &root, "app/models/user.rb");
    assert_eq!(selected_files(&users), vec!["test/models/user_test.rb"]);
    assert_eq!(
        users["selected_tests"][0]["targets"][0]["base_command"],
        serde_json::json!(["bin/rails", "test"])
    );
    let job = plan("rails", &root, "app/jobs/welcome_job.rb");
    assert_eq!(selected_files(&job), vec!["spec/jobs/welcome_job_spec.rb"]);
    assert_eq!(
        job["selected_tests"][0]["targets"][0]["base_command"],
        serde_json::json!(["rspec"])
    );
}

#[test]
fn test_plan_php_laravel_uses_artisan() {
    let root = fixture("php-test-plan");
    let plan = plan("php", &root, "app/Http/Controllers/UserController.php");
    assert_eq!(selected_files(&plan), vec!["tests/UserControllerTest.php"]);
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(
        target["base_command"],
        serde_json::json!(["php", "artisan", "test"])
    );
}

#[test]
fn test_plan_vitest_fields_unchanged_when_language_packages_configured() {
    let root = fixture("vitest-lang-parity");
    let with_python = plan("vitest", &root, "src/value.ts");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--config",
        root.join("vitest-only.no-mistakes.yml").to_str().unwrap(),
        "--changed-file",
        "src/value.ts",
        "--json",
    ]);
    assert!(output.status.success());
    let without_python: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        selected_files(&with_python),
        selected_files(&without_python)
    );
    assert_eq!(
        with_python["fallback_triggered"],
        without_python["fallback_triggered"]
    );
}

#[test]
fn test_plan_jest_selects_owning_import_test() {
    let root = fixture("jest-test-plan");
    let plan = plan("jest", &root, "src/value.ts");
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(selected_files(&plan), vec!["src/value.test.ts"]);
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(target["runner"], "jest");
    assert_eq!(target["base_command"], serde_json::json!(["jest"]));
    assert_eq!(target["config"], "jest.config.js");
    assert_eq!(
        target["runner_args"],
        serde_json::json!(["--config", "jest.config.js", "src/value.test.ts"])
    );
}

#[test]
fn test_plan_jest_empty_configs_selects_nothing() {
    let root = fixture("jest-test-plan");
    let output = run(&[
        "test",
        "plan",
        "jest",
        "--root",
        root.to_str().unwrap(),
        "--config",
        root.join("empty.no-mistakes.yml").to_str().unwrap(),
        "--changed-file",
        "src/value.ts",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(selected_files(&plan).is_empty());
}

#[test]
fn test_plan_vitest_fields_unchanged_when_jest_configs_configured() {
    let root = fixture("jest-test-plan");
    let with_jest = plan("vitest", &root, "src/value.ts");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--config",
        root.join("vitest-only.no-mistakes.yml").to_str().unwrap(),
        "--changed-file",
        "src/value.ts",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let without_jest: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(with_jest, without_jest);
}

#[test]
fn cargo_manifest_targets_and_ambiguous_helpers_match_cli_contract() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/cargo-target-ownership"),
    );
    let root_arg = root.to_str().unwrap();
    let output = run(&[
        "tests",
        "targets",
        "cargo",
        "--root",
        root_arg,
        "crates/custom/library/tests.rs",
        "crates/engine/src/tests.rs",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let known = report["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["testFile"] == "crates/custom/library/tests.rs")
        .unwrap();
    assert_eq!(
        known["targets"][0]["runner_args"],
        serde_json::json!([
            "--manifest-path",
            "crates/custom/Cargo.toml",
            "-p",
            "renamed-package",
            "--lib"
        ])
    );
    assert_eq!(report["warnings"][0]["type"], "cargo-target-unresolved");
    let ambiguous = plan("cargo", &root, "crates/engine/src/tests.rs");
    assert!(ambiguous.get("execution_targets").is_none());
    assert!(ambiguous["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "cargo-target-unresolved"));
    let commands = run(&[
        "tests",
        "targets",
        "cargo",
        "--root",
        root_arg,
        "crates/engine/src/tests.rs",
        "--format",
        "commands",
    ]);
    assert!(!commands.status.success());
    assert!(
        commands.stdout.is_empty(),
        "ambiguous ownership must not print runnable commands"
    );
}

#[test]
fn cargo_external_entries_keep_multiple_declaring_selectors_and_package_scopes() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/cargo-external-targets"),
    );
    let unscoped = plan("cargo", &root, "shared/tests/smoke.rs");
    let targets = unscoped["selected_tests"][0]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2);
    assert_eq!(
        targets[0]["runner_args"],
        serde_json::json!([
            "--manifest-path",
            "a/Cargo.toml",
            "-p",
            "declaring-a",
            "--test",
            "a-shared"
        ])
    );
    assert_eq!(
        targets[1]["runner_args"],
        serde_json::json!([
            "--manifest-path",
            "b/Cargo.toml",
            "-p",
            "declaring-b",
            "--test",
            "b-shared"
        ])
    );
    assert_eq!(unscoped["fallback_triggered"], false);
    let output = run(&[
        "tests",
        "targets",
        "cargo",
        "--root",
        root.to_str().unwrap(),
        "--config",
        root.join("a.yml").to_str().unwrap(),
        "shared/tests/smoke.rs",
        "b/tests/physical.rs",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    for test in report["tests"].as_array().unwrap() {
        assert_eq!(test["targets"].as_array().unwrap().len(), 1);
        assert_eq!(test["targets"][0]["project"], "declaring-a");
    }
    assert!(report["warnings"].as_array().unwrap().is_empty());
}
