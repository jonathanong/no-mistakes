use super::QueueVisitor;
use crate::queue::extract_helpers::{collect_jobs_from_args, literal_expr, processor_specifier};
use crate::queue::extract_model::WorkerSite;
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, Expression, NewExpression};
use oxc_span::GetSpan;

impl QueueVisitor<'_> {
    pub fn is_queue_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.queue_classes.contains(id.name.as_str()))
    }

    pub fn is_worker_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.worker_classes.contains(id.name.as_str()))
    }

    pub fn is_flow_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.flow_classes.contains(id.name.as_str()))
    }

    pub(super) fn note_constructed_worker(&mut self, new_expr: &NewExpression<'_>) {
        if !self.is_worker_constructor(&new_expr.callee) {
            return;
        }
        let queue_name = new_expr
            .arguments
            .first()
            .and_then(|arg| self.literal_arg(arg));
        let mut jobs = collect_jobs_from_args(&new_expr.arguments, self.source);
        jobs.sort();
        jobs.dedup();
        let processor_specifier =
            processor_specifier(&new_expr.arguments, self.source, &self.namespace_imports);
        self.facts.workers.push(WorkerSite {
            file: self.path.to_path_buf(),
            line: line_number(self.source, new_expr.span.start),
            queue_name,
            queue_name_import: None,
            wildcard: jobs.is_empty(),
            jobs,
            processor_specifier,
            processor_file: None,
        });
        if let Some(arg) = new_expr.arguments.get(1) {
            let span = arg.span();
            let index = self.facts.workers.len() - 1;
            self.processor_spans.push((span.start, span.end, index));
        }
    }

    pub(super) fn stamp_worker(&mut self, recorded: usize) {
        let Some(index) = self.worker_stack.last().copied() else {
            return;
        };
        for producer in &mut self.facts.producers[recorded..] {
            producer.worker_index = Some(index);
        }
    }

    /// `ns.q.add`: the root is an import, so the member call is a producer even
    /// when the queue behind that member does not resolve.
    pub(super) fn imported_member_binding(&self, object: &Expression<'_>) -> Option<String> {
        let Expression::StaticMemberExpression(member) = object else {
            return None;
        };
        let Expression::Identifier(root) = &member.object else {
            return None;
        };
        let name = root.name.as_str();
        if !self.facts.string_imports.contains_key(name)
            && !self.namespace_imports.contains_key(name)
        {
            return None;
        }
        Some(format!("{name}.{}", member.property.name))
    }

    pub fn tracks_job_name(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => self.job_frames.iter().any(|frame| {
                frame
                    .name_idents
                    .iter()
                    .any(|name| name == id.name.as_str())
            }),
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(object) = &member.object else {
                    return false;
                };
                member.property.name == "name"
                    && self
                        .job_frames
                        .iter()
                        .any(|frame| frame.object.as_deref() == Some(object.name.as_str()))
            }
            _ => false,
        }
    }

    pub fn tracks_job_token(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => self
                .job_frames
                .iter()
                .any(|frame| frame.id_idents.iter().any(|name| name == id.name.as_str())),
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(object) = &member.object else {
                    return false;
                };
                matches!(member.property.name.as_str(), "id" | "deduplicationId")
                    && self
                        .job_frames
                        .iter()
                        .any(|frame| frame.object.as_deref() == Some(object.name.as_str()))
            }
            _ => false,
        }
    }

    pub fn literal_arg(&self, arg: &Argument<'_>) -> Option<String> {
        match arg {
            Argument::StringLiteral(value) => Some(value.value.as_str().to_string()),
            Argument::Identifier(id) => self.const_strings.get(id.name.as_str()).cloned(),
            _ => arg
                .as_expression()
                .and_then(|expr| literal_expr(expr, &self.const_strings)),
        }
    }
}
