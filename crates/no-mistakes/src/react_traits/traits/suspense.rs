mod dynamic_names;

#[cfg(test)]
pub(crate) use dynamic_names::collect_dynamic_names;
pub(crate) use dynamic_names::{
    collect_dynamic_names_for_spans, is_dynamic_or_lazy_call, is_dynamic_or_lazy_call_by_callee,
};

use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, JSXElementName, JSXMemberExpressionObject, Program,
    Statement,
};
use oxc_span::Span;
use std::collections::HashSet;

#[cfg(test)]
use oxc_ast_visit::{walk, Visit};

#[cfg(test)]
struct SuspenseVisitor<'a> {
    has_suspense: bool,
    span: Span,
    dynamic_names: &'a HashSet<String>,
}

#[cfg(test)]
fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

pub(crate) fn jsx_opening_is_suspense(
    elem: &oxc_ast::ast::JSXOpeningElement<'_>,
    dynamic_names: &HashSet<String>,
) -> bool {
    match &elem.name {
        JSXElementName::IdentifierReference(id) => {
            id.name == "Suspense" || dynamic_names.contains(id.name.as_ref())
        }
        JSXElementName::MemberExpression(m) => {
            m.property.name == "Suspense"
                && matches!(
                    &m.object,
                    JSXMemberExpressionObject::IdentifierReference(obj) if obj.name == "React"
                )
        }
        _ => false,
    }
}

#[cfg(test)]
impl<'a> Visit<'a> for SuspenseVisitor<'a> {
    fn visit_jsx_opening_element(&mut self, elem: &oxc_ast::ast::JSXOpeningElement<'a>) {
        if !within(elem.span, self.span) {
            walk::walk_jsx_opening_element(self, elem);
            return;
        }
        if jsx_opening_is_suspense(elem, self.dynamic_names) {
            self.has_suspense = true;
        }
        walk::walk_jsx_opening_element(self, elem);
    }
}

fn overlaps(a: Span, b: Span) -> bool {
    a.start < b.end && a.end > b.start
}

pub(crate) fn is_component_direct_lazy(program: &Program<'_>, span: Span) -> bool {
    for stmt in &program.body {
        match stmt {
            Statement::ExportDefaultDeclaration(e) if overlaps(e.span, span) => {
                if let ExportDefaultDeclarationKind::CallExpression(call) = &e.declaration {
                    if is_dynamic_or_lazy_call_by_callee(&call.callee) {
                        return true;
                    }
                }
            }
            Statement::ExportDeclaration(e) => {
                if let Declaration::VariableDeclaration(v) = &e.declaration {
                    for d in &v.declarations {
                        if overlaps(d.span, span) {
                            if let Some(init) = &d.init {
                                if is_dynamic_or_lazy_call(init) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            Statement::VariableDeclaration(v) => {
                for d in &v.declarations {
                    if overlaps(d.span, span) {
                        if let Some(init) = &d.init {
                            if is_dynamic_or_lazy_call(init) {
                                return true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
pub(crate) fn detect_uses_suspense(program: &Program<'_>, span: Span) -> bool {
    if is_component_direct_lazy(program, span) {
        return true;
    }
    let dynamic_names = collect_dynamic_names(program, span);
    let mut visitor = SuspenseVisitor {
        has_suspense: false,
        span,
        dynamic_names: &dynamic_names,
    };
    visitor.visit_program(program);
    visitor.has_suspense
}

#[cfg(test)]
mod tests;
