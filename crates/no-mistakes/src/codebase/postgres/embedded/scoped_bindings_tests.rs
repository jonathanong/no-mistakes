use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn scan(name: &str, options: &EmbeddedSqlOptions) -> Vec<String> {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(&path, &source, options)
        .calls
        .into_iter()
        .filter_map(|call| call.sql_text)
        .map(|sql| sql.trim_start_matches("SELECT id FROM ").to_string())
        .collect()
}

fn scoped(specifier: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured(specifier, &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()])
}

#[test]
fn factory_results_are_scanned_for_every_declaration_kind() {
    assert_eq!(
        scan("scoped-factory-decls.ts", &scoped("@example/db")),
        [
            "const_awaited",
            "const_plain",
            "let_awaited",
            "using_plain",
            "await_using_awaited",
            "wrapped"
        ]
    );
}

#[test]
fn factory_member_query_follows_member_query_option() {
    assert_eq!(
        scan("scoped-factory-member-query.ts", &scoped("@example/db")),
        ["member_query", "computed_query"]
    );
    // No importSpecifier and no `query` executor name: member queries stay off.
    let options = EmbeddedSqlOptions::configured("", &["other".into()])
        .with_scoped_executors(&["openTransaction".into()], &[]);
    assert!(scan("scoped-factory-member-query.ts", &options).is_empty());
}

#[test]
fn typed_parameters_are_scanned() {
    assert_eq!(
        scan("scoped-typed-params.ts", &scoped("@example/db")),
        [
            "plain_param",
            "optional_param",
            "arrow_param",
            "union_param",
            "destructured_param"
        ]
    );
}

#[test]
fn inline_type_specifier_alias_is_scanned() {
    assert_eq!(
        scan(
            "scoped-typed-inline-type-specifier.ts",
            &scoped("@example/db")
        ),
        ["inline_type_specifier"]
    );
}

#[test]
fn same_name_outside_the_declaring_scope_is_not_scanned() {
    assert_eq!(
        scan("scoped-negatives.ts", &scoped("@example/db")),
        ["typed_here", "inside_block"]
    );
}

#[test]
fn every_block_kind_scopes_bindings_and_odd_shapes_do_not_bind() {
    assert_eq!(
        scan("scoped-scope-kinds.ts", &scoped("@example/db")),
        ["static_block", "switch_case", "for_init", "quoted_key"]
    );
}

#[test]
fn imports_from_another_module_do_not_match() {
    assert!(scan("scoped-different-module.ts", &scoped("@example/db")).is_empty());
}

#[test]
fn subpath_imports_of_the_specifier_match() {
    assert_eq!(
        scan("scoped-subpath-imports.ts", &scoped("@example/db")),
        ["subpath_factory", "subpath_type", "subpath_inline_type"]
    );
}

#[test]
fn modules_sharing_only_a_string_prefix_do_not_match() {
    assert!(scan("scoped-lookalike-modules.ts", &scoped("@example/db")).is_empty());
}

#[test]
fn empty_specifier_matches_any_module() {
    assert_eq!(
        scan("scoped-any-module.ts", &scoped("")),
        ["any_module_factory", "any_module_type"]
    );
}

#[test]
fn defaults_leave_findings_unchanged() {
    // Same source, options absent: only the imported `query` executor is found.
    let baseline = EmbeddedSqlOptions::configured("@example/db", &[]);
    assert_eq!(scan("scoped-defaults.ts", &baseline), ["imported_query"]);
    assert_eq!(
        baseline,
        EmbeddedSqlOptions::configured("@example/db", &[]).with_scoped_executors(&[], &[])
    );
    assert_eq!(
        scan("scoped-defaults.ts", &scoped("@example/db")),
        ["imported_query", "factory_default_off", "type_default_off"]
    );
}

#[test]
fn scoped_names_are_sorted_and_deduplicated() {
    let options = EmbeddedSqlOptions::configured("", &[]).with_scoped_executors(
        &["b".into(), "a".into(), "b".into()],
        &["z".into(), "z".into()],
    );
    assert_eq!(options.executor_factory_names, ["a", "b"]);
    assert_eq!(options.executor_type_names, ["z"]);
}
