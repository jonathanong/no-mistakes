use crate::queue::extract_helpers::{binding_name, literal_expr, JobFrame};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Expression, FormalParameters, Function, Statement,
    SwitchStatement,
};
use oxc_ast_visit::{walk, Visit};

pub(crate) fn visit_function<'a>(
    visitor: &mut QueueVisitor<'a>,
    func: &Function<'a>,
    flags: oxc_syntax::scope::ScopeFlags,
) {
    let frames = visitor.job_frames.clone();
    super::dedup::mask_parameters(visitor, &func.params);
    let worker = enter_worker(visitor, func.span);
    enter_job(visitor, &func.params, func.span);
    walk::walk_function(visitor, func, flags);
    visitor.job_frames = frames;
    if worker {
        visitor.worker_stack.pop();
    }
}

pub(crate) fn visit_arrow<'a>(visitor: &mut QueueVisitor<'a>, arrow: &ArrowFunctionExpression<'a>) {
    let frames = visitor.job_frames.clone();
    super::dedup::mask_parameters(visitor, &arrow.params);
    let worker = enter_worker(visitor, arrow.span);
    enter_job(visitor, &arrow.params, arrow.span);
    walk::walk_arrow_function_expression(visitor, arrow);
    visitor.job_frames = frames;
    if worker {
        visitor.worker_stack.pop();
    }
}

fn enter_worker(visitor: &mut QueueVisitor<'_>, span: oxc_span::Span) -> bool {
    let Some(index) = visitor
        .processor_spans
        .iter()
        .rev()
        .find(|record| span.start >= record.0 && span.end <= record.1)
        .map(|record| record.2)
    else {
        return false;
    };
    visitor.worker_stack.push(index);
    true
}

pub(crate) fn visit_switch<'a>(visitor: &mut QueueVisitor<'a>, stmt: &SwitchStatement<'a>) {
    let tracks = visitor.tracks_job_name(&stmt.discriminant);
    super::super::extract_processor::note_switch(visitor, stmt);
    if !tracks {
        visitor.visit_expression(&stmt.discriminant);
    }
    for case in &stmt.cases {
        let label = tracks
            .then(|| case_label(visitor, case.test.as_ref()))
            .flatten();
        let previous = visitor.current_case.clone();
        if label.is_some() {
            visitor.current_case = label;
        }
        if let Some(test) = &case.test {
            visitor.visit_expression(test);
        }
        for statement in &case.consequent {
            visit_statement(visitor, statement);
        }
        visitor.current_case = previous;
    }
}

fn enter_job(
    visitor: &mut QueueVisitor<'_>,
    params: &FormalParameters<'_>,
    span: oxc_span::Span,
) -> bool {
    let processor = claim_processor(visitor, span);
    let Some(mut frame) = job_frame(params, processor) else {
        return false;
    };
    frame.scope = span.start;
    visitor.dispatch_scopes.insert(span.start);
    visitor.job_frames.push(frame);
    true
}

/// The outermost function inside a processor argument is the processor, so a
/// parenthesized, cast, or call-wrapped callback still gets the first-parameter
/// fallback. A nested callback does not: its span was already claimed.
fn claim_processor(visitor: &mut QueueVisitor<'_>, span: oxc_span::Span) -> bool {
    let Some(index) = visitor
        .processor_spans
        .iter()
        .enumerate()
        .filter(|(_, record)| span.start >= record.0 && span.end <= record.1)
        .min_by_key(|(_, record)| record.1.saturating_sub(record.0))
        .map(|(index, _)| index)
    else {
        return false;
    };
    if visitor.claimed_processors.contains(&index) {
        return false;
    }
    visitor.claimed_processors.push(index);
    true
}

/// A parameter named `job` is the job in any function. The first-parameter
/// fallback applies only to the processor itself, so a nested callback such as
/// `items.map((item) => queue.add(..., { jobId: item.id }))` is not a self-dedup.
fn job_frame(params: &FormalParameters<'_>, processor: bool) -> Option<JobFrame> {
    let named = params
        .items
        .iter()
        .find(|param| binding_name(&param.pattern).as_deref() == Some("job"));
    let pattern = if let Some(param) = named {
        &param.pattern
    } else if processor {
        &params.items.first()?.pattern
    } else {
        return None;
    };
    frame_from_pattern(pattern)
}

fn frame_from_pattern(pattern: &BindingPattern<'_>) -> Option<JobFrame> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(JobFrame {
            object: Some(id.name.as_str().to_string()),
            ..JobFrame::default()
        }),
        BindingPattern::ObjectPattern(object) => {
            let mut frame = JobFrame::default();
            for property in &object.properties {
                let Some(key) = property.key.static_name().filter(|_| !property.computed) else {
                    continue;
                };
                let Some(name) = bound_ident(&property.value) else {
                    continue;
                };
                match key.as_ref() {
                    "name" => frame.name_idents.push(name.to_string()),
                    "id" | "deduplicationId" => frame.id_idents.push(name.to_string()),
                    _ => {}
                }
            }
            (!frame.name_idents.is_empty() || !frame.id_idents.is_empty()).then_some(frame)
        }
        _ => None,
    }
}

fn bound_ident<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        BindingPattern::AssignmentPattern(assignment) => bound_ident(&assignment.left),
        _ => None,
    }
}

fn case_label(visitor: &QueueVisitor<'_>, test: Option<&Expression<'_>>) -> Option<String> {
    literal_expr(test?, &visitor.const_strings)
}

fn visit_statement<'a>(visitor: &mut QueueVisitor<'a>, statement: &Statement<'a>) {
    visitor.visit_statement(statement);
}
