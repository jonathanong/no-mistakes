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
fn dependencies_symbols_ignore_local_function_shadowing() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "local-function-shadow.mts#run",
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
fn dependencies_symbols_follow_nested_function_helpers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "nested-helper-function.mts#run",
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
fn dependencies_symbols_capture_references_for_later_named_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-export-reference.mts#beta",
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
fn dependencies_symbols_treat_default_arrow_as_default_symbol() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-arrow.mts#default",
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
fn dependencies_symbols_follow_exported_alias_to_helper_reference() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-alias-helper.mts#publicApi",
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
fn dependencies_symbols_follow_exported_class_members() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-class-member.mts#Client",
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
fn dependencies_symbols_follow_exported_object_methods() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-object-method.mts#api",
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
fn dependencies_symbols_follow_exported_object_methods_with_spreads() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-object-spread.mts#api",
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
fn dependents_symbols_file_roots_seed_exported_symbols() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependents",
        "file-root-source.mts",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "file-root-consumer.mts\nfile-root-consumer.mts#value\n"
    );
}

#[test]
fn dependents_symbols_file_roots_without_exports_are_allowed() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependents",
        "no-export-file.mts",
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
fn dependencies_symbols_include_exported_arrow_variable_annotations() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-arrow-annotation.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "typed-handler.mts#Handler\n");
}

#[test]
fn dependencies_symbols_type_parameters_shadow_imported_types() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "generic-type-shadow.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_do_not_follow_default_function_helper_references() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-helper-reference-only.mts#default",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}
