use super::*;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, Function, NewExpression, SimpleAssignmentTarget,
    TaggedTemplateExpression, TemplateLiteral,
};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
mod recovery;
impl<'a> Visit<'a> for Syntax<'_> {
    fn visit_with_statement(&mut self, value: &oxc_ast::ast::WithStatement<'a>) {
        self.recovery.dynamic_scope = true;
        walk::walk_with_statement(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &oxc_ast::ast::VariableDeclaration<'a>) {
        if self.recover_helpers {
            self.recovery.local(
                value,
                &helper_recovery::lower::Context {
                    source: self.source,
                    bindings: &self.recovery_bindings,
                    tags: &self.template_binds,
                },
            );
        }
        walk::walk_variable_declaration(self, value);
    }
    fn visit_expression(&mut self, value: &Expression<'a>) {
        if !matches!(
            unwrap_ts_wrappers(value),
            Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
        ) {
            self.metadata.reference(span(value.span()));
        }
        walk::walk_expression(self, value);
    }
    fn visit_program(&mut self, program: &Program<'a>) {
        walk::walk_program(self, program);
        for call in &mut self.calls {
            self.template_binds.validate(&mut call.input);
        }
    }
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        self.template_binds.write(target);
        self.metadata.assignment_receiver(target);
        if self.recover_helpers {
            if let Some(at) = template_binds::assignment_root(target) {
                if let Some(id) = self.recovery_bindings.id(oxc_span::Span::new(at.0, at.1)) {
                    self.recovery.writes.insert(id);
                }
            }
        }
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.template_binds.write_span(span(target.binding.span));
        if self.recover_helpers {
            if let Some(id) = self.recovery_bindings.id(target.binding.span) {
                self.recovery.writes.insert(id);
            }
        }
        walk::walk_assignment_target_property_identifier(self, target);
    }
    fn visit_static_member_expression(&mut self, value: &oxc_ast::ast::StaticMemberExpression<'a>) {
        self.metadata.reference(span(value.span));
        if let Some((root, parts)) = chain(&value.object) {
            if parts.len() == 1 && self.namespace_targets.contains_key(&root) {
                self.namespace_members
                    .insert(root, value.property.name.to_string());
            }
        }
        walk::walk_static_member_expression(self, value);
    }
    fn visit_computed_member_expression(
        &mut self,
        value: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        self.metadata.reference(span(value.span));
        if let Some((root, parts)) = chain(&value.object) {
            if parts.len() == 1 && self.namespace_targets.contains_key(&root) {
                if let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) {
                    self.namespace_members
                        .insert(root, crate::ast::js_literal(key).into_owned());
                }
            }
        }
        walk::walk_computed_member_expression(self, value);
    }
    fn visit_private_field_expression(&mut self, value: &oxc_ast::ast::PrivateFieldExpression<'a>) {
        self.metadata.reference(span(value.span));
        walk::walk_private_field_expression(self, value);
    }
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        self.metadata.reference(span(value.span));
        let callee = chain(&value.callee);
        let unproven_member = self.scoped_member_advisory
            && callee.as_ref().is_some_and(|(_, parts)| {
                parts.len() > 1 && parts.last().is_some_and(|name| name == "query")
            })
            && !value.arguments.is_empty();
        let selected = self.detached_calls.contains(&span(value.span))
            || callee
                .as_ref()
                .is_some_and(|(root, parts)| self.selected_callee(root, parts))
            || unproven_member;
        self.recover_call(value, selected);
        let root = selected.then(|| callee.as_ref().unwrap().0);
        if selected {
            self.calls.push(Call {
                span: span(value.span),
                input: if self.detached_calls.contains(&span(value.span))
                    || callee.as_ref().is_some_and(|(root, parts)| {
                        (parts.len() == 1 && self.targets.contains(root))
                            || (parts.len() == 2
                                && parts[1] == "query"
                                && self.typed_targets.contains(root))
                    }) {
                    value
                        .arguments
                        .first()
                        .and_then(|arg| arg.as_expression())
                        .map_or(Input::Unknown, |value| {
                            input(value, &mut self.env_references, &self.template_binds)
                        })
                } else {
                    Input::Unknown
                },
                callee,
                unproven_member,
            });
        }
        let previous = self.metadata.enter_call(
            root,
            Some(span(value.span)),
            Some(span(value.callee.span())),
        );
        walk::walk_call_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_identifier_reference(&mut self, value: &oxc_ast::ast::IdentifierReference<'a>) {
        self.metadata.reference(span(value.span));
    }
    fn visit_arguments(&mut self, arguments: &oxc_allocator::Vec<'a, Argument<'a>>) {
        for (index, argument) in arguments.iter().enumerate() {
            let previous = self.metadata.argument(index);
            self.visit_argument(argument);
            self.metadata.restore(previous);
        }
    }
    fn visit_object_expression(&mut self, value: &oxc_ast::ast::ObjectExpression<'a>) {
        let previous = self.metadata.aggregate();
        self.metadata.reference(span(value.span));
        walk::walk_object_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_array_expression(&mut self, value: &oxc_ast::ast::ArrayExpression<'a>) {
        let previous = self.metadata.aggregate();
        self.metadata.reference(span(value.span));
        walk::walk_array_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_assignment_expression(&mut self, value: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.opaque_metadata(|this| walk::walk_assignment_expression(this, value));
    }
    fn visit_yield_expression(&mut self, value: &oxc_ast::ast::YieldExpression<'a>) {
        self.opaque_metadata(|this| walk::walk_yield_expression(this, value));
    }
    fn visit_class(&mut self, value: &oxc_ast::ast::Class<'a>) {
        self.opaque_metadata(|this| walk::walk_class(this, value));
    }
    fn visit_function(&mut self, value: &Function<'a>, flags: ScopeFlags) {
        self.recover_function(value, flags);
    }
    fn visit_arrow_function_expression(&mut self, value: &ArrowFunctionExpression<'a>) {
        self.recover_arrow(value);
    }
    fn visit_new_expression(&mut self, value: &NewExpression<'a>) {
        self.recover_constructor(value);
    }
    fn visit_tagged_template_expression(&mut self, value: &TaggedTemplateExpression<'a>) {
        let owner = self.template_binds.tag_owner(&value.tag);
        let previous = self.metadata.enter_tag(owner, span(value.span));
        walk::walk_tagged_template_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_template_literal(&mut self, value: &TemplateLiteral<'a>) {
        self.template_literal(value);
    }
}
