use super::ScopeVisitor;
use crate::codebase::postgres::embedded::bindings::callee_name;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, IdentifierReference, TaggedTemplateExpression,
};
use oxc_ast_visit::{walk, Visit};

impl ScopeVisitor<'_> {
    /// Exempt an exact direct reference, never an entire expression subtree.
    pub(super) fn allow_builder_reference(&mut self, expression: &Expression<'_>) {
        if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
            self.builder_reference_exemptions.push(id.span.start);
        }
    }

    // These are already modeled result projections. Containers are deliberately
    // absent: passing [q] to a known consumer still exposes an unmodeled alias.
    pub(super) fn allow_builder_value(&mut self, expression: &Expression<'_>) {
        match unwrap_ts_wrappers(expression) {
            Expression::ConditionalExpression(branch) => {
                self.allow_builder_value(&branch.consequent);
                self.allow_builder_value(&branch.alternate);
            }
            Expression::LogicalExpression(branch) => {
                self.allow_builder_value(&branch.left);
                self.allow_builder_value(&branch.right);
            }
            Expression::SequenceExpression(sequence) => {
                if let Some(last) = sequence.expressions.last() {
                    self.allow_builder_value(last);
                }
            }
            Expression::AssignmentExpression(assignment)
                if matches!(
                    assignment.left,
                    AssignmentTarget::AssignmentTargetIdentifier(_)
                ) =>
            {
                self.allow_builder_value(&assignment.right);
            }
            _ => self.allow_builder_reference(expression),
        }
    }

    pub(super) fn visit_builder_template<'a>(&mut self, template: &TaggedTemplateExpression<'a>)
    where
        Self: Visit<'a>,
    {
        let start = self.builder_reference_exemptions.len();
        if super::super::tags::is_sql_tag(
            &template.tag,
            &mut |name| super::resolve::compose::tag_shadowed(name, self),
            self.functions.imported_sql_tags(),
        ) {
            for expression in &template.quasi.expressions {
                self.allow_builder_value(expression);
            }
        }
        walk::walk_tagged_template_expression(self, template);
        self.builder_reference_exemptions.truncate(start);
    }

    pub(super) fn with_builder_receiver(
        &mut self,
        receiver: &Expression<'_>,
        text: bool,
        walk: impl FnOnce(&mut Self),
    ) {
        let start = self.builder_reference_exemptions.len();
        if text {
            self.allow_builder_reference(receiver);
        }
        walk(self);
        self.builder_reference_exemptions.truncate(start);
    }

    pub(super) fn begin_safe_builder_arguments(
        &mut self,
        call: &CallExpression<'_>,
        escaped: &[(String, Option<u32>)],
        modeled_builder: Option<&str>,
    ) -> usize {
        let start = self.builder_reference_exemptions.len();
        match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                self.allow_builder_value(&member.object);
                for argument in call
                    .arguments
                    .iter()
                    .filter_map(|argument| argument.as_expression())
                {
                    self.allow_builder_value(argument);
                }
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") =>
            {
                self.allow_builder_value(&member.object);
                for argument in call
                    .arguments
                    .iter()
                    .filter_map(|argument| argument.as_expression())
                {
                    self.allow_builder_value(argument);
                }
            }
            _ => {}
        }
        let parameter = if self.executor_alias_callee(&call.callee)
            || callee_name(call, self.bindings, self.scoped).is_some()
        {
            Some(0)
        } else {
            match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(id) if !self.shadowed_locally(id.name.as_str()) => self
                    .functions
                    .parameter_builder(id.name.as_str())
                    .map(|helper| helper.parameter_index),
                _ => None,
            }
        };
        if let Some(argument) = parameter
            .and_then(|index| call.arguments.get(index))
            .and_then(|argument| argument.as_expression())
        {
            self.allow_builder_value(argument);
        }
        if modeled_builder.is_some() || !escaped.is_empty() {
            for argument in call
                .arguments
                .iter()
                .filter_map(|argument| argument.as_expression())
            {
                if self.builder_result_names(argument).iter().any(|name| {
                    modeled_builder == Some(name.as_str())
                        || escaped.iter().any(|effect| effect.0 == *name)
                }) {
                    self.allow_builder_value(argument);
                }
            }
        }
        start
    }

    pub(super) fn escape_builder_reference(&mut self, id: &IdentifierReference<'_>) {
        if self.builder_reference_exemptions.contains(&id.span.start) {
            return;
        }
        let name = id.name.as_str();
        let Some(identity) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .filter(|binding| {
                binding.initialized
                    && (binding.builder_identity.is_some()
                        || binding
                            .variants
                            .as_ref()
                            .is_some_and(|values| values.iter().any(|value| value.fragment)))
            })
            .map(|binding| binding.builder_identity)
        else {
            return;
        };
        self.record_argument_helper_effects(&[(name.to_owned(), identity)], id.span.start);
        // Returning/yielding or retaining a captured object can expose the
        // original even after this function's lexical scope has disappeared.
        // Conservatively invalidate it without requiring proof of invocation.
        self.apply_builder_effect(name.to_owned(), identity, None, true, false, 0);
    }
}
