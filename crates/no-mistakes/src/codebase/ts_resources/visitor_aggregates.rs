use super::ResourceVisitor;
use oxc_ast::ast::{Class, Expression, MethodDefinition, ObjectProperty, PropertyDefinition};
use oxc_ast_visit::walk;

impl<'a> ResourceVisitor<'a> {
    pub(super) fn visit_default_class(&mut self, scope: String, class: &Class<'a>) {
        self.push_aggregate(scope);
        walk::walk_class(self, class);
        self.pop_aggregate();
    }

    pub(super) fn visit_method_definition_impl(&mut self, method: &MethodDefinition<'a>) {
        walk::walk_decorators(self, &method.decorators);
        walk::walk_property_key(self, &method.key);
        let name =
            crate::codebase::ts_source::static_property_key_name(&method.key).map(str::to_string);
        self.visit_scoped_function(
            self.aggregate_member_scope(name),
            &method.value,
            oxc_syntax::scope::ScopeFlags::empty(),
        );
    }

    pub(super) fn visit_object_property_impl(&mut self, property: &ObjectProperty<'a>) {
        let name =
            crate::codebase::ts_source::static_property_key_name(&property.key).map(str::to_string);
        match &property.value {
            Expression::FunctionExpression(function) => {
                walk::walk_property_key(self, &property.key);
                self.visit_scoped_function(
                    self.aggregate_member_scope(name),
                    function,
                    oxc_syntax::scope::ScopeFlags::empty(),
                );
            }
            Expression::ArrowFunctionExpression(arrow) => {
                walk::walk_property_key(self, &property.key);
                self.visit_scoped_arrow(self.aggregate_member_scope(name), arrow);
            }
            Expression::ObjectExpression(_) if name.is_some() => {
                walk::walk_property_key(self, &property.key);
                if self.function_stack.is_empty() && !self.aggregate_stack.is_empty() {
                    self.push_aggregate(name.expect("checked above"));
                    walk::walk_expression(self, &property.value);
                    self.pop_aggregate();
                } else {
                    self.push_function(name);
                    walk::walk_expression(self, &property.value);
                    self.pop_function();
                }
            }
            _ => walk::walk_object_property(self, property),
        }
    }

    pub(super) fn visit_property_definition_impl(&mut self, property: &PropertyDefinition<'a>) {
        let name =
            crate::codebase::ts_source::static_property_key_name(&property.key).map(str::to_string);
        match property.value.as_ref() {
            Some(Expression::FunctionExpression(function)) => {
                self.walk_property_definition_prefix(property);
                self.visit_scoped_function(
                    self.aggregate_member_scope(name),
                    function,
                    oxc_syntax::scope::ScopeFlags::empty(),
                );
            }
            Some(Expression::ArrowFunctionExpression(arrow)) => {
                self.walk_property_definition_prefix(property);
                self.visit_scoped_arrow(self.aggregate_member_scope(name), arrow);
            }
            _ if !property.r#static
                && self.function_stack.is_empty()
                && !self.aggregate_stack.is_empty() =>
            {
                let scope = self.aggregate_stack.join("/");
                self.push_function(Some(scope));
                walk::walk_property_definition(self, property);
                self.pop_function();
            }
            _ => walk::walk_property_definition(self, property),
        }
    }

    fn walk_property_definition_prefix(&mut self, property: &PropertyDefinition<'a>) {
        walk::walk_decorators(self, &property.decorators);
        walk::walk_property_key(self, &property.key);
        if let Some(annotation) = &property.type_annotation {
            walk::walk_ts_type_annotation(self, annotation);
        }
    }
}
