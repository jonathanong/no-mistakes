use super::*;
use crate::ast::with_program;
use oxc_ast::ast::{CallExpression, Expression, Statement};
use std::path::Path;

fn with_call_expression<F>(source: &str, f: F)
where
    F: FnOnce(&CallExpression<'_>),
{
    crate::ast::with_program(Path::new("test.ts"), source, |program, _| {
        let stmt = program.body.first().expect("Expected statement");
        let Statement::ExpressionStatement(expr_stmt) = stmt else {
            panic!("Expected expression statement");
        };
        let Expression::CallExpression(call_expr) = &expr_stmt.expression else {
            panic!("Expected call expression");
        };
        f(call_expr);
    })
    .unwrap();
}

#[test]
fn test_cache_wrapper_name_react_cache() {
    with_call_expression("cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_some());
        let (name, kind) = result.unwrap();
        assert_eq!(name, "cache");
        assert_eq!(kind, CacheKind::ReactCache);
    });
}

#[test]
fn test_cache_wrapper_name_unstable_cache() {
    with_call_expression("unstable_cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_some());
        let (name, kind) = result.unwrap();
        assert_eq!(name, "unstable_cache");
        assert_eq!(kind, CacheKind::UnstableCache);
    });
}

#[test]
fn test_cache_wrapper_name_other_function() {
    with_call_expression("other_function(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_iife() {
    with_call_expression("(() => {})(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_parenthesized_identifier() {
    with_call_expression("(cache)(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_nested_call() {
    with_call_expression("cache()()", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_member_expression() {
    with_call_expression("obj.cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

fn extract_from_source(source: &str) -> (bool, CacheKind) {
    let mut result = None;
    with_program(Path::new("test.ts"), source, |program, _| {
        for stmt in &program.body {
            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                    if let Some(arg) = call_expr.arguments.get(1) {
                        if let Some(Expression::ObjectExpression(obj)) = arg.as_expression() {
                            result = Some(extract_fetch_cache_options(obj));
                            return;
                        }
                    }
                }
            }
        }
    })
    .unwrap();

    result.expect("expected fetch(...) call with second argument object")
}

#[test]
fn test_cache_force_cache() {
    let source = "fetch('url', { cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_next_revalidate() {
    let source = "fetch('url', { next: { revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_next_tags() {
    let source = "fetch('url', { next: { tags: [] } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextTags);
}

#[test]
fn test_empty_options() {
    let source = "fetch('url', {});";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_extract_fetch_cache_options_direct_ast_construction() {
    let allocator = oxc_allocator::Allocator::default();

    // Construct a dummy SpreadElement
    let spread = oxc_ast::ast::SpreadElement {
        node_id: std::cell::Cell::new(oxc_syntax::node::NodeId::new(0)),
        span: oxc_span::Span::default(),
        argument: Expression::Identifier(oxc_allocator::Box::new_in(
            oxc_ast::ast::IdentifierReference {
                node_id: std::cell::Cell::new(oxc_syntax::node::NodeId::new(0)),
                span: oxc_span::Span::default(),
                name: "spread".into(),
                reference_id: std::cell::Cell::new(None),
            },
            &allocator,
        )),
    };

    let properties = oxc_allocator::Vec::from_iter_in(
        vec![oxc_ast::ast::ObjectPropertyKind::SpreadProperty(
            oxc_allocator::Box::new_in(spread, &allocator),
        )],
        &allocator,
    );

    let obj = oxc_ast::ast::ObjectExpression {
        node_id: std::cell::Cell::new(oxc_syntax::node::NodeId::new(0)),
        span: oxc_span::Span::default(),
        properties,
    };

    let (cached, kind) = extract_fetch_cache_options(&obj);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_spread_property_with_valid_cache() {
    let source = "fetch('url', { ...spread, cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_dynamic_key_with_valid_cache() {
    let source = "fetch('url', { [dynamic]: 'value', cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_spread_property_in_next_with_valid_revalidate() {
    let source = "fetch('url', { next: { ...spread, revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_dynamic_key_in_next_with_valid_revalidate() {
    let source = "fetch('url', { next: { [dynamic]: 'value', revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_method_property() {
    let source = "fetch('url', { method() {}, cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_method_property_in_next() {
    let source = "fetch('url', { next: { method() {}, revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_other_options() {
    let source = "fetch('url', { method: 'POST', cache: 'no-store' });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_revalidate_zero() {
    let source = "fetch('url', { next: { revalidate: 0 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_empty() {
    let source = "fetch('url', { next: {} });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_spread_options() {
    let source = "fetch('url', { ...spreadOpts });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_dynamic_cache_key_is_static_string() {
    let source = "fetch('url', { ['cache']: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_cache_not_string() {
    let source = "fetch('url', { cache: true });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_cache_unknown_mode() {
    let source = "fetch('url', { cache: 'unknown-mode' });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_not_object() {
    let source = "fetch('url', { next: true });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_is_null() {
    let source = "fetch('url', { next: null });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_revalidate_string_value() {
    let source = "fetch('url', { next: { revalidate: '60' } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_tags_non_array() {
    let source = "fetch('url', { next: { tags: 'foo' } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_spread() {
    let source = "fetch('url', { next: { ...spread } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_dynamic_key_is_static_string() {
    let source = "fetch('url', { next: { ['revalidate']: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_next_unrelated_key() {
    let source = "fetch('url', { next: { unrelated: 1 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_cache_and_next_properties() {
    let source = "fetch('url', { cache: 'force-cache', next: { revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_dynamic_cache_key_is_not_static() {
    let source = "fetch('url', { [dynamicVar]: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_dynamic_key_is_not_static() {
    let source = "fetch('url', { next: { [dynamicVar]: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_revalidate_negative() {
    let source = "fetch('url', { next: { revalidate: -1 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_revalidate_dynamic_expression() {
    let source = "fetch('url', { next: { revalidate: computedDelay } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_cache_dynamic_expression() {
    let source = "fetch('url', { cache: cacheMode });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_dynamic_cache_key_is_static_string_unrelated() {
    let source = "fetch('url', { ['unrelated']: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_next_dynamic_key_is_not_static_unrelated() {
    let source = "fetch('url', { next: { [dynamicVarUnrelated]: 'force-cache' } });";
    let (cached, kind) = extract_from_source(source);
    assert!(!cached);
    assert_eq!(kind, CacheKind::None);
}

#[test]
fn test_cache_spread_continues_to_next_property() {
    let source = "fetch('url', { ...spreadOpts, cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_cache_dynamic_key_continues_to_next_property() {
    let source = "fetch('url', { [dynamicVar]: 'no-store', cache: 'force-cache' });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchCache);
}

#[test]
fn test_next_spread_continues_to_next_property() {
    let source = "fetch('url', { next: { ...spreadOpts, revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_next_dynamic_key_continues_to_next_property() {
    let source = "fetch('url', { next: { [dynamicVar]: 'no-store', revalidate: 60 } });";
    let (cached, kind) = extract_from_source(source);
    assert!(cached);
    assert_eq!(kind, CacheKind::FetchNextRevalidate);
}

#[test]
fn test_cache_wrapper_name_case_sensitivity() {
    with_call_expression("CACHE(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
    with_call_expression("Unstable_Cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
    with_call_expression("unstableCache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_prefix_suffix() {
    with_call_expression("my_cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
    with_call_expression("unstable_cache_2(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}

#[test]
fn test_cache_wrapper_name_empty_or_special() {
    with_call_expression("_(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
    with_call_expression("$cache(fn)", |expr| {
        let result = cache_wrapper_name(expr);
        assert!(result.is_none());
    });
}
