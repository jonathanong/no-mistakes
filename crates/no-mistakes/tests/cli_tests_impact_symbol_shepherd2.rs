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

#[test]
fn dependencies_symbols_type_declaration_scopes_shadow_imports() {
    let root = fixture("symbol-export");
    for entrypoint in [
        "scoped-type-declarations.mts#aliasRun",
        "scoped-type-declarations.mts#interfaceRun",
    ] {
        let output = run(&[
            "dependencies",
            entrypoint,
            "--root",
            root.to_str().unwrap(),
            "--symbols",
            "--relationship",
            "import-type",
            "--format",
            "paths",
        ]);

        assert!(output.status.success());
        assert_eq!(stdout(&output), "", "{entrypoint}");
    }
}

#[test]
fn dependencies_symbols_visit_star_reexports_by_type_and_value_kind() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "star-dual-consumer.mts#value",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-static",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("star-dual-source.mts#Dual\n"));
}

#[test]
fn dependencies_symbols_scope_named_default_functions_by_export_local() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-named-function.mts#default",
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
fn dependencies_symbols_attribute_computed_object_keys_to_alias() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "computed-object-key-alias.mts#api",
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
fn dependencies_symbols_resolve_bare_namespace_references() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "bare-namespace-return.mts#getSource",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts\n");
}

#[test]
fn dependencies_symbols_record_jsx_member_components() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "jsx-member-view.tsx#View",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("jsx-member-ui.tsx#Button\n"));
}

#[test]
fn dependencies_symbols_visit_exported_generic_constraints() {
    let root = fixture("symbol-export");
    for entrypoint in [
        "generic-constraint.mts#Box",
        "generic-constraint.mts#HasValue",
    ] {
        let output = run(&[
            "dependencies",
            entrypoint,
            "--root",
            root.to_str().unwrap(),
            "--symbols",
            "--relationship",
            "import-type",
            "--format",
            "paths",
        ]);

        assert!(output.status.success());
        assert_eq!(stdout(&output), "source.mts#SourceShape\n", "{entrypoint}");
    }
}

#[test]
fn dependents_symbols_skip_ambiguous_star_reexports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependents",
        "star-ambiguous-a.mts#alpha",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(!stdout(&output).contains("star-ambiguous-consumer.mts#value\n"));
}

#[test]
fn dependencies_symbols_skip_value_exports_under_type_star() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-star-value-consumer.mts#Use",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(!stdout(&output).contains("type-star-value-source.mts#Foo\n"));
}

#[test]
fn dependencies_symbols_scope_inline_arrow_callback_parameters() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "inline-arrow-shadow.mts#api",
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
fn dependencies_symbols_expand_runtime_file_edges_from_exported_symbol() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runRuntimeEdges",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    assert!(out.contains("routes/users.mts\n"), "{out}");
    assert!(out.contains("worker.mts\n"), "{out}");
}

#[test]
fn dependencies_symbols_do_not_expand_unrelated_runtime_file_edges() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#formatRuntimeEdges",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_expand_member_process_runtime_edges() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runMemberRuntimeEdge",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("worker.mts\n"));
}

#[test]
fn dependencies_symbols_skip_scopes_without_runtime_calls_in_runtime_files() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#unrelatedRuntimeCall",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependencies_symbols_ignore_unrelated_runtime_method_names() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#unrelatedRuntimeMethods",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn dependents_symbols_continue_through_intermediate_owner_files() {
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
    assert!(
        stdout(&output).contains("intermediate-owner-side-effect.test.mts\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn dependents_symbols_do_not_expand_namespace_reexports_as_star_barrels() {
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
    assert!(!stdout(&output).contains("namespace-star-barrel.mts#alpha\n"));
}

#[test]
fn dependents_symbols_handle_duplicate_star_reexport_sources() {
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
    assert!(stdout(&output).contains("star-duplicate-barrel.mts#alpha\n"));
}

#[test]
fn dependents_symbols_ignore_string_named_star_exports() {
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
    assert!(!stdout(&output).contains("star-string-name-barrel.mts#alpha\n"));
}

#[test]
fn dependencies_symbols_default_interface_uses_default_scope() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-interface.mts#default",
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
fn dependencies_symbols_keep_value_edges_for_dual_direct_reexports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "dual-direct-barrel.mts#Foo",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-static",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("dual-direct-source.mts#Foo\n"));
}

#[test]
fn dependencies_symbols_follow_local_namespace_export_members() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-local-member-consumer.mts#value",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("namespace-direct-source.mts#alpha\n"));
}
