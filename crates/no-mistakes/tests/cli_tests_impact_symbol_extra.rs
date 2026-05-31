use std::path::PathBuf;
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

#[test]
fn dependencies_symbols_type_filters_direct_type_reexports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-barrel.mts#SourceShape",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "source.mts#SourceShape\n");
}

#[test]
fn dependents_symbols_keep_namespace_fallback_for_no_export_importers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependents",
        "source.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout(&output).contains("namespace-side-effect.mts\n"));
}

#[test]
fn dependencies_symbols_ignores_shadowed_callable_import_references() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "shadowed-call-import.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_respects_block_scoped_shadowing() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "block-shadow.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_do_not_follow_uninvoked_helper_reference() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "uncalled-helper-reference.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_ignores_shadowed_namespace_references() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-shadow.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_include_namespace_asset_imports_as_files() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-asset-consumer.mts#get",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "asset",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "data.json\n");
}

#[test]
fn dependencies_symbols_file_roots_cross_owner_bridge_with_relationship_filter() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "dynamic-loader.mts",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-dynamic",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout(&output).contains("dynamic-chunk.mts\n"));
}

#[test]
fn dependencies_symbols_can_target_external_namespace_imports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "external-namespace-schema.mts#schema",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--target-module",
        "zod",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "zod\n");
}

#[test]
fn dependencies_symbols_resolves_external_reexports_to_modules() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "external-reexport.mts#z",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--target-module",
        "zod",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "zod\n");
}

#[test]
fn dependencies_symbols_resolves_workspace_namespace_imports() {
    let root = fixture("symbol-workspace");
    let output = run(&[
        "dependencies",
        "packages/app/src/consumer.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "packages/core/src/index.mts#alpha\n");
}

#[test]
fn dependencies_symbols_resolves_workspace_reexports() {
    let root = fixture("symbol-workspace");
    let output = run(&[
        "dependencies",
        "packages/app/src/workspace-barrel.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "packages/core/src/index.mts#alpha\n");
}

#[test]
fn dependencies_symbols_keep_anonymous_function_bindings_scoped() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "anonymous-default-param-scope.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_walks_helper_references_without_calls() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "helper-reference-only.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "source.mts#gamma\n");
}

#[test]
fn dependencies_symbols_resolves_jsx_component_references() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "parent.tsx#Parent",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "child.tsx#Child\n");
}

#[test]
fn dependencies_symbols_generic_type_parameters_shadow_imports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "generic-shadow.mts#Box",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_resolves_scoped_workspace_imports() {
    let root = fixture("symbol-workspace");
    let output = run(&[
        "dependencies",
        "packages/app/src/scoped-workspace-loader.mts#load",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "workspace",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = stdout(&output);
    assert!(stdout.contains("packages/core/src/index.mts\n"));
    assert!(!stdout.contains("@fixture/core"));
}

#[test]
fn dependencies_symbols_preserve_workspace_kind_through_star_barrels() {
    let root = fixture("symbol-workspace");
    let output = run(&[
        "dependencies",
        "packages/app/src/workspace-barrel-consumer.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "workspace",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "packages/core/src/index.mts#alpha\n");
}

#[test]
fn dependencies_symbols_preserve_workspace_kind_for_star_reexports() {
    let root = fixture("symbol-workspace");
    let output = run(&[
        "dependencies",
        "packages/app/src/workspace-star-barrel.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "workspace",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "packages/core/src/index.mts#alpha\n");
}

#[test]
fn tests_plan_rejects_symbol_entrypoint_without_symbols() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "plan",
        "--entrypoint",
        "service.mts#parse",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uses `#symbol`; pass --symbols"));
    assert!(!stderr.contains("panicked"));
}

#[test]
fn tests_impact_symbols_select_test_file_that_owns_reached_symbol() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "utils.mts#parseDate",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout(&output).contains("helper-export.test.mts\n"));
}
