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
fn dependencies_symbols_keep_namespace_asset_reexports_as_assets() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-asset-reexport.mts#data",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "asset",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "data.json\n");
}

#[test]
fn dependencies_symbols_follow_hoisted_nested_helpers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "nested-hoisted-helper.mts#api",
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
fn dependencies_symbols_validate_hoisted_default_identifier_callables() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-hoisted-identifier.mts#default",
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
fn dependencies_symbols_scope_parenthesized_default_arrows() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-parenthesized-arrow.mts#default",
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
fn dependencies_symbols_attach_top_level_side_effect_imports_to_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-side-effect.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "setup-side-effect.mts\n");
}

#[test]
fn dependencies_symbols_do_not_attach_side_effect_imports_to_type_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-only-side-effect.mts#Public",
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
fn dependencies_symbols_treat_local_type_specifier_exports_as_type_only() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "local-type-specifier-export.mts#Public",
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
fn dependencies_symbols_keep_specifier_type_later_exports_scoped() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "specifier-type-later-export.mts#Public",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts\n");
}

#[test]
fn dependencies_symbols_scope_later_exported_computed_class_keys() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-export-computed-class.mts#Api",
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
fn dependencies_symbols_match_http_edges_to_symbol_call_paths() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runRuntimeEdges",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "routes/users.mts\n");
}

#[test]
fn dependencies_symbols_scope_destructuring_default_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "destructured-default-export.mts#api",
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
fn dependencies_symbols_follow_default_object_methods() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "default-object-method.mts#default",
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
fn dependencies_symbols_follow_value_aliases_read_by_callable_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "callable-reads-value-alias.mts#api",
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
fn dependencies_symbols_scope_object_spreads_under_alias() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "object-spread-alias.mts#api",
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
fn dependencies_symbols_follow_exported_class_expression_methods() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "exported-class-expression.mts#Api",
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
fn dependencies_symbols_preserve_bare_namespace_refs_with_member_refs() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "namespace-bare-and-member.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts\nsource.mts#alpha\n");
}

#[test]
fn dependencies_symbols_scope_later_exported_dynamic_imports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-export-dynamic-import.mts#lazy",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-dynamic",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts\n");
}

#[test]
fn dependencies_symbols_do_not_emit_owner_bridges_for_relationship_filters() {
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

    assert!(output.status.success());
    assert_eq!(stdout(&output), "dynamic-chunk.mts\n");
}

#[test]
fn dependencies_symbols_preserve_class_types_through_type_star_reexports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "type-star-class-barrel.mts#Api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "type-star-class-source.mts#Api\n");
}

#[test]
fn dependencies_symbols_scope_later_exported_enum_initializers() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-export-enum-initializer.mts#E",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-static",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_match_process_edges_to_symbol_call_paths() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runRuntimeEdges",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "worker.mts\n");
}

#[test]
fn dependencies_symbols_scope_later_exported_import_types() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "later-export-import-type.mts#Public",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-type",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "source.mts\n");
}

#[test]
fn dependencies_symbols_attach_top_level_import_calls_to_exports() {
    let root = fixture("symbol-export");
    let output = run(&[
        "dependencies",
        "top-level-import-call.mts#api",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "import-static",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "top-level-setup.mts#init\n");
}

#[test]
fn dependencies_symbols_accept_custom_http_clients() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runCustomHttpClient",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "http",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "routes/admin.mts\n");
}

#[test]
fn dependencies_symbols_accept_member_spawn_calls() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runMemberSpawnRuntimeEdge",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "member-worker.mts\n");
}

#[test]
fn dependencies_symbols_match_process_edges_with_cwd() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runCwdSpawnRuntimeEdge",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "scripts/cwd-worker.mts\n");
}

#[test]
fn dependencies_symbols_ignore_unresolved_member_spawn_calls() {
    let root = fixture("symbol-runtime-edges");
    let output = run(&[
        "dependencies",
        "client.mts#runMissingSpawnRuntimeEdge",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--relationship",
        "process",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}
