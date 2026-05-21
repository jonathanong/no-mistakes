use super::{binding_names, ClientHttpVisitor};
use oxc_ast::ast::{
    Class, Function, Statement, SwitchCase, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

impl ClientHttpVisitor<'_> {
    pub(super) fn mark_var_declarations_shadowed(&mut self, statements: &[Statement<'_>]) {
        let mut visitor = VarBindingVisitor::default();
        for statement in statements {
            visitor.visit_statement(statement);
        }
        let previous_in_var_declaration = self.in_var_declaration;
        self.in_var_declaration = true;
        for name in visitor.names {
            self.shadow_name(name);
        }
        self.in_var_declaration = previous_in_var_declaration;
    }

    pub(super) fn mark_lexical_declarations_shadowed(&mut self, statements: &[Statement<'_>]) {
        for statement in statements {
            match statement {
                Statement::VariableDeclaration(declaration)
                    if declaration.kind != VariableDeclarationKind::Var =>
                {
                    for declarator in &declaration.declarations {
                        self.mark_binding_pattern_shadowed(&declarator.id);
                    }
                }
                Statement::ClassDeclaration(class) => {
                    if let Some(id) = class.id.as_ref() {
                        self.shadow_name(id.name.to_string());
                    }
                }
                Statement::FunctionDeclaration(function) => {
                    if let Some(id) = function.id.as_ref() {
                        self.shadow_name(id.name.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn mark_switch_lexical_declarations_shadowed(&mut self, cases: &[SwitchCase<'_>]) {
        for case in cases {
            self.mark_lexical_declarations_shadowed(&case.consequent);
        }
    }
}

#[derive(Default)]
struct VarBindingVisitor {
    names: Vec<String>,
}

impl<'a> Visit<'a> for VarBindingVisitor {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.names.extend(binding_names(&declarator.id));
            }
        }
    }

    fn visit_function(&mut self, _function: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
    }

    fn visit_class(&mut self, _class: &Class<'a>) {}
}
