use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::Expression;
mod append_visits;
mod collect;
pub(super) use collect::collect_with_helpers;
mod const_visits;
mod destructure;
mod field_visits;
mod finite_visits;
mod head_kind;
mod helper_visits;
pub(super) mod literals;
mod visitor;
pub(super) struct Call {
    pub span: Span,
    pub head: Head,
    pub context: super::assignments::Context,
}
#[derive(Default)]
pub(super) struct Syntax {
    pub collect_helper_heads: bool,
    pub helper_returns: FxHashMap<Span, super::helper_heads::Summary>,
    pub record_returns: FxHashMap<Span, super::helper_records::Summary>,
    pub returned_params: FxHashMap<Span, super::returned_params::Candidate>,
    pub returned_param_roots: FxHashMap<Span, super::returned_params::Root>,
    pub finite_returns: FxHashMap<Span, super::finite_returns::Candidate>,
    pub record_initializers: FxHashMap<Span, (Span, Span)>,
    pub record_destructures: FxHashMap<Span, destructure::Destructure>,
    pub destructure_calls: FxHashMap<Span, (Span, Span)>,
    pub builder_direct_calls: FxHashMap<Span, Span>,
    pub confirmed_builder_reads: FxHashSet<Span>,
    pub repeated_append_refs: FxHashSet<Span>,
    pub optional_calls: FxHashSet<Span>,
    pub helper_arguments: FxHashMap<Span, Span>,
    pub collect_finite: bool,
    pub finite_calls: Vec<super::finite_comments::CallSite>,
    pub finite_templates: Vec<super::finite_comments::TemplateUse>,
    pub finite_roots: Vec<super::finite_comments::Root>,
    pub finite_functions: Vec<super::finite_comments::Function>,
    pub finite_selects: Vec<super::finite_comments::Select>,
    pub active_domains: Vec<super::finite_comments::Domain>,
    pub namespace_destructures: Vec<super::namespace_heads::Candidate>,
    pub effect_functions: FxHashMap<Span, [Option<super::effects::FunctionSyntax>; 2]>,
    pub effect_calls: Vec<super::effects::CallSyntax>,
    pub returned_effect_appends: FxHashSet<Span>,
    pub dynamic_refs: Vec<Span>,
    pub return_sinks: FxHashSet<Span>,
    pub append_tags: FxHashMap<Span, Span>,
    pub collect_append_tags: bool,
    pub literal_helpers: Option<Vec<super::literal_helpers::Candidate>>,
    pub calls: Vec<Call>,
    pub loop_candidates: Vec<super::loop_carried::Candidate>,
    pub constants: FxHashMap<Span, Head>,
    pub string_constants: FxHashSet<Span>,
    pub value_reads: FxHashMap<Span, Span>,
    pub record_result_calls: FxHashSet<Span>,
    pub record_options_calls: FxHashSet<Span>,
    pub assignments: super::assignments::Assignments,
    pub branch_assignments: Vec<super::branch_assignments::Candidate>,
    pub append_receivers: FxHashSet<Span>,
    pub discarded_appends: FxHashSet<Span>,
    pub prefix_discarded_appends: FxHashSet<Span>,
    pub contexts: FxHashMap<Span, Option<Span>>,
    current_function: Option<Span>,
    pub repeated_calls: FxHashSet<Span>,
}
fn span(span: oxc_span::Span) -> Span {
    (span.start, span.end)
}
#[derive(Clone, Copy)]
pub(super) enum Head {
    Literal,
    Tag,
    AppendTag,
    AppendPrefixTag,
    Text(super::text::TextHead),
    Reference(Span),
    FieldReference(Span),
    AppendReference(Span),
    AppendPrefixReference(Span),
    Unproven,
}
pub(super) fn head<'a>(expression: &'a Expression<'a>) -> Head {
    bounded_head(expression, 8)
}
pub(super) fn bounded_head<'a>(mut expression: &'a Expression<'a>, depth: u8) -> Head {
    let mut concatenated = false;
    let mut appended = false;
    let mut opaque_append = false;
    loop {
        expression = unwrap_ts_wrappers(expression);
        if let Expression::CallExpression(call) = expression {
            if let Some(receiver) = super::append::literal_receiver(call) {
                appended = true;
                expression = receiver;
                continue;
            }
            if let Some(receiver) = super::append::prefix_receiver(call) {
                opaque_append = true;
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
    if let Expression::ConditionalExpression(branch) = expression {
        return super::conditional::head(branch, depth, concatenated || appended || opaque_append);
    }
    if literals::literal(expression) {
        if opaque_append {
            return Head::Unproven;
        }
        return if appended {
            Head::AppendTag
        } else {
            Head::Literal
        };
    }
    if !concatenated && !appended && !opaque_append {
        if let Some((_, reference)) = field_visits::field(expression, "text") {
            return Head::FieldReference(reference);
        }
    }
    if !concatenated {
        if let Expression::Identifier(id) = expression {
            if opaque_append {
                return Head::AppendPrefixReference(span(id.span));
            }
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
            .filter(|part| part.value.cooked.is_some())
            .map(crate::ast::template_quasi_text)
            .is_some_and(|text| literals::named(&text))
        {
            return if opaque_append {
                Head::AppendPrefixTag
            } else if appended {
                Head::AppendTag
            } else {
                Head::Tag
            };
        }
    }
    super::text::head(expression, concatenated, appended, opaque_append)
        .map_or(Head::Unproven, Head::Text)
}
