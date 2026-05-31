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
fn dependencies_symbols_named_data_exports_follow_local_aliases() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-data-alias.mts#publicApi",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_record_exported_enum_initializers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-enum.mts#Numbers",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_record_exported_class_computed_keys() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-class-computed-key.mts#Client",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_expand_dynamic_import_file_targets() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "dynamic-loader.mts#load",
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
    assert_eq!(stdout(&output), "dynamic-chunk.mts\nsource.mts\n");
}

#[test]
fn tests_plan_symbols_changed_file_seeds_exported_symbols() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "plan",
        "--changed-file",
        "alpha-source.mts",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let test_files: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(test_files, vec!["alpha-consumer.test.mts"]);
}

#[test]
fn tests_plan_symbols_entrypoint_file_seeds_exported_symbols() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "plan",
        "--entrypoint",
        "alpha-source.mts",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let test_files: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(test_files, vec!["alpha-consumer.test.mts"]);
}

#[test]
fn dependencies_symbols_file_roots_expand_exported_symbol_edges() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "file-root-export.mts",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("file-root-export.mts#api\n"));
    assert!(stdout.contains("source.mts#alpha\n"));
}

#[test]
fn dependencies_symbols_scope_later_exported_variable_initializers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-var-export.mts#publicApi",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_scope_later_exported_type_declarations() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-type-export.mts#Public",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#SourceShape\n");
}

#[test]
fn dependencies_symbols_scope_destructured_export_initializers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "destructured-export.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "destructure-source.mts#source\n");
}

#[test]
fn dependencies_symbols_keep_type_shadows_scoped_to_declaring_function() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-shadow-scope.mts#PublicShape",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#SourceShape\n");
}

#[test]
fn dependencies_symbols_type_parameters_do_not_shadow_values() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-parameter-value-call.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_hoisted_local_functions_shadow_imports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "hoisted-local-function.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependents_symbols_follow_nested_star_reexports() {
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

    assert!(output.status.success());
    assert!(stdout(&output).contains("nested-star-consumer.mts#value\n"));
}

#[test]
fn dependencies_symbols_preserve_references_inside_object_aliases() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "object-alias-spread.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}
