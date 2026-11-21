use super::super::extract_model::{JobOptionEvidence, JobOptionSite};
use super::super::extract_visitor::QueueVisitor;
use super::super::source::line_number;
use super::walk::read;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind};

pub(super) fn evidence_at(call: &CallExpression<'_>, index: usize) -> JobOptionEvidence {
    if let Some(arg) = call.arguments.get(index) {
        if let Some(expr) = arg.as_expression() {
            return read(expr);
        }
        return JobOptionEvidence::open();
    }
    if call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::SpreadElement(_)))
    {
        return JobOptionEvidence::open();
    }
    JobOptionEvidence::default()
}

pub(super) fn push_site(
    visitor: &mut QueueVisitor<'_>,
    start: u32,
    binding: Option<String>,
    queue_name: Option<String>,
    job: Option<String>,
    evidence: JobOptionEvidence,
) {
    visitor.facts.job_option_sites.push(JobOptionSite {
        line: line_number(visitor.source, start),
        binding,
        queue_name,
        queue_file: None,
        job,
        evidence,
    });
}

pub(super) fn job_at(
    visitor: &QueueVisitor<'_>,
    call: &CallExpression<'_>,
    index: usize,
) -> Option<String> {
    call.arguments
        .get(index)
        .and_then(|arg| visitor.literal_arg(arg))
}

pub(super) fn member_expr<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (crate::ast::property_key_name(&property.key).as_deref() == Some(key))
            .then_some(&property.value)
    })
}

pub(super) fn member_string(object: &ObjectExpression<'_>, key: &str) -> Option<String> {
    member_expr(object, key).and_then(super::super::extract_helpers::static_string)
}
