use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{Expression, Program};
use oxc_ast_visit::Visit;
mod append_visits;
mod visitor;

pub(super) struct Call {
    pub span: Span,
    pub head: Head,
    pub context: super::assignments::Context,
}
#[derive(Default)]
pub(super) struct Syntax {
    pub append_tags: FxHashMap<Span, Span>,
    pub collect_append_tags: bool,
    pub calls: Vec<Call>,
    pub constants: FxHashMap<Span, Head>,
    pub assignments: super::assignments::Assignments,
    pub append_receivers: FxHashSet<Span>,
    pub discarded_appends: FxHashSet<Span>,
    pub contexts: FxHashMap<Span, Option<Span>>,
    current_function: Option<Span>,
    pub repeated_calls: FxHashSet<Span>,
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
#[derive(Clone, Copy)]
pub(super) enum Head {
    Literal,
    Tag,
    AppendTag,
    Text(super::text::TextHead),
    Reference(Span),
    AppendReference(Span),
    Unproven,
}
impl Head {
    pub fn is_constant(self) -> bool {
        matches!(self, Self::Literal | Self::Tag | Self::AppendTag)
            || matches!(self, Self::Text(value) if value.current != super::HeadProof::Unproven)
    }
    pub fn tagged(self) -> bool {
        matches!(self, Self::Tag | Self::AppendTag)
            || matches!(self, Self::Text(value) if value.tagged)
    }
    pub fn appended(self) -> bool {
        matches!(self, Self::AppendTag) || matches!(self, Self::Text(value) if value.appended)
    }
    pub fn proof(self, continued: bool) -> super::HeadProof {
        match self {
            Self::Literal | Self::Tag | Self::AppendTag => super::HeadProof::Literal,
            Self::Text(value) if continued => value.continued,
            Self::Text(value) => value.current,
            _ => super::HeadProof::Unproven,
        }
    }
    pub fn reference(self) -> Option<Span> {
        match self {
            Self::Reference(span) | Self::AppendReference(span) => Some(span),
            _ => None,
        }
    }
}
// Preserve Tag through concatenation so local constants still require safe uses.
pub(super) fn head<'a>(mut expression: &'a Expression<'a>) -> Head {
    let mut concatenated = false;
    let mut appended = false;
    loop {
        expression = unwrap_ts_wrappers(expression);
        if let Expression::CallExpression(call) = expression {
            if let Some(receiver) = super::append::literal_receiver(call) {
                appended = true;
                expression = receiver;
                continue;
            }
        }
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
        return if appended {
            Head::AppendTag
        } else {
            Head::Literal
        };
    }
    if !concatenated {
        if let Expression::Identifier(id) = expression {
            return if appended {
                Head::AppendReference(span(id.span))
            } else {
                Head::Reference(span(id.span))
            };
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
            return if appended { Head::AppendTag } else { Head::Tag };
        }
    }
    super::text::head(expression, concatenated, appended).map_or(Head::Unproven, Head::Text)
}
pub(super) fn collect(program: &Program<'_>, collect_append_tags: bool) -> Syntax {
    let mut syntax = Syntax {
        collect_append_tags,
        ..Default::default()
    };
    syntax.visit_program(program);
    syntax
}
