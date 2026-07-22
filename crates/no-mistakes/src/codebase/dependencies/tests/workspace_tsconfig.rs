use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

fn framework_project_alias_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/framework-project-alias"),
    )
}

fn unresolved_package_extends_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/unresolved-package-extends"),
    )
}

fn dependencies(root: &Path, file: &str, tsconfig: Option<&str>) -> Value {
    let mut command = Command::cargo_bin("no-mistakes").unwrap();
    command
        .args(["dependencies", file, "--root"])
        .arg(root)
        .args(["--relationship", "import", "--format", "json"]);
    if let Some(tsconfig) = tsconfig {
        command.args(["--tsconfig", tsconfig]);
    }
    serde_json::from_slice(&command.assert().success().get_output().stdout).unwrap()
}

fn paths(report: &Value) -> Vec<&str> {
    report["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["path"].as_str())
        .collect()
}

fn test_plan(root: &Path) -> Value {
    let mut command = Command::cargo_bin("no-mistakes").unwrap();
    command
        .args(["tests", "plan", "vitest", "--root"])
        .arg(root)
        .args([
            "--changed-file",
            "packages/shared/src/message.ts",
            "--format",
            "json",
        ]);
    serde_json::from_slice(&command.assert().success().get_output().stdout).unwrap()
}

#[test]
fn shared_traversal_prepares_workspace_config_ownership() {
    let root = fixture_root();
    let allowed =
        std::collections::HashSet::from([crate::codebase::dependencies::EdgeKind::Import]);
    let plan = crate::codebase::dependencies::graph::GraphBuildPlan::from_allowed(Some(&allowed));
    let framework_plan = crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(plan);
    let mut shared =
        crate::codebase::dependencies::SharedTraversalContext::prepare_with_framework_plan(
            root.clone(),
            None,
            None,
            plan,
            framework_plan,
        )
        .unwrap();
    assert_eq!(
        shared
            .tsconfig_catalog
            .provenance_for(&root.join("apps/web/src/entry.ts"))
            .config
            .as_deref(),
        Some(root.join("apps/web/tsconfig.json").as_path())
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new(
        &shared.tsconfig_catalog,
        shared.graph_files.visible(),
    );
    assert_eq!(
        resolver.resolve("@runtime/value", &root.join("apps/web/src/entry.ts")),
        Some(root.join("apps/web/src/runtime/value.ts"))
    );
    let graph = shared.request_graph(plan).unwrap();
    let entry = crate::codebase::dependencies::NodeId::File(root.join("apps/web/src/entry.ts"));
    let dependencies = graph.dependencies_of_node(&entry).unwrap();
    assert!(dependencies.iter().any(|(node, _)| {
        node == &crate::codebase::dependencies::NodeId::File(
            root.join("apps/web/src/runtime/value.ts"),
        )
    }));
}

#[test]
fn automatic_workspace_configs_keep_conflicting_aliases_isolated() {
    let root = fixture_root();
    let web = dependencies(&root, "apps/web/src/entry.ts", None);
    let worker = dependencies(&root, "services/worker/src/entry.ts", None);

    assert!(
        paths(&web).contains(&"apps/web/src/runtime/value.ts"),
        "{web:#?}"
    );
    assert!(!paths(&web).contains(&"services/worker/src/runtime/value.ts"));
    assert!(
        paths(&worker).contains(&"services/worker/src/runtime/value.ts"),
        "{worker:#?}"
    );
    assert!(!paths(&worker).contains(&"apps/web/src/runtime/value.ts"));
    assert!(
        paths(&web).contains(&"packages/shared/src/message.ts"),
        "{web:#?}"
    );
    assert!(
        paths(&worker).contains(&"packages/shared/src/message.ts"),
        "{worker:#?}"
    );
}

#[test]
fn explicit_workspace_tsconfig_forces_legacy_single_config_mode() {
    let root = fixture_root();
    let forced = dependencies(
        &root,
        "services/worker/src/entry.ts",
        Some("apps/web/tsconfig.json"),
    );
    let paths = paths(&forced);

    assert!(
        paths.contains(&"apps/web/src/runtime/value.ts"),
        "{forced:#?}"
    );
    assert!(
        !paths.contains(&"services/worker/src/runtime/value.ts"),
        "{forced:#?}"
    );
}

#[test]
fn workspace_shared_source_change_selects_importing_project_tests() {
    let plan = test_plan(&fixture_root());
    let selected = plan["selected_tests"].as_array().unwrap();
    let files = selected
        .iter()
        .filter_map(|test| test["test_file"].as_str())
        .collect::<Vec<_>>();

    assert!(files.contains(&"apps/web/tests/entry.test.ts"), "{plan:#?}");
    assert!(
        files.contains(&"services/worker/tests/entry.test.ts"),
        "{plan:#?}"
    );
}

#[test]
fn framework_config_alias_discovers_project_tests_with_the_importers_tsconfig() {
    let root = framework_project_alias_fixture_root();
    let mut command = Command::cargo_bin("no-mistakes").unwrap();
    command
        .args(["tests", "plan", "vitest", "--root"])
        .arg(&root)
        .args([
            "--changed-file",
            "apps/web/src/value.ts",
            "--format",
            "json",
        ]);
    let plan: Value =
        serde_json::from_slice(&command.assert().success().get_output().stdout).unwrap();
    assert!(
        plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|test| { test["test_file"] == "apps/web/tests/value.impact.ts" }),
        "{plan:#?}"
    );
}

#[test]
fn symlinked_workspace_root_keeps_alias_targets_in_graph_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let report = dependencies(&root, "src/entry.ts", None);
    assert!(paths(&report).contains(&"src/value.ts"), "{report:#?}");
    assert_eq!(report["tsconfig_provenance"][0]["config"], "tsconfig.json");
}

#[test]
fn automatic_workspace_diagnostics_are_stable_and_identify_ambiguous_ownership() {
    let root = fixture_root();
    let first = dependencies(&root, "apps/ambiguous/src/entry.ts", None);
    let second = dependencies(&root, "apps/ambiguous/src/entry.ts", None);

    assert_eq!(first["diagnostics"], second["diagnostics"]);
    let diagnostics = first["diagnostics"].as_array().unwrap();
    let ambiguous = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.to_string().contains("ambiguous"))
        .unwrap_or_else(|| panic!("expected an ambiguous ownership diagnostic: {diagnostics:#?}"));
    let rendered = ambiguous.to_string();
    assert!(rendered.contains("tsconfig.a.json"), "{ambiguous:#?}");
    assert!(rendered.contains("tsconfig.b.json"), "{ambiguous:#?}");
}

#[test]
fn invalid_automatic_root_config_warns_instead_of_aborting() {
    let root = fixture_root().join("invalid-extends");
    let report = dependencies(&root, "src/entry.ts", None);
    assert!(report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "invalid-extends"));
}

#[test]
fn unresolved_package_extends_warns_but_keeps_local_aliases() {
    let root = unresolved_package_extends_fixture_root();
    let report = dependencies(&root, "src/entry.ts", None);

    assert!(paths(&report).contains(&"src/value.ts"), "{report:#?}");
    assert!(report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| {
            diagnostic["kind"] == "invalid-extends"
                && diagnostic["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("@missing/tsconfig"))
        }));
}
