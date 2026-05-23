use crate::fetches::report::types::CacheKind;
use no_mistakes::fetch::visitor::{FetchScope, FetchVisitor};
use oxc_ast_visit::Visit;
use std::collections::HashSet;

#[test]
fn test_visitor_non_fetch() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "notFetch();";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}

#[test]
fn test_visitor_complex_variants() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch(url, options);
            fetch(url, { notMethod: 'POST' });
            fetch(url, { method: methodVar });
            fetch(url, { ...spread });
            fetch(url, { [dynamic]: 'POST' });
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 5);
    for fetch in &visitor.fetches {
        assert_eq!(fetch.method, "GET");
        assert!(fetch.dynamic);
        assert!(fetch.line > 0);
    }
}

#[test]
fn test_visitor_arrow_function_expression() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            const run = () => {
                fetch('/api/arrow');
            };
            run();
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 1);
    assert_eq!(visitor.fetches[0].path, "/api/arrow");
}

#[test]
fn test_visitor_cache_options_are_extracted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch('/api/cache', { cache: 'force-cache' });
            fetch('/api/next', { next: { revalidate: 60 }});
            fetch('/api/next-zero', { next: { revalidate: 0 }});
            fetch('/api/tags', { next: { tags: ['a', 'b'] }});
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 4);
    assert!(visitor.fetches[0].cached);
    assert_eq!(visitor.fetches[0].cache_kind, CacheKind::FetchCache);
    assert!(visitor.fetches[1].cached);
    assert_eq!(
        visitor.fetches[1].cache_kind,
        CacheKind::FetchNextRevalidate
    );
    assert!(!visitor.fetches[2].cached);
    assert_eq!(visitor.fetches[2].cache_kind, CacheKind::None);
    assert!(visitor.fetches[3].cached);
    assert_eq!(visitor.fetches[3].cache_kind, CacheKind::FetchNextTags);
}

#[test]
fn test_visitor_cache_options_unknown_flags_are_ignored() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch('/api/no-store', { cache: 'no-store' });
            fetch('/api/not-object', { next: ['revalidate'] });
            fetch('/api/next-unknown', { next: { unknown: true, ...tags }});
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 3);
    for fetch in &visitor.fetches {
        assert!(!fetch.cached);
    }
    assert_eq!(visitor.fetches[0].cache_kind, CacheKind::None);
    assert_eq!(visitor.fetches[1].cache_kind, CacheKind::None);
    assert_eq!(visitor.fetches[2].cache_kind, CacheKind::None);
}

#[test]
fn test_visitor_shadowed_fetch_is_not_counted() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            fetch('/api/outer');
            {
                const fetch = () => {};
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
fn test_mark_identifier_shadowed_in_var_scope_falls_back_without_var_scope() {
    let mut visitor = FetchVisitor::new("fetch('/api/outer')", "test.ts", false, false);
    visitor.fetch_scope_stack = vec![FetchScope {
        shadowed_identifiers: HashSet::new(),
        tracks_var_bindings: false,
    }];
    visitor.mark_identifier_shadowed_in_var_scope("fetch");
    assert!(visitor
        .fetch_scope_stack
        .last()
        .unwrap()
        .shadowed_identifiers
        .contains("fetch"));
}

#[test]
fn test_visitor_function_declaration_shadows_fetch() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            function fetch() {}
            fetch('/api/function');
        ";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}

#[test]
fn test_visitor_ts_declare_function_shadows_fetch() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "declare function fetch(): void;";
    let source_type = oxc_span::SourceType::from_path(std::path::Path::new("test.ts")).unwrap();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, "test.ts", false, false);
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.fetches.len(), 0);
}
