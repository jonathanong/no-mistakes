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
fn dependencies_symbols_exported_namespace_bindings_reach_source_file() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-local-export.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "namespace-direct-source.mts\n");
}

#[test]
fn dependents_symbols_carry_intermediate_star_export_shadows() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependents",
        "star-intermediate-source.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(!stdout(&output).contains("star-intermediate-consumer.mts#value\n"));
}

#[test]
fn dependencies_symbols_type_only_exports_do_not_shadow_values() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-shadow-consumer.mts#value",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-static",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("type-shadow-values.mts#Foo\n"));
}

#[test]
fn dependencies_symbols_scope_default_expression_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-expression.mts#default",
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
fn dependencies_symbols_scope_top_level_object_methods_by_object() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "object-method-collision.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}
