use super::bindings::{require_member_binding, require_module_or_promises};
use super::ResourceVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Expression, Function, MethodDefinition, ObjectProperty,
    VariableDeclarator,
};
use oxc_ast_visit::walk;

impl<'a> ResourceVisitor<'a> {
    pub(super) fn visit_scoped_function(
        &mut self,
        name: Option<String>,
        function: &Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.push_function(name);
        if let Some(id) = &function.id {
            self.declare_binding(id.name.as_str(), None);
        }
        self.shadow_parameters(&function.params);
        self.predeclare_function_var_bindings(function);
        walk::walk_function(self, function, flags);
        self.pop_function();
    }

    pub(super) fn visit_scoped_arrow(
        &mut self,
        name: Option<String>,
        arrow: &ArrowFunctionExpression<'a>,
    ) {
        self.push_function(name);
        self.shadow_parameters(&arrow.params);
        self.predeclare_var_bindings_in_statements(&arrow.body.statements);
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_function();
    }

    pub(super) fn visit_variable_declarator_impl(&mut self, declarator: &VariableDeclarator<'a>) {
        self.register_require(declarator);
        let is_require_binding = declarator.init.as_ref().is_some_and(|init| {
            !self.is_shadowed("require")
                && (require_module_or_promises(init).is_some()
                    || require_member_binding(init).is_some())
        });
        if !is_require_binding {
            self.shadow_pattern(&declarator.id);
        }
        let function_name = match &declarator.id {
            BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.to_string()),
            _ => None,
        };
        match (&declarator.init, function_name) {
            (Some(Expression::ArrowFunctionExpression(arrow)), Some(name)) => {
                self.visit_scoped_arrow(Some(name), arrow)
            }
            (Some(Expression::FunctionExpression(function)), Some(name)) => self
                .visit_scoped_function(
                    Some(name),
                    function,
                    oxc_syntax::scope::ScopeFlags::empty(),
                ),
            (
                Some(Expression::ObjectExpression(_) | Expression::ClassExpression(_)),
                Some(name),
            ) if self.function_stack.is_empty() => {
                self.push_function(Some(name));
                walk::walk_variable_declarator(self, declarator);
                self.pop_function();
            }
            _ => walk::walk_variable_declarator(self, declarator),
        }
    }

    pub(super) fn visit_export_default_declaration_impl(
        &mut self,
        export: &ExportDefaultDeclaration<'a>,
    ) {
        match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => self
                .visit_scoped_function(
                    Some(
                        function
                            .id
                            .as_ref()
                            .map_or_else(|| "default".to_string(), |id| id.name.to_string()),
                    ),
                    function,
                    oxc_syntax::scope::ScopeFlags::empty(),
                ),
            ExportDefaultDeclarationKind::FunctionExpression(function) => self
                .visit_scoped_function(
                    Some("default".to_string()),
                    function,
                    oxc_syntax::scope::ScopeFlags::empty(),
                ),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                self.visit_scoped_arrow(Some("default".to_string()), arrow)
            }
            ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
                self.visit_parenthesized_default(&parenthesized.expression, export);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                let scope = class
                    .id
                    .as_ref()
                    .map_or_else(|| "default".to_string(), |id| id.name.to_string());
                self.visit_default_class(scope, class);
            }
            _ => self.walk_default_export(export),
        }
    }

    fn visit_parenthesized_default(
        &mut self,
        expression: &Expression<'a>,
        export: &ExportDefaultDeclaration<'a>,
    ) {
        if let Some(function) = parenthesized_default_function(expression) {
            self.visit_scoped_function(
                Some("default".to_string()),
                function,
                oxc_syntax::scope::ScopeFlags::empty(),
            );
        } else if let Some(arrow) = parenthesized_default_arrow(expression) {
            self.visit_scoped_arrow(Some("default".to_string()), arrow);
        } else {
            self.walk_default_export(export);
        }
    }

    pub(super) fn visit_default_class(&mut self, scope: String, class: &Class<'a>) {
        self.push_function(Some(scope));
        walk::walk_class(self, class);
        self.pop_function();
    }

    fn walk_default_export(&mut self, export: &ExportDefaultDeclaration<'a>) {
        self.push_function(Some("default".to_string()));
        walk::walk_export_default_declaration(self, export);
        self.pop_function();
    }

    pub(super) fn visit_method_definition_impl(&mut self, method: &MethodDefinition<'a>) {
        walk::walk_decorators(self, &method.decorators);
        walk::walk_property_key(self, &method.key);
        self.visit_scoped_function(
            crate::codebase::ts_source::static_property_key_name(&method.key).map(str::to_string),
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
                self.visit_scoped_function(name, function, oxc_syntax::scope::ScopeFlags::empty());
            }
            Expression::ArrowFunctionExpression(arrow) => {
                walk::walk_property_key(self, &property.key);
                self.visit_scoped_arrow(name, arrow);
            }
            Expression::ObjectExpression(_) if name.is_some() => {
                walk::walk_property_key(self, &property.key);
                self.push_function(name);
                walk::walk_expression(self, &property.value);
                self.pop_function();
            }
            _ => walk::walk_object_property(self, property),
        }
    }
}

fn parenthesized_default_function<'ast, 'reference>(
    expression: &'reference Expression<'ast>,
) -> Option<&'reference Function<'ast>> {
    match expression {
        Expression::FunctionExpression(function) => Some(function),
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_default_function(&parenthesized.expression)
        }
        _ => None,
    }
}

fn parenthesized_default_arrow<'ast, 'reference>(
    expression: &'reference Expression<'ast>,
) -> Option<&'reference ArrowFunctionExpression<'ast>> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => Some(arrow),
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_default_arrow(&parenthesized.expression)
        }
        _ => None,
    }
}
