use super::super::extract_model::{JobOptionEvidence, QueueDefaultOptions};
use super::super::extract_visitor::QueueVisitor;
use super::site::{evidence_at, push_site};
use super::walk::read_member;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

pub(crate) fn note_factory_defaults(
    visitor: &mut QueueVisitor<'_>,
    binding: &str,
    queue_name: Option<&str>,
    name_index: usize,
    call: &CallExpression<'_>,
) {
    let mut recorded = blank(binding, queue_name.map(str::to_string));
    for (index, arg) in call.arguments.iter().enumerate() {
        if index == name_index {
            continue;
        }
        note_options_argument(arg, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_constructed_defaults(
    visitor: &mut QueueVisitor<'_>,
    binding: &str,
    new_expr: &oxc_ast::ast::NewExpression<'_>,
) {
    let mut recorded = blank(
        binding,
        new_expr
            .arguments
            .first()
            .and_then(|arg| visitor.literal_arg(arg)),
    );
    if let Some(arg) = new_expr.arguments.get(1) {
        note_options_argument(arg, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_argument_options(
    visitor: &mut QueueVisitor<'_>,
    binding: &str,
    job: &Option<String>,
    call: &CallExpression<'_>,
    index: usize,
) {
    push_site(
        visitor,
        call.span().start,
        Some(binding.to_string()),
        None,
        job.clone(),
        evidence_at(call, index),
    );
}

fn blank(binding: &str, queue_name: Option<String>) -> QueueDefaultOptions {
    QueueDefaultOptions {
        binding: Some(binding.to_string()),
        queue_name,
        container: None,
        evidence: JobOptionEvidence::default(),
    }
}

fn note_options_argument(arg: &Argument<'_>, recorded: &mut QueueDefaultOptions) {
    let Some(expr) = arg.as_expression() else {
        recorded.container = None;
        recorded.evidence = JobOptionEvidence::open();
        return;
    };
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => {
            recorded.container = None;
            recorded.evidence = read_member(object, "defaultJobOptions").unwrap_or_default();
        }
        Expression::Identifier(id) => {
            recorded.container = Some(id.name.to_string());
            recorded.evidence = JobOptionEvidence::default();
        }
        Expression::StringLiteral(_) => {}
        _ => {
            recorded.container = None;
            recorded.evidence = JobOptionEvidence::open();
        }
    }
}
