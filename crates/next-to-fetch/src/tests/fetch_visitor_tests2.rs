use crate::report::types::CacheKind;
use no_mistakes_core::fetch::visitor::FetchVisitor;
use oxc_ast_visit::Visit;

#[test]
fn test_visitor_anonymous_function_expression_does_not_mark_fetch_shadowing() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            const f = function() {};
            fetch('/api/outer');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert_eq!(visitor.fetches[0].path, "/api/outer");
}

#[test]
fn test_visitor_var_shadowing_survives_blocks() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch('/api/outer');
            if (true) {
                var fetch = () => {};
            }
            fetch('/api/inner');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert_eq!(visitor.fetches[0].path, "/api/outer");
}

#[test]
fn test_visitor_shadowed_fetch_in_catch_clause_is_not_counted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch('/api/outer');
            try {
                throw new Error('boom');
            } catch (fetch) {
                fetch('/api/inner');
            }
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert_eq!(visitor.fetches[0].path, "/api/outer");
}

#[test]
fn test_visitor_imported_fetch_is_not_counted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            import { fetch } from './legacy-fetch';
            fetch('/api/imported');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}

#[test]
fn test_visitor_default_imported_fetch_is_not_counted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            import fetch from './legacy-fetch';
            fetch('/api/imported');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}

#[test]
fn test_visitor_namespace_imported_fetch_is_not_counted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            import * as fetch from './legacy-fetch';
            fetch('/api/imported');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}

#[test]
fn test_visitor_cache_wrappers_mark_fetch_calls() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            cache(fetch('/api/cached', { method: 'POST' }));
            unstable_cache(fetch('/api/unstable', { next: { revalidate: 60 } }));
            const getUsers = cache(fetch('/api/users', { method: 'PUT' }));
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 3);

    assert!(visitor.fetches[0].cached);
    assert_eq!(visitor.fetches[0].cache_kind, CacheKind::ReactCache);
    assert_eq!(visitor.fetches[0].cached_function.as_deref(), Some("cache"));
    assert_eq!(visitor.fetches[0].method, "POST");

    assert!(visitor.fetches[1].cached);
    assert_eq!(visitor.fetches[1].cache_kind, CacheKind::UnstableCache);
    assert_eq!(
        visitor.fetches[1].cached_function.as_deref(),
        Some("unstable_cache")
    );
    assert_eq!(visitor.fetches[1].method, "GET");

    assert!(visitor.fetches[2].cached);
    assert_eq!(visitor.fetches[2].cache_kind, CacheKind::ReactCache);
    assert_eq!(
        visitor.fetches[2].cached_function.as_deref(),
        Some("getUsers")
    );
    assert_eq!(visitor.fetches[2].method, "PUT");
}

#[test]
fn test_visitor_no_args() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "fetch();";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert_eq!(visitor.fetches[0].path, "unknown");
    assert!(visitor.fetches[0].dynamic);
    assert!(visitor.fetches[0].unsupported);
}

#[test]
fn test_visitor_dynamic_and_template() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "fetch(url); fetch(`/api/${id}`, { method: 'PATCH' }); fetch('/api/get');";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 3);
    assert_eq!(visitor.fetches[0].path, "dynamic");
    assert!(visitor.fetches[0].dynamic);
    assert!(visitor.fetches[0].rsc);
    assert_eq!(visitor.fetches[1].path, "/api/${id}");
    assert!(visitor.fetches[1].dynamic);
    assert_eq!(visitor.fetches[1].method, "PATCH");
    assert!(visitor.fetches[1].rsc);
    assert_eq!(visitor.fetches[2].path, "/api/get");
    assert!(!visitor.fetches[2].dynamic);
    assert_eq!(visitor.fetches[2].method, "GET");
    assert!(visitor.fetches[2].rsc);
}

#[test]
fn test_visitor_route_handler_fetches_are_non_rsc() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "fetch('/api/route');";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "app/api/route.ts", false, true);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert!(!visitor.fetches[0].rsc);
}
