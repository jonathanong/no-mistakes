//! End-to-end coverage for issue #418 `tests impact` enhancements:
//! always-surfaced `.mock.test.*` stubs, `next/dynamic()` boundary traversal,
//! and the registry hint.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn impact_json(root: &Path, entrypoints: &[&str]) -> serde_json::Value {
    let mut args = vec!["tests", "impact"];
    args.extend_from_slice(entrypoints);
    let root_str = root.to_str().unwrap();
    args.extend_from_slice(&["--root", root_str, "--json"]);
    let output = run(&args);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

fn selected_files(plan: &serde_json::Value) -> Vec<String> {
    let mut files: Vec<String> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap().to_string())
        .collect();
    files.sort();
    files
}

fn registry_hints(plan: &serde_json::Value) -> Vec<String> {
    let mut hints: Vec<String> = plan["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["type"] == "registry-hint")
        .map(|w| w["message"].as_str().unwrap().to_string())
        .collect();
    hints.sort();
    hints
}

// ── Part 1: always-surface `.mock.test.*` stubs ─────────────────────────────

#[test]
fn impact_surfaces_suite_excluded_mock_stub_via_always_include() {
    let root = fixture("tests-impact-mock-stub");
    let plan = impact_json(&root, &["widget.mts"]);
    // The mock stub is `exclude`d from the vitest suite but surfaced anyway by
    // `tests.impact.alwaysIncludeTests`. `helper.mts` is a non-test importer and
    // must NOT appear.
    assert_eq!(
        selected_files(&plan),
        vec!["widget.mock.test.mts", "widget.test.mts"]
    );
}

#[test]
fn impact_omits_mock_stub_when_always_include_not_configured() {
    let root = fixture("tests-impact-mock-stub");
    // Override config with one that excludes the stub but does NOT opt in.
    let config = root.join("no-always-include.no-mistakes.yml");
    let config = config.to_str().unwrap();
    let root_str = root.to_str().unwrap();
    let output = run(&[
        "tests",
        "impact",
        "widget.mts",
        "--root",
        root_str,
        "--config",
        config,
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(selected_files(&plan), vec!["widget.test.mts"]);
}

#[test]
fn tests_plan_does_not_apply_always_include_stub_override() {
    let root = fixture("tests-impact-mock-stub");
    let root_str = root.to_str().unwrap();
    // `tests plan` schedules tests to run and must respect the suite exclude even
    // though `tests.impact.alwaysIncludeTests` is configured (it is impact-only).
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root_str,
        "--changed-file",
        "widget.mts",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let names = selected_files(&plan);
    assert!(
        !names.contains(&"widget.mock.test.mts".to_string()),
        "tests plan must not surface the suite-excluded stub: {names:?}"
    );
    assert!(names.contains(&"widget.test.mts".to_string()), "{names:?}");
}

// ── Part 2: next/dynamic() boundary traversal ───────────────────────────────

#[test]
fn impact_traverses_next_dynamic_boundary_at_medium_confidence() {
    let root = fixture("tests-impact-next-dynamic");
    let plan = impact_json(&root, &["foo.mts"]);
    let selected = plan["selected_tests"].as_array().unwrap();
    let test = selected
        .iter()
        .find(|t| t["test_file"] == "caller.test.mts")
        .expect("caller.test.mts should be surfaced through the next/dynamic boundary");
    assert_eq!(test["confidence"], "medium");
    assert!(
        plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["type"] == "dynamic-import"),
        "expected a dynamic-import warning: {plan:?}"
    );
}

#[test]
fn impact_traverses_default_export_next_dynamic_boundary() {
    let root = fixture("tests-impact-next-dynamic");
    // `export default dynamic(() => import('./foo.mts'))` must also surface its
    // test through the dynamic boundary.
    let plan = impact_json(&root, &["foo.mts"]);
    let names = selected_files(&plan);
    assert!(
        names.contains(&"default-caller.test.mts".to_string()),
        "expected default-caller.test.mts to be surfaced: {names:?}"
    );
}

#[test]
fn impact_traverses_default_alias_next_dynamic_boundary() {
    let root = fixture("tests-impact-next-dynamic");
    // `const Lazy = dynamic(...); export default Lazy;` must also surface its test.
    let plan = impact_json(&root, &["foo.mts"]);
    let names = selected_files(&plan);
    assert!(
        names.contains(&"aliased-default.test.mts".to_string()),
        "expected aliased-default.test.mts to be surfaced: {names:?}"
    );
}

#[test]
fn impact_traverses_wrapped_default_next_dynamic_boundary() {
    let root = fixture("tests-impact-next-dynamic");
    // `const Lazy = dynamic(...); export default wrap(Lazy);` (HOC wrapper) must
    // also surface its test.
    let plan = impact_json(&root, &["foo.mts"]);
    let names = selected_files(&plan);
    assert!(
        names.contains(&"wrapped-default.test.mts".to_string()),
        "expected wrapped-default.test.mts to be surfaced: {names:?}"
    );
}

#[test]
fn impact_does_not_overseed_parenthesized_function_default() {
    let root = fixture("tests-impact-next-dynamic");
    // `paren-fn-default.mts` only assigns the lazy binding to an unused private
    // const and shadows its name inside a parenthesized function default, so its
    // test must NOT be surfaced by a change to `foo.mts`.
    let plan = impact_json(&root, &["foo.mts"]);
    let names = selected_files(&plan);
    assert!(
        !names.contains(&"paren-fn-default.test.mts".to_string()),
        "parenthesized function default must not over-seed: {names:?}"
    );
}

#[test]
fn impact_does_not_overseed_shadowed_or_type_default_references() {
    let root = fixture("tests-impact-next-dynamic");
    // A lazy binding referenced only by a shadowed name inside a nested callback,
    // or only in a type position, must NOT pull its test into a `foo.mts` change.
    let plan = impact_json(&root, &["foo.mts"]);
    let names = selected_files(&plan);
    assert!(
        !names.contains(&"nested-callback-default.test.mts".to_string()),
        "nested callback shadow must not over-seed: {names:?}"
    );
    assert!(
        !names.contains(&"type-collision-default.test.mts".to_string()),
        "type-position reference must not over-seed: {names:?}"
    );
}

#[test]
fn impact_registry_hint_skips_deeply_nested_uninvoked_loader() {
    let root = fixture("tests-impact-registry");
    // `deep-loader-registry.mts` buries its dynamic import in an uninvoked nested
    // function, so reachability prunes the edge and no hint is emitted for it.
    let plan = impact_json(&root, &["feature.mts"]);
    assert!(
        !registry_hints(&plan)
            .iter()
            .any(|hint| hint.contains("deep-loader-registry.mts")),
        "deeply-nested uninvoked loader must not produce a hint: {:?}",
        registry_hints(&plan)
    );
}

// ── Part 3: registry hint ───────────────────────────────────────────────────

#[test]
fn impact_emits_registry_hint_per_target_and_registry() {
    let root = fixture("tests-impact-registry");
    // Both features are registered in the dynamic-import registry; only
    // `feature` is also in `widgets-registry.mts`. `plain-consumer.mts` imports
    // the target but is not a registry, so it produces no hint.
    let plan = impact_json(&root, &["feature.mts", "feature2.mts"]);
    assert_eq!(
        registry_hints(&plan),
        vec![
            "`feature.mts` is registered in `auth-gated-code-splitting.mts`; verify the registry entry is up to date".to_string(),
            "`feature.mts` is registered in `widgets-registry.mts`; verify the registry entry is up to date".to_string(),
            "`feature2.mts` is registered in `auth-gated-code-splitting.mts`; verify the registry entry is up to date".to_string(),
        ]
    );
}

#[test]
fn impact_registry_hint_dedups_repeated_target() {
    let root = fixture("tests-impact-registry");
    // The same target passed twice must still yield one hint per registry.
    let plan = impact_json(&root, &["feature.mts", "feature.mts"]);
    assert_eq!(
        registry_hints(&plan),
        vec![
            "`feature.mts` is registered in `auth-gated-code-splitting.mts`; verify the registry entry is up to date".to_string(),
            "`feature.mts` is registered in `widgets-registry.mts`; verify the registry entry is up to date".to_string(),
        ]
    );
}

#[test]
fn impact_emits_no_registry_hint_without_registries_config() {
    let root = fixture("tests-impact-next-dynamic");
    // This fixture configures no registries, so no registry hints appear.
    let plan = impact_json(&root, &["foo.mts"]);
    assert!(registry_hints(&plan).is_empty());
}

#[test]
fn impact_emits_no_registry_hint_when_target_has_no_importers() {
    let root = fixture("tests-impact-registry");
    // The registry file itself has no importers, so the reverse-dependent scan
    // returns nothing and no hint is emitted.
    let plan = impact_json(&root, &["auth-gated-code-splitting.mts"]);
    assert!(registry_hints(&plan).is_empty());
}

#[test]
fn impact_tolerates_malformed_registry_glob() {
    let root = fixture("tests-impact-registry");
    let config = root.join("malformed-registry.no-mistakes.yml");
    let config = config.to_str().unwrap();
    let root_str = root.to_str().unwrap();
    let output = run(&[
        "tests",
        "impact",
        "feature.mts",
        "--root",
        root_str,
        "--config",
        config,
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    // A malformed glob degrades to no registries, so no hints (but no crash).
    assert!(registry_hints(&plan).is_empty());
}

#[test]
fn impact_registry_hint_ignores_type_only_imports() {
    let root = fixture("tests-impact-registry");
    // `types-registry.mts` only `import type`s the target, which is not a runtime
    // registration, so it must not appear in any registry hint.
    let plan = impact_json(&root, &["feature.mts"]);
    let hints = registry_hints(&plan);
    assert!(
        !hints.iter().any(|hint| hint.contains("types-registry.mts")),
        "type-only import must not produce a registry hint: {hints:?}"
    );
    // The value-import registries still produce hints.
    assert!(hints
        .iter()
        .any(|hint| hint.contains("widgets-registry.mts")));
}

#[test]
fn impact_skips_registry_hint_for_symbol_entrypoint() {
    let root = fixture("tests-impact-registry");
    let root_str = root.to_str().unwrap();
    // A symbol-scoped entrypoint asks about one export; a file-level registry
    // hint could be unrelated, so none is emitted.
    let output = run(&[
        "tests",
        "impact",
        "feature.mts#feature",
        "--root",
        root_str,
        "--symbols",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(registry_hints(&plan).is_empty());
}

#[test]
fn impact_registry_hint_renders_in_markdown() {
    let root = fixture("tests-impact-registry");
    let root_str = root.to_str().unwrap();
    let output = run(&[
        "tests",
        "impact",
        "feature.mts",
        "--root",
        root_str,
        "--format",
        "md",
    ]);
    assert!(output.status.success());
    let md = stdout(&output);
    assert!(
        md.contains("⚠️ **registry-hint**") && md.contains("auth-gated-code-splitting.mts"),
        "markdown should render the registry hint: {md}"
    );
}
