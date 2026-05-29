use super::super::*;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

fn fixture_file(name: &str, file: &str) -> PathBuf {
    fixture(name).join(file)
}

fn required_string_or_array(
    expression: &Expression<'_>,
    source: &str,
    name: &str,
) -> anyhow::Result<Vec<String>> {
    if let Some(value) = test_config::shared::optional_string(expression, source) {
        return Ok(vec![value]);
    }
    let Expression::ArrayExpression(array) = parenthesized_expression(expression) else {
        anyhow::bail!("expected string literal or string array for {name}");
    };
    let mut values = Vec::new();
    for element in &array.elements {
        match element {
            ArrayExpressionElement::StringLiteral(literal) => {
                values.push(literal.value.to_string())
            }
            ArrayExpressionElement::TemplateLiteral(template)
                if template.expressions.is_empty() =>
            {
                values.push(crate::ast::template_literal_text(template, source));
            }
            _ => anyhow::bail!("expected string literal array entries for {name}"),
        }
    }
    if values.is_empty() {
        anyhow::bail!("expected string literal or string array for {name}");
    }
    Ok(values)
}

fn parenthesized_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression(&parenthesized.expression)
        }
        _ => expression,
    }
}

#[test]
fn config_parsers_reject_invalid_literals() {
    let root = fixture("coverage");
    let tsconfig = tsconfig_without_config(&root);
    let pw_path = root.join("playwright.invalid.ts");
    let pw_source = std::fs::read_to_string(&pw_path).unwrap();
    let pw_err =
        match test_config::playwright::parse_from_path(&pw_source, &pw_path, &root, &tsconfig) {
            Ok(_) => panic!("expected invalid Playwright config to fail"),
            Err(err) => err,
        };
    assert!(pw_err.to_string().contains("expected string literal"));
    let empty_match_path = root.join("playwright.empty-match-invalid.ts");
    let empty_match_source = std::fs::read_to_string(&empty_match_path).unwrap();
    let empty_match_err = match test_config::playwright::parse_from_path(
        &empty_match_source,
        &empty_match_path,
        &root,
        &tsconfig,
    ) {
        Ok(_) => panic!("expected empty testMatch to fail"),
        Err(error) => error,
    };
    assert!(empty_match_err
        .to_string()
        .contains("expected string literal or string array"));

    let vitest_path = root.join("vitest.invalid.mts");
    let vitest_source = std::fs::read_to_string(&vitest_path).unwrap();
    let vitest_err =
        test_config::vitest::parse_from_path(&vitest_source, &vitest_path, &root, &root, &tsconfig)
            .unwrap_err();
    assert!(vitest_err
        .to_string()
        .contains("expected string literal array entries"));
}

#[test]
fn shared_config_helpers_cover_ast_edge_shapes() {
    let path = fixture_file("coverage", "parser-helpers.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, source| {
        let bindings = test_config::shared::top_level_object_bindings(program);
        assert!(bindings.contains_key("nested"));
        assert!(!bindings.contains_key("noInit"));
        assert!(!bindings.contains_key("destructured"));

        let object = test_config::shared::default_export_object(program, &bindings).unwrap();
        assert_eq!(
            test_config::shared::property_expression(object, "name")
                .and_then(|expr| test_config::shared::optional_string(expr, source))
                .as_deref(),
            Some("nested")
        );

        let fixture_object = test_config::shared::property_object(object, "missing", &bindings);
        assert!(fixture_object.is_none());
        let oxc_ast::ast::Expression::ObjectExpression(object) = bindings.get("object").unwrap()
        else {
            panic!("expected object binding");
        };
        assert_eq!(
            test_config::shared::property_expression(object, "name")
                .map(|expr| test_config::shared::required_string(expr, source, "name").unwrap())
                .as_deref(),
            Some("literal")
        );
        assert!(test_config::shared::property_expression(object, "computed").is_none());
        assert!(test_config::shared::property_expression(object, "quoted").is_some());

        let list = test_config::shared::property_expression(object, "list").unwrap();
        assert_eq!(
            required_string_or_array(list, source, "list").unwrap(),
            vec!["one".to_string(), "two".to_string()]
        );
        let name = test_config::shared::property_expression(object, "name").unwrap();
        assert_eq!(
            required_string_or_array(name, source, "name").unwrap(),
            vec!["literal".to_string()]
        );
        let wrapped_list = test_config::shared::property_expression(object, "wrappedList").unwrap();
        assert_eq!(
            required_string_or_array(wrapped_list, source, "wrappedList").unwrap(),
            vec!["three".to_string()]
        );
        let non_array = test_config::shared::property_expression(object, "nonArray").unwrap();
        assert!(required_string_or_array(non_array, source, "nonArray").is_err());
        let bad_list = test_config::shared::property_expression(object, "badList").unwrap();
        assert!(required_string_or_array(bad_list, source, "badList").is_err());
        let empty_list = test_config::shared::property_expression(object, "emptyList").unwrap();
        assert!(required_string_or_array(empty_list, source, "emptyList").is_err());
        assert!(
            test_config::shared::inferred_string_or_array(non_array, source, "nonArray").is_err()
        );
        let spread_list = test_config::shared::property_expression(object, "spreadList").unwrap();
        assert_eq!(
            test_config::shared::inferred_string_or_array(spread_list, source, "spreadList")
                .unwrap(),
            vec!["one".to_string(), "two".to_string()]
        );
        let wrapped_spread_list =
            test_config::shared::property_expression(object, "wrappedSpreadList").unwrap();
        assert_eq!(
            test_config::shared::inferred_string_or_array(
                wrapped_spread_list,
                source,
                "wrappedSpreadList"
            )
            .unwrap(),
            vec!["four".to_string()]
        );
    })
    .unwrap();

    let path = fixture_file("coverage", "parser-edge.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let bindings = test_config::shared::top_level_object_bindings(program);
        assert!(test_config::shared::default_export_object(program, &bindings).is_none());
        let oxc_ast::ast::Expression::ObjectExpression(object) = bindings.get("object").unwrap()
        else {
            panic!("expected object binding");
        };
        assert!(test_config::shared::property_expression(object, "quoted").is_some());
        assert!(test_config::shared::property_object(object, "cyclic", &bindings).is_none());
    })
    .unwrap();

    let path = fixture_file("coverage", "parser-cycle.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let bindings = test_config::shared::top_level_object_bindings(program);
        assert!(test_config::shared::default_export_object(program, &bindings).is_none());
    })
    .unwrap();

    let path = fixture_file("coverage", "playwright.call-invalid.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let bindings = test_config::shared::top_level_object_bindings(program);
        assert!(test_config::shared::default_export_object(program, &bindings).is_none());
    })
    .unwrap();
}
