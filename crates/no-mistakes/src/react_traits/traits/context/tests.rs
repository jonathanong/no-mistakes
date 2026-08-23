use super::jsx_is_context_provider;
use crate::ast;
use oxc_ast::ast::Program;
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::path::PathBuf;

struct ContextVisitor {
    has_provider: bool,
    span: Span,
}

fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

impl<'a> Visit<'a> for ContextVisitor {
    fn visit_jsx_opening_element(&mut self, elem: &oxc_ast::ast::JSXOpeningElement<'a>) {
        if !within(elem.span, self.span) {
            return;
        }
        if jsx_is_context_provider(elem) {
            self.has_provider = true;
        }
        walk::walk_jsx_opening_element(self, elem);
    }
}

fn detect_context_provider(program: &Program<'_>, span: Span) -> bool {
    let mut visitor = ContextVisitor {
        has_provider: false,
        span,
    };
    visitor.visit_program(program);
    visitor.has_provider
}

fn fixture_source(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-analyze/context/fixture")
        .join(name)
        .join("test.tsx");
    let source = std::fs::read_to_string(&path).expect("fixture must be readable");
    (path, source)
}

fn check(name: &str) -> bool {
    let (path, source) = fixture_source(name);
    let span = oxc_span::Span::new(0, source.len() as u32);
    ast::with_program(&path, &source, |program, _| {
        detect_context_provider(program, span)
    })
    .unwrap()
}

#[test]
fn detects_context_provider() {
    assert!(check("with-provider"));
}

#[test]
fn no_context_provider() {
    assert!(!check("without-provider"));
}

#[test]
fn detects_standalone_provider_tag() {
    assert!(check("standalone-provider"));
}

#[test]
fn provider_outside_span_not_detected() {
    let (path, source) = fixture_source("with-provider");
    let result = ast::with_program(&path, &source, |program, _| {
        detect_context_provider(program, oxc_span::Span::new(0, 0))
    })
    .unwrap();
    assert!(!result);
}
