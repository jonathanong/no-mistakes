use super::super::{resolve::compose, ScopeVisitor};
use crate::codebase::postgres::embedded::tags;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTarget, BindingPattern, CallExpression,
    Expression, VariableDeclarator,
};
use oxc_span::GetSpan;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn builder_identity_for_init(
        &self,
        expression: &Expression<'_>,
    ) -> Option<u32> {
        let mut current = expression;
        loop {
            match unwrap_ts_wrappers(current) {
                Expression::Identifier(id) => {
                    return self.lookup(id.name.as_str())?.builder_identity
                }
                Expression::TaggedTemplateExpression(tagged) => {
                    return self
                        .trusted_builder_tag(&tagged.tag)
                        .then_some(current.span().start);
                }
                Expression::CallExpression(call) => {
                    if self.fresh_builder_call(call) {
                        return Some(current.span().start);
                    }
                    if matches!(unwrap_ts_wrappers(&call.callee), Expression::StaticMemberExpression(member) if member.property.name != "append")
                    {
                        return None;
                    }
                    if matches!(unwrap_ts_wrappers(&call.callee), Expression::ComputedMemberExpression(member)
                        if !matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append"))
                    {
                        return None;
                    }
                    current = self.returned_builder_argument(call)?;
                }
                _ => return None,
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn refresh_builder_identity(
        &mut self,
        declaration: &VariableDeclarator<'_>,
    ) {
        let BindingPattern::BindingIdentifier(id) = &declaration.id else {
            return;
        };
        let Some(init) = &declaration.init else {
            return;
        };
        let identity = self.builder_identity_for_init(init);
        self.record_builder_aliases(id.name.as_str(), init);
        self.set_builder_identity(id.name.as_str(), identity);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn assignment_builder_identity(
        &mut self,
        assignment: &AssignmentExpression<'_>,
    ) -> Option<u32> {
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left else {
            return None;
        };
        if assignment.operator != AssignmentOperator::Assign {
            return None;
        };
        self.record_builder_aliases(id.name.as_str(), &assignment.right);
        (self.control_depth == 0 && self.loop_depth == 0)
            .then(|| self.builder_identity_for_init(&assignment.right))
            .flatten()
    }

    pub(in crate::codebase::postgres::embedded::walk) fn set_builder_identity(
        &mut self,
        name: &str,
        identity: Option<u32>,
    ) {
        if let Some(binding) = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
        {
            binding.builder_identity = identity;
        }
    }

    fn trusted_builder_tag(&self, tag: &Expression<'_>) -> bool {
        tags::is_sql_tag(
            tag,
            &mut |name| compose::tag_shadowed(name, self),
            self.functions.imported_sql_tags(),
        )
    }

    fn fresh_builder_call(&self, call: &CallExpression<'_>) -> bool {
        self.trusted_builder_tag(&call.callee)
            || matches!(unwrap_ts_wrappers(&call.callee), Expression::StaticMemberExpression(member)
                if matches!(member.property.name.as_str(), "raw" | "join") && self.trusted_builder_tag(&member.object))
    }

    fn returned_builder_argument<'a>(
        &self,
        call: &'a CallExpression<'a>,
    ) -> Option<&'a Expression<'a>> {
        match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => Some(&member.object),
            Expression::ComputedMemberExpression(member) => Some(&member.object),
            Expression::Identifier(id) if !self.shadowed_locally(id.name.as_str()) => {
                let helper = self.functions.parameter_builder(id.name.as_str())?;
                call.arguments.get(helper.parameter_index)?.as_expression()
            }
            _ => None,
        }
    }

    pub(super) fn builder_result_names(&self, expression: &Expression<'_>) -> Vec<String> {
        let mut names = Vec::new();
        let mut pending = vec![expression];
        while let Some(expression) = pending.pop() {
            match unwrap_ts_wrappers(expression) {
                Expression::Identifier(parent) => names.push(parent.name.to_string()),
                Expression::ConditionalExpression(branch) => {
                    pending.extend([&branch.consequent, &branch.alternate])
                }
                Expression::LogicalExpression(branch) => {
                    pending.extend([&branch.left, &branch.right])
                }
                Expression::SequenceExpression(sequence) => {
                    pending.extend(sequence.expressions.last())
                }
                Expression::CallExpression(call) if !self.fresh_builder_call(call) => {
                    pending.extend(self.returned_builder_argument(call))
                }
                _ => {}
            }
        }
        names
    }

    fn record_builder_aliases(&mut self, name: &str, expression: &Expression<'_>) {
        for parent in self.builder_result_names(expression) {
            self.builder_aliases
                .entry(parent.clone())
                .or_default()
                .push(name.to_string());
            self.builder_aliases
                .entry(name.to_string())
                .or_default()
                .push(parent);
        }
    }
}
