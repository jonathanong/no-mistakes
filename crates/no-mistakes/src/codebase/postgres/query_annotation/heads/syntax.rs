use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, VariableDeclaration};
use oxc_ast_visit::{walk, Visit};

pub(super) struct Call {
    pub span: Span,
    pub head: Head,
}
#[derive(Default)]
pub(super) struct Syntax {
    pub calls: Vec<Call>,
    pub constants: FxHashMap<Span, Head>,
}
fn span(span: oxc_span::Span) -> Span {
    (span.start, span.end)
}
fn named(text: &str) -> bool {
    // Inspect only the FIRST block comment; a later named comment cannot repair
    // an empty first one. Decoded/cooked text determines runtime string bytes.
    text.trim_start()
        .strip_prefix("/*")
        .and_then(|tail| tail.find("*/").map(|end| !tail[..end].trim().is_empty()))
        .unwrap_or(false)
}
fn literal<'a>(mut expression: &'a Expression<'a>) -> bool {
    loop {
        expression = unwrap_ts_wrappers(expression);
        match expression {
            Expression::StringLiteral(value) => return named(&value.value),
            Expression::TemplateLiteral(value) => {
                return value
                    .quasis
                    .first()
                    .and_then(|part| part.value.cooked.as_ref())
                    .is_some_and(|text| named(text))
            }
            Expression::BinaryExpression(value)
                if value.operator == oxc_ast::ast::BinaryOperator::Addition =>
            {
                expression = &value.left;
            }
            _ => return false,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) enum Head {
    Literal,
    Tag(Span),
    Reference(Span),
    Unproven,
}
impl Head {
    pub fn reference(self) -> Option<Span> {
        match self {
            Self::Tag(span) | Self::Reference(span) => Some(span),
            _ => None,
        }
    }
}
fn head(expression: &Expression<'_>) -> Head {
    let expression = unwrap_ts_wrappers(expression);
    if literal(expression) {
        return Head::Literal;
    }
    if let Expression::Identifier(id) = expression {
        return Head::Reference(span(id.span));
    }
    if let Expression::TaggedTemplateExpression(value) = expression {
        if value
            .quasi
            .quasis
            .first()
            .and_then(|part| part.value.cooked.as_ref())
            .is_some_and(|text| named(text))
        {
            if let Expression::Identifier(tag) = unwrap_ts_wrappers(&value.tag) {
                return Head::Tag(span(tag.span));
            }
        }
    }
    Head::Unproven
}
impl<'a> Visit<'a> for Syntax {
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        self.calls.push(Call {
            span: span(value.span),
            head: value
                .arguments
                .first()
                .and_then(|arg| arg.as_expression())
                .map_or(Head::Unproven, head),
        });
        walk::walk_call_expression(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                    if matches!(head, Head::Literal | Head::Tag(_)) {
                        self.constants.insert(span(binding.span), head);
                    }
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
pub(super) fn collect(program: &Program<'_>) -> Syntax {
    let mut syntax = Syntax::default();
    syntax.visit_program(program);
    syntax
}
