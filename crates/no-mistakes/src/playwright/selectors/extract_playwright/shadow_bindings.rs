use oxc_ast::ast::{BindingPattern, Statement, VariableDeclaration, VariableDeclarationKind};
use oxc_ast_visit::Visit;
use std::collections::HashSet;

pub(super) fn collect_binding_names(pattern: &BindingPattern<'_>, names: &mut HashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.to_string());
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, names);
        }
    }
}

pub(super) fn collect_direct_lexical_declarations(
    statements: &[Statement<'_>],
    names: &mut HashSet<String>,
) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration)
                if declaration.kind != VariableDeclarationKind::Var =>
            {
                for declarator in &declaration.declarations {
                    collect_binding_names(&declarator.id, names);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(identifier) = &function.id {
                    names.insert(identifier.name.to_string());
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(identifier) = &class.id {
                    names.insert(identifier.name.to_string());
                }
            }
            _ => {}
        }
    }
}

pub(super) fn collect_lexical_variable_names(
    declaration: &VariableDeclaration<'_>,
    names: &mut HashSet<String>,
) {
    if declaration.kind != VariableDeclarationKind::Var {
        for declarator in &declaration.declarations {
            collect_binding_names(&declarator.id, names);
        }
    }
}

pub(super) fn collect_function_scope_declarations(
    statements: &[Statement<'_>],
    names: &mut HashSet<String>,
) {
    collect_direct_lexical_declarations(statements, names);
    let mut visitor = VarBindingVisitor { names };
    for statement in statements {
        visitor.visit_statement(statement);
    }
}

struct VarBindingVisitor<'a> {
    names: &'a mut HashSet<String>,
}

impl<'a, 'b> Visit<'a> for VarBindingVisitor<'b> {
    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                collect_binding_names(&declarator.id, self.names);
            }
        }
    }

    fn visit_function(&mut self, _: &oxc_ast::ast::Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
    }
    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &oxc_ast::ast::Class<'a>) {}
}
