use oxc_ast::ast::JSXElementName;

#[cfg(test)]
use oxc_ast::ast::Program;
#[cfg(test)]
use oxc_ast_visit::{walk, Visit};
#[cfg(test)]
use oxc_span::Span;

#[cfg(test)]
struct ContextVisitor {
    has_provider: bool,
    span: Span,
}

#[cfg(test)]
fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

pub(crate) fn jsx_is_context_provider(elem: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
    match &elem.name {
        JSXElementName::MemberExpression(m) if m.property.name == "Provider" => true,
        JSXElementName::IdentifierReference(id) if id.name == "Provider" => true,
        _ => false,
    }
}

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn detect_context_provider(program: &Program<'_>, span: Span) -> bool {
    let mut visitor = ContextVisitor {
        has_provider: false,
        span,
    };
    visitor.visit_program(program);
    visitor.has_provider
}

#[cfg(test)]
mod tests;
