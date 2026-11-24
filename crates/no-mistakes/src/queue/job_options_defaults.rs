use super::super::extract_model::{JobOptionEvidence, QueueDefaultOptions};
use super::super::extract_visitor::QueueVisitor;
use super::site::{evidence_at, positional_args_open, push_site};
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

#[path = "job_options_default_read.rs"]
mod read;

pub(crate) fn note_factory_defaults(
    visitor: &mut QueueVisitor<'_>,
    declaration: u32,
    queue_name: Option<&str>,
    name_index: usize,
    call: &CallExpression<'_>,
) {
    let mut recorded = blank(
        visitor,
        call.span.start,
        declaration,
        queue_name.map(str::to_string),
    );
    for (index, arg) in call.arguments.iter().enumerate() {
        if index == name_index {
            continue;
        }
        note_options_argument(arg, &visitor.object_bindings, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_constructed_defaults(
    visitor: &mut QueueVisitor<'_>,
    declaration: u32,
    new_expr: &oxc_ast::ast::NewExpression<'_>,
) {
    let mut recorded = blank(
        visitor,
        new_expr.span.start,
        declaration,
        new_expr
            .arguments
            .first()
            .and_then(|arg| visitor.literal_arg(arg)),
    );
    if positional_args_open(&new_expr.arguments, 1) {
        recorded.evidence = JobOptionEvidence::open();
    } else if let Some(arg) = new_expr.arguments.get(1) {
        note_options_argument(arg, &visitor.object_bindings, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_argument_options(
    visitor: &mut QueueVisitor<'_>,
    binding: &str,
    job: &Option<String>,
    call: &CallExpression<'_>,
    index: Option<usize>,
) {
    push_site(
        visitor,
        call.span().start,
        Some(binding.to_string()),
        None,
        job.clone(),
        index.map_or_else(JobOptionEvidence::default, |index| evidence_at(call, index)),
    );
}

fn blank(
    visitor: &QueueVisitor<'_>,
    start: u32,
    declaration: u32,
    queue_name: Option<String>,
) -> QueueDefaultOptions {
    QueueDefaultOptions {
        declaration,
        line: super::super::source::line_number(visitor.source, start),
        queue_name,
        evidence: JobOptionEvidence::default(),
    }
}

fn note_options_argument(
    arg: &Argument<'_>,
    bindings: &ObjectBindings<'_>,
    recorded: &mut QueueDefaultOptions,
) {
    let Some(expr) = arg.as_expression() else {
        recorded.evidence = JobOptionEvidence::open();
        return;
    };
    if !matches!(expr.get_inner_expression(), Expression::StringLiteral(_)) {
        recorded.evidence = read::queue_defaults(expr, bindings);
    }
}
