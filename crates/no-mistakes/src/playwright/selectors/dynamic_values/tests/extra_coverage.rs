use super::parse_and_collect;
use crate::playwright::ast;
use crate::playwright::test_support::fixture_path;
use std::path::Path;

fn page_extras_path() -> std::path::PathBuf {
    fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page-extras.tsx",
    ])
}

fn resolve_from_extras(local_name: &str) -> Vec<String> {
    let page_path = page_extras_path();
    let source = std::fs::read_to_string(&page_path).unwrap();
    ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values(local_name, program, &page_path)
    })
    .unwrap()
}

#[test]
fn deferred_cross_file_exports_resolve_against_precollected_static_values() {
    let page_path = crate::codebase::ts_resolver::normalize_path(&fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]));
    let selectors_path = page_path.with_file_name("selectors.ts");
    let visible = std::collections::HashSet::from([page_path.clone(), selectors_path.clone()]);
    let page_source = std::fs::read_to_string(&page_path).unwrap();
    let marker = ast::with_program(&page_path, &page_source, |program, _| {
        super::super::cross_file::defer_imported_values_from_visible(
            "getSelector",
            program,
            &page_path,
            &visible,
        )
    })
    .unwrap()
    .into_iter()
    .next()
    .expect("visible import produces a deferred marker");

    let selectors_source = std::fs::read_to_string(&selectors_path).unwrap();
    let static_values = ast::with_program(&selectors_path, &selectors_source, |program, _| {
        super::super::cross_file::collect_static_export_values(program)
    })
    .unwrap();
    assert_eq!(
        static_values.values("getSelector", false),
        &["imported-fn-val".to_string()]
    );
    assert!(static_values.values("missing", false).is_empty());

    let exports = std::collections::HashMap::from([(selectors_path, static_values)]);
    let resolved = super::super::cross_file::resolve_deferred_import(&marker, &exports)
        .expect("precollected named export resolves");
    assert_eq!(resolved, &["imported-fn-val".to_string()]);
    assert!(super::super::cross_file::resolve_deferred_import(
        "\0no-mistakes-playwright-import:not-json",
        &exports,
    )
    .is_none());
    assert!(super::super::cross_file::resolve_deferred_import(
        &marker,
        &std::collections::HashMap::new(),
    )
    .expect("a valid marker with no matching export resolves empty")
    .is_empty());
}

#[test]
fn visible_collector_resolves_imported_function_and_object_values() {
    let page_path = crate::codebase::ts_resolver::normalize_path(&fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]));
    let selectors_path = page_path.with_file_name("selectors.ts");
    let visible = std::collections::HashSet::from([page_path.clone(), selectors_path]);
    let source = std::fs::read_to_string(&page_path).unwrap();

    let collected = ast::with_program(&page_path, &source, |program, source| {
        super::collect_dynamic_identifier_values_with_file_from_visible(
            program, source, &page_path, &visible,
        )
    })
    .unwrap();
    let values = collected
        .into_iter()
        .flat_map(|entry| entry.values)
        .collect::<std::collections::HashSet<_>>();

    assert!(values.contains("imported-fn-val"));
    assert!(values.contains("imported-obj-a"));
    assert!(values.contains("imported-obj-b"));
}

#[test]
fn deferred_collector_preserves_imports_local_returns_and_direct_values() {
    let imported_path = crate::codebase::ts_resolver::normalize_path(&fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]));
    let selectors_path = imported_path.with_file_name("selectors.ts");
    let visible = std::collections::HashSet::from([imported_path.clone(), selectors_path]);
    let source = std::fs::read_to_string(&imported_path).unwrap();
    let imported = ast::with_program(&imported_path, &source, |program, source| {
        super::collect_dynamic_identifier_values_with_file_from_visible_deferred(
            program,
            source,
            &imported_path,
            &visible,
        )
    })
    .unwrap();
    let imported_values = imported
        .iter()
        .flat_map(|entry| entry.values.iter())
        .collect::<Vec<_>>();
    assert_eq!(imported_values.len(), 2);
    assert!(imported_values
        .iter()
        .all(|value| value.starts_with("\0no-mistakes-playwright-import:")));

    let local_path = fixture_path(&["ast-snippets", "selectors", "dynamic-function-return.tsx"]);
    let local_source = std::fs::read_to_string(&local_path).unwrap();
    let local = ast::with_program(&local_path, &local_source, |program, source| {
        super::collect_dynamic_identifier_values_with_file_from_visible_deferred(
            program,
            source,
            &local_path,
            &std::collections::HashSet::new(),
        )
    })
    .unwrap();
    assert_eq!(local[0].values, ["fn-a", "fn-b"]);

    let literal_path = fixture_path(&["ast-snippets", "selectors", "dynamic-ternary.tsx"]);
    let literal_source = std::fs::read_to_string(&literal_path).unwrap();
    let literal = ast::with_program(&literal_path, &literal_source, |program, source| {
        super::collect_dynamic_identifier_values_with_file_from_visible_deferred(
            program,
            source,
            &literal_path,
            &std::collections::HashSet::new(),
        )
    })
    .unwrap();
    assert_eq!(literal[0].values, ["option-a", "option-b"]);
}

#[test]
fn static_export_collection_covers_default_and_destructured_declarations() {
    let root = page_extras_path()
        .parent()
        .expect("saved fixture has a parent")
        .to_path_buf();
    let default_path = root.join("default-obj.ts");
    let default_source = std::fs::read_to_string(&default_path).unwrap();
    let default_values = ast::with_program(&default_path, &default_source, |program, _| {
        super::super::cross_file::collect_static_export_values(program)
    })
    .unwrap();
    assert_eq!(
        default_values.values("ignored", true),
        &["obj-a-val".to_string(), "obj-b-val".to_string()]
    );

    let extras_path = root.join("extras.ts");
    let extras_source = std::fs::read_to_string(&extras_path).unwrap();
    let extras = ast::with_program(&extras_path, &extras_source, |program, _| {
        super::super::cross_file::collect_static_export_values(program)
    })
    .unwrap();
    assert!(extras.values("firstArr", false).is_empty());
    assert_eq!(
        extras.values("fnBlockBody", false),
        &["block-body-val".to_string()]
    );
}

#[test]
fn visible_cross_file_resolution_handles_missing_bindings_and_unreadable_targets() {
    let page_path = Path::new("/repo/page.tsx");
    // The visible universe can identify an import even when its saved source
    // is unavailable; selector analysis must degrade to no static values.
    let visible = std::collections::HashSet::from([Path::new("/repo/selectors.ts").to_path_buf()]);
    let source = "import { value } from './selectors';";
    ast::with_program(page_path, source, |program, _| {
        assert!(
            super::super::cross_file::defer_imported_values_from_visible(
                "missing", program, page_path, &visible,
            )
            .is_empty()
        );
        assert!(
            super::super::cross_file::resolve_imported_values_from_visible(
                "value", program, page_path, &visible,
            )
            .is_empty()
        );
    })
    .unwrap();
}

#[test]
fn cross_file_resolves_default_fn_export_with_if_else_body() {
    let mut values = resolve_from_extras("DefaultFn");
    values.sort();
    assert_eq!(values, vec!["default-fn-else", "default-fn-val"]);
}

#[test]
fn cross_file_resolves_default_obj_export() {
    let mut values = resolve_from_extras("DefaultObj");
    values.sort();
    assert_eq!(values, vec!["obj-a-val", "obj-b-val"]);
}

#[test]
fn cross_file_resolves_default_arrow_expression_body() {
    let values = resolve_from_extras("DefaultArrowExpr");
    assert_eq!(values, vec!["arrow-expr-val"]);
}

#[test]
fn cross_file_resolves_default_arrow_block_body() {
    let values = resolve_from_extras("DefaultArrowBlock");
    assert_eq!(values, vec!["arrow-block-val"]);
}

#[test]
fn cross_file_default_literal_export_returns_empty() {
    let values = resolve_from_extras("DefaultLiteral");
    assert!(values.is_empty());
}

#[test]
fn cross_file_namespace_import_returns_empty() {
    let values = resolve_from_extras("Ns");
    assert!(values.is_empty());
}

#[test]
fn cross_file_fn_with_expr_statement_consequent() {
    let values = resolve_from_extras("fnWithExprConsequent");
    assert_eq!(values, vec!["expr-base-val"]);
}

#[test]
fn cross_file_uninitialized_export_let_returns_empty() {
    let values = resolve_from_extras("uninitializedLet");
    assert!(values.is_empty());
}

#[test]
fn cross_file_array_destructured_export_returns_empty() {
    let values = resolve_from_extras("firstArr");
    assert!(values.is_empty());
}

#[test]
fn cross_file_unresolvable_relative_import_returns_empty() {
    let source = r#"import { x } from './definitely-does-not-exist-xyz';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::super::cross_file::resolve_imported_values("x", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn visitor_collect_from_statements_with_no_scope_returns_empty() {
    let source = r#"const x = 'hello';"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let mut visitor = super::super::visitor::DynamicValuesVisitor::new();
        visitor.collect_from_statements(&program.body);
        assert!(visitor.collected.is_empty());
    })
    .unwrap();
}

#[test]
fn cross_file_default_obj_computed_skips_computed_properties() {
    let values = resolve_from_extras("DefaultObjComputed");
    assert_eq!(values, vec!["static-val"]);
}

#[test]
fn cross_file_fn_with_bare_return_in_if_consequent() {
    let values = resolve_from_extras("fnBareReturn");
    assert!(values.contains(&"bare-return-val".to_string()));
}

#[test]
fn cross_file_fn_with_block_statement_body() {
    let values = resolve_from_extras("fnBlockBody");
    assert_eq!(values, vec!["block-body-val"]);
}

#[test]
fn visitor_else_branch_adds_new_variable_to_all_names() {
    let source = r#"
function Page() {
  let a, b;
  if (cond) {
    a = 'val-a';
  } else {
    b = 'val-b';
  }
}
"#;
    let collected = parse_and_collect(source);
    let a_entry = collected.iter().find(|e| e.name == "a").expect("a entry");
    assert_eq!(a_entry.values, vec!["val-a"]);
    let b_entry = collected.iter().find(|e| e.name == "b").expect("b entry");
    assert_eq!(b_entry.values, vec!["val-b"]);
}
