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
fn literal(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::StringLiteral(value) => named(&value.value),
        Expression::TemplateLiteral(value) => value
            .quasis
            .first()
            .and_then(|part| part.value.cooked.as_ref())
            .is_some_and(|text| named(text)),
        _ => false,
    }
}
fn text<'a>(expression: &'a Expression<'a>) -> Option<(&'a str, bool, bool)> {
    match expression {
        Expression::StringLiteral(value) => Some((&value.value, false, false)),
        Expression::TemplateLiteral(value) => Some((
            value.quasis.first()?.value.cooked.as_ref()?,
            false,
            !value.expressions.is_empty(),
        )),
        Expression::TaggedTemplateExpression(value) => Some((
            value.quasi.quasis.first()?.value.cooked.as_ref()?,
            true,
            !value.quasi.expressions.is_empty(),
        )),
        _ => None,
    }
}
fn transaction(text: &str, can_continue: bool) -> bool {
    // Share max's Unicode boundary and leading-comment semantics. EOF after
    // BEGIN is not a stable boundary when an unknown suffix can make BEGINNING.
    crate::codebase::postgres::annotation::transaction_command_prefix_end(text)
        .is_some_and(|end| !can_continue || end < text.len())
}
fn uncertain_missing(text: &str, can_continue: bool) -> bool {
    let mut leading = text.trim_start();
    if can_continue && (leading.is_empty() || leading == "/") {
        return true;
    }
    if let Some(comment) = leading.strip_prefix("/*") {
        let Some(end) = comment.find("*/") else {
            return can_continue;
        };
        leading = comment[end + 2..].trim_start();
        if can_continue && leading.is_empty() {
            return true;
        }
    }
    can_continue
        && ["BEGIN", "COMMIT", "ROLLBACK"].iter().any(|command| {
            leading.len() <= command.len()
                && command
                    .get(..leading.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(leading))
        })
}
#[derive(Clone, Copy)]
pub(super) enum Head {
    Literal,
    Tag,
    Missing,
    MissingTag,
    Reference(Span),
    Unproven,
}
impl Head {
    pub fn reference(self) -> Option<Span> {
        match self {
            Self::Reference(span) => Some(span),
            _ => None,
        }
    }
}
// Preserve Tag through concatenation so local constants still require safe uses.
fn head<'a>(mut expression: &'a Expression<'a>) -> Head {
    let mut concatenated = false;
    loop {
        expression = unwrap_ts_wrappers(expression);
        if let Expression::BinaryExpression(value) = expression {
            if value.operator == oxc_ast::ast::BinaryOperator::Addition {
                concatenated = true;
                expression = &value.left;
                continue;
            }
        }
        break;
    }
    if literal(expression) {
        return Head::Literal;
    }
    if !concatenated {
        if let Expression::Identifier(id) = expression {
            return Head::Reference(span(id.span));
        }
    }
    if let Expression::TaggedTemplateExpression(value) = expression {
        if value
            .quasi
            .quasis
            .first()
            .and_then(|part| part.value.cooked.as_ref())
            .is_some_and(|text| named(text))
        {
            // Low effort judges the literal head, independently of the tag.
            return Head::Tag;
        }
    }
    if let Some((text, tagged, interpolated)) = text(expression) {
        let can_continue = interpolated || concatenated;
        if transaction(text, can_continue) {
            return if tagged { Head::Tag } else { Head::Literal };
        }
        if !uncertain_missing(text, can_continue) {
            return if tagged {
                Head::MissingTag
            } else {
                Head::Missing
            };
        }
    }
    Head::Unproven
}
impl<'a> Visit<'a> for Syntax {
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        let argument = value.arguments.first().and_then(|arg| arg.as_expression());
        let head = argument.map_or(Head::Unproven, head);
        self.calls.push(Call {
            span: span(value.span),
            head,
        });
        walk::walk_call_expression(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                    if matches!(
                        head,
                        Head::Literal | Head::Tag | Head::Missing | Head::MissingTag
                    ) {
                        let key = span(binding.span);
                        self.constants.insert(key, head);
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
