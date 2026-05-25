use no_mistakes::fetch::types::SourceType;
use no_mistakes::fetch::visitor::FetchVisitor;
use oxc_ast_visit::Visit;

fn parse_and_visit(source: &str, file: &str) -> Vec<no_mistakes::fetch::types::FetchOccurrence> {
    let allocator = oxc_allocator::Allocator::default();
    let source_type =
        oxc_span::SourceType::from_path(std::path::Path::new(file)).unwrap_or_default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let mut visitor = FetchVisitor::new(source, file, false, false);
    visitor.visit_program(&parsed.program);
    visitor.fetches
}

#[test]
fn function_name_from_named_function() {
    let fetches = parse_and_visit("function getData() { fetch('/api/data'); }", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].function_name, Some("getData".to_string()));
}

#[test]
fn function_name_from_arrow_function() {
    let fetches = parse_and_visit(
        "const getUsers = async () => { fetch('/api/users'); };",
        "test.ts",
    );
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].function_name, Some("getUsers".to_string()));
}

#[test]
fn function_name_none_at_top_level() {
    let fetches = parse_and_visit("fetch('/api/data');", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].function_name, None);
}

#[test]
fn function_name_nearest_named_ancestor() {
    let fetches = parse_and_visit(
        "function outer() { (() => { fetch('/api/data'); })(); }",
        "test.ts",
    );
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].function_name, Some("outer".to_string()));
}

#[test]
fn function_name_from_function_expression() {
    let fetches = parse_and_visit(
        "const handler = function() { fetch('/api/data'); };",
        "test.ts",
    );
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].function_name, Some("handler".to_string()));
}

#[test]
fn conditional_from_if_statement() {
    let fetches = parse_and_visit(
        "if (true) { fetch('/api/cond'); } fetch('/api/uncond');",
        "test.ts",
    );
    assert_eq!(fetches.len(), 2);
    let cond = fetches.iter().find(|f| f.path == "/api/cond").unwrap();
    let uncond = fetches.iter().find(|f| f.path == "/api/uncond").unwrap();
    assert!(cond.conditional);
    assert!(!uncond.conditional);
}

#[test]
fn conditional_from_ternary() {
    let fetches = parse_and_visit("true ? fetch('/api/a') : fetch('/api/b');", "test.ts");
    assert_eq!(fetches.len(), 2);
    assert!(fetches[0].conditional);
    assert!(fetches[1].conditional);
}

#[test]
fn conditional_from_logical_and() {
    let fetches = parse_and_visit("true && fetch('/api/right');", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert!(fetches[0].conditional);
}

#[test]
fn conditional_from_logical_or() {
    let fetches = parse_and_visit("false || fetch('/api/right');", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert!(fetches[0].conditional);
}

#[test]
fn not_conditional_in_if_test() {
    let fetches = parse_and_visit("if (fetch('/api/test')) { }", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert!(!fetches[0].conditional);
}

#[test]
fn in_promise_all() {
    let fetches = parse_and_visit(
        "Promise.all([fetch('/api/a'), fetch('/api/b')]); fetch('/api/c');",
        "test.ts",
    );
    assert_eq!(fetches.len(), 3);
    let a = fetches.iter().find(|f| f.path == "/api/a").unwrap();
    let b = fetches.iter().find(|f| f.path == "/api/b").unwrap();
    let c = fetches.iter().find(|f| f.path == "/api/c").unwrap();
    assert!(a.in_promise_all);
    assert!(b.in_promise_all);
    assert!(!c.in_promise_all);
}

#[test]
fn in_promise_all_settled() {
    let fetches = parse_and_visit("Promise.allSettled([fetch('/api/a')]);", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert!(fetches[0].in_promise_all);
}

#[test]
fn error_handled_try_catch() {
    let fetches = parse_and_visit(
        "try { fetch('/api/handled'); } catch (e) {} fetch('/api/not');",
        "test.ts",
    );
    assert_eq!(fetches.len(), 2);
    let handled = fetches.iter().find(|f| f.path == "/api/handled").unwrap();
    let not = fetches.iter().find(|f| f.path == "/api/not").unwrap();
    assert!(handled.error_handled);
    assert!(!not.error_handled);
}

#[test]
fn not_error_handled_try_finally() {
    let fetches = parse_and_visit("try { fetch('/api/data'); } finally {}", "test.ts");
    assert_eq!(fetches.len(), 1);
    assert!(!fetches[0].error_handled);
}

#[test]
fn error_handled_does_not_leak_to_catch_body() {
    let fetches = parse_and_visit(
        "try { fetch('/api/try'); } catch (e) { fetch('/api/catch'); }",
        "test.ts",
    );
    assert_eq!(fetches.len(), 2);
    let try_fetch = fetches.iter().find(|f| f.path == "/api/try").unwrap();
    let catch_fetch = fetches.iter().find(|f| f.path == "/api/catch").unwrap();
    assert!(try_fetch.error_handled);
    assert!(!catch_fetch.error_handled);
}

#[test]
fn source_type_from_file_stem() {
    assert_eq!(SourceType::from_file_stem("app/page.tsx"), SourceType::Page);
    assert_eq!(
        SourceType::from_file_stem("app/layout.tsx"),
        SourceType::Layout
    );
    assert_eq!(
        SourceType::from_file_stem("app/loading.tsx"),
        SourceType::Loading
    );
    assert_eq!(
        SourceType::from_file_stem("app/error.tsx"),
        SourceType::Error
    );
    assert_eq!(
        SourceType::from_file_stem("app/not-found.tsx"),
        SourceType::Error
    );
    assert_eq!(
        SourceType::from_file_stem("app/template.tsx"),
        SourceType::Template
    );
    assert_eq!(
        SourceType::from_file_stem("app/api/route.ts"),
        SourceType::Route
    );
    assert_eq!(
        SourceType::from_file_stem("lib/utils.ts"),
        SourceType::Module
    );
}

#[test]
fn combined_dimensions() {
    let fetches = parse_and_visit(
        "try { if (true) { Promise.all([fetch('/api/all')]); } } catch (e) {}",
        "test.ts",
    );
    assert_eq!(fetches.len(), 1);
    assert!(fetches[0].conditional);
    assert!(fetches[0].in_promise_all);
    assert!(fetches[0].error_handled);
}
