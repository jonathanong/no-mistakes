use super::{ImportedMember, Member, QueueVisitor};
use crate::queue::extract_job_id::referenced_symbol;
use oxc_ast::ast::{Argument, BindingPattern, Expression, FormalParameters, TSType, TSTypeName};

impl QueueVisitor<'_> {
    pub(in crate::queue) fn note_constructor_rate_limit_processor(
        &mut self,
        callee: &Expression<'_>,
        argument: &Argument<'_>,
        worker: usize,
    ) {
        if self.shared_rate_limit.enabled
            && self
                .worker_member(callee)
                .is_some_and(|import| self.shared_rate_limit.matches_value(&import))
        {
            self.note_rate_limit_processor(argument, worker, false);
        }
    }

    fn worker_member(&self, expr: &Expression<'_>) -> Option<ImportedMember> {
        if let Expression::StaticMemberExpression(member) = expr.get_inner_expression() {
            if matches!(member.property.name.as_str(), "Worker" | "TestWorker") {
                return self.rate_limit_imported_member(&member.object, Member::Namespace);
            }
        }
        self.rate_limit_imported_member(expr, Member::Worker)
    }

    fn rate_limit_imported_member(
        &self,
        expr: &Expression<'_>,
        member: Member,
    ) -> Option<ImportedMember> {
        let Expression::Identifier(id) = expr.get_inner_expression() else {
            return None;
        };
        Some(ImportedMember {
            symbol: referenced_symbol(self, id)?,
            member,
        })
    }

    pub(in crate::queue::extract_visitor::rate_limit) fn note_worker_parameters(
        &mut self,
        start: u32,
        params: &FormalParameters<'_>,
    ) {
        for param in &params.items {
            let Some(annotation) = &param.type_annotation else {
                continue;
            };
            let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
                continue;
            };
            let (name, member) = match &reference.type_name {
                TSTypeName::IdentifierReference(name) => (name, Member::Worker),
                TSTypeName::QualifiedName(qualified)
                    if matches!(qualified.right.name.as_str(), "Worker" | "TestWorker") =>
                {
                    let TSTypeName::IdentifierReference(name) = &qualified.left else {
                        continue;
                    };
                    (name, Member::Namespace)
                }
                _ => continue,
            };
            let BindingPattern::BindingIdentifier(binding) = &param.pattern else {
                continue;
            };
            if let (Some(parameter), Some(symbol)) = (
                self.stable_rate_limit_symbol(binding),
                referenced_symbol(self, name),
            ) {
                self.shared_rate_limit
                    .parameters
                    .insert(parameter, (start, ImportedMember { symbol, member }));
            }
        }
    }
}
