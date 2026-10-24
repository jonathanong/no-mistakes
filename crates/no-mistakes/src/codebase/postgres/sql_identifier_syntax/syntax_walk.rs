use super::*;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, Function, NewExpression, TaggedTemplateExpression,
    TemplateLiteral,
};
use oxc_syntax::scope::ScopeFlags;

impl<'a> Visit<'a> for Syntax {
    fn visit_program(&mut self, program: &Program<'a>) {
        walk::walk_program(self, program);
        for call in &mut self.calls {
            self.template_binds.validate(&mut call.input);
        }
    }
    fn visit_simple_assignment_target(
        &mut self,
        target: &oxc_ast::ast::SimpleAssignmentTarget<'a>,
    ) {
        self.template_binds.write(target);
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.template_binds.write_span(span(target.binding.span));
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
                    self.namespace_members.insert(root, key.value.to_string());
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
        let callee = chain(&value.callee);
        let selected = callee.as_ref().is_some_and(|(root, parts)| {
            (parts.len() == 1 && (self.targets.contains(root) || self.aliases.contains_key(root)))
                || (parts.len() == 2 && self.aliases.contains_key(root) && parts[1] == "query")
                || (self.typed_targets.contains(root) && input::indirect_executor_call(parts))
                || (parts.len() == 2
                    && self
                        .namespace_targets
                        .get(root)
                        .is_some_and(|names| names.contains(&parts[1])))
                || (self.loose_query
                    && parts.len() > 1
                    && parts.last().is_some_and(|name| name == "query"))
        });
        let root = selected.then(|| callee.as_ref().unwrap().0);
        if selected {
            self.calls.push(Call {
                span: span(value.span),
                input: if callee.as_ref().is_some_and(|(root, parts)| {
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
            });
        }
        let previous = self.metadata.enter_call(root, Some(span(value.span)));
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
        walk::walk_object_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_array_expression(&mut self, value: &oxc_ast::ast::ArrayExpression<'a>) {
        let previous = self.metadata.aggregate();
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
        self.opaque_metadata(|this| walk::walk_function(this, value, flags));
    }
    fn visit_arrow_function_expression(&mut self, value: &ArrowFunctionExpression<'a>) {
        self.opaque_metadata(|this| walk::walk_arrow_function_expression(this, value));
    }
    fn visit_new_expression(&mut self, value: &NewExpression<'a>) {
        let previous = self.metadata.enter_call(None, Some(span(value.span)));
        walk::walk_new_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_tagged_template_expression(&mut self, value: &TaggedTemplateExpression<'a>) {
        let owner = self.template_binds.tag_owner(&value.tag);
        let previous = self.metadata.enter_tag(owner, span(value.span));
        walk::walk_tagged_template_expression(self, value);
        self.metadata.restore(previous);
    }
    fn visit_template_literal(&mut self, value: &TemplateLiteral<'a>) {
        for quasi in &value.quasis {
            self.visit_template_element(quasi);
        }
        for expression in &value.expressions {
            let previous = self.metadata.bind_expression();
            self.visit_expression(expression);
            self.metadata.restore(previous);
        }
    }
}
