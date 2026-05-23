use super::{
    is_client_factory_member, is_client_method_name,
    modules::{is_client_named_binding, is_client_named_callee},
    ClientHttpVisitor,
};
use oxc_ast::ast::{BindingPattern, Expression, ImportDeclarationSpecifier, ModuleExportName};

pub(super) fn import_local_name(specifier: &ImportDeclarationSpecifier<'_>) -> String {
    match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => spec.local.name.to_string(),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => spec.local.name.to_string(),
        ImportDeclarationSpecifier::ImportSpecifier(spec) => spec.local.name.to_string(),
    }
}

pub(super) fn binding_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => vec![id.name.to_string()],
        BindingPattern::ObjectPattern(object) => {
            let mut names = Vec::new();
            for prop in &object.properties {
                names.extend(binding_names(&prop.value));
            }
            if let Some(rest) = &object.rest {
                names.extend(binding_names(&rest.argument));
            }
            names
        }
        BindingPattern::ArrayPattern(array) => {
            let mut names = Vec::new();
            for element in array.elements.iter().flatten() {
                names.extend(binding_names(element));
            }
            if let Some(rest) = &array.rest {
                names.extend(binding_names(&rest.argument));
            }
            names
        }
        BindingPattern::AssignmentPattern(assign) => binding_names(&assign.left),
    }
}

impl ClientHttpVisitor<'_> {
    pub(super) fn add_client_import_specifier(
        &mut self,
        source: &str,
        specifier: &ImportDeclarationSpecifier<'_>,
    ) {
        let local = import_local_name(specifier);
        if self.is_shadowed_name(&local) {
            return;
        }
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
            | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                self.add_client_name(local);
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let imported = module_export_name(&specifier.imported);
                if is_client_named_binding(source, &imported) {
                    self.add_client_name(local);
                } else if is_client_named_callee(source, &imported)
                    || is_client_method_name(&imported)
                {
                    self.add_client_callee_name(local);
                } else if is_client_factory_member(&imported) {
                    self.add_client_factory_callee_name(local);
                }
            }
        }
    }

    pub(super) fn add_client_bindings_from_pattern(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::ObjectPattern(object) => {
                for prop in &object.properties {
                    self.add_client_binding_for_property(
                        prop.key.static_name().as_deref(),
                        binding_names(&prop.value),
                    );
                }
                if let Some(rest) = &object.rest {
                    self.add_client_bindings_from_pattern(&rest.argument);
                }
            }
            BindingPattern::ArrayPattern(_) => {
                self.mark_binding_pattern_shadowed(pattern);
            }
            _ => {
                for name in binding_names(pattern) {
                    self.add_client_name(name);
                }
            }
        }
    }

    fn add_client_binding_for_property(&mut self, key: Option<&str>, names: Vec<String>) {
        if key.is_some_and(is_client_method_name) {
            for name in names {
                self.add_client_callee_name(name);
            }
        } else if key.is_some_and(is_client_factory_member) {
            for name in names {
                self.add_client_factory_callee_name(name);
            }
        } else {
            for name in names {
                self.shadow_name(name);
            }
        }
    }

    pub(super) fn client_object_method_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.client_object_method_expr(&expr.expression)
            }
            _ => expr.as_member_expression().is_some_and(|member| {
                member
                    .static_property_name()
                    .is_some_and(is_client_method_name)
                    && self.client_expr(member.object())
            }),
        }
    }

    pub(super) fn client_factory_method_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.client_factory_method_expr(&expr.expression)
            }
            _ => expr.as_member_expression().is_some_and(|member| {
                member
                    .static_property_name()
                    .is_some_and(is_client_factory_member)
                    && self.client_expr(member.object())
            }),
        }
    }
}

#[rustfmt::skip]
fn module_export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(id) => id.name.to_string(), ModuleExportName::IdentifierReference(id) => id.name.to_string(),
        ModuleExportName::StringLiteral(value) => value.value.to_string(),
    }
}
