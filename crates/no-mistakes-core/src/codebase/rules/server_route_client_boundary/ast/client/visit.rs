use super::{binding_names, is_client_http_module, ClientHttpVisitor};
use oxc_ast::ast::{
    AssignmentExpression, CallExpression, Class, ClassType, FunctionType, ImportOrExportKind,
    SwitchStatement, TSImportEqualsDeclaration, TSModuleReference, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl<'a> Visit<'a> for ClientHttpVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let source = import.source.value.as_str();
        if is_client_http_module(source) {
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    self.add_client_import_specifier(source, specifier);
                }
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_ts_import_equals_declaration(&mut self, import: &TSImportEqualsDeclaration<'a>) {
        if import.import_kind == ImportOrExportKind::Value
            && matches!(
                &import.module_reference,
                TSModuleReference::ExternalModuleReference(reference)
                    if is_client_http_module(reference.expression.value.as_str())
            )
        {
            self.add_client_name(import.id.name.to_string());
        }
        walk::walk_ts_import_equals_declaration(self, import);
    }

    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        let previous_in_var_declaration = self.in_var_declaration;
        self.in_var_declaration = declaration.kind == VariableDeclarationKind::Var;
        walk::walk_variable_declaration(self, declaration);
        self.in_var_declaration = previous_in_var_declaration;
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if !assignment.operator.is_assign() {
            walk::walk_assignment_expression(self, assignment);
            return;
        }
        self.visit_assignment_target(&assignment.left);
        self.visit_expression(&assignment.right);
        self.update_client_assignment(assignment);
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(init) = decl.init.as_ref() {
            if self.client_factory_method_expr(init) {
                for name in binding_names(&decl.id) {
                    self.add_client_factory_callee_name(name);
                }
            } else if self.client_object_method_expr(init) {
                for name in binding_names(&decl.id) {
                    self.add_client_callee_name(name);
                }
            } else if super::client_module_expr(init) || self.client_expr(init) {
                self.add_client_bindings_from_pattern(&decl.id);
            } else {
                self.mark_binding_pattern_shadowed(&decl.id);
            }
        } else {
            self.mark_binding_pattern_shadowed(&decl.id);
        }
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        let declares_function_binding = matches!(
            function.r#type,
            FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction
        );
        if declares_function_binding {
            if let Some(id) = function.id.as_ref() {
                self.shadow_name(id.name.to_string());
            }
        }
        self.enter_scope(true);
        if !declares_function_binding {
            if let Some(id) = function.id.as_ref() {
                self.shadow_name(id.name.to_string());
            }
        }
        self.mark_parameters_shadowed(&function.params);
        if let Some(body) = &function.body {
            self.mark_var_declarations_shadowed(&body.statements);
            self.mark_lexical_declarations_shadowed(&body.statements);
        }
        walk::walk_function(self, function, flags);
        self.leave_scope();
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.enter_scope(true);
        for param in &arrow.params.items {
            self.mark_binding_pattern_shadowed(&param.pattern);
        }
        if let Some(rest) = &arrow.params.rest {
            self.mark_binding_pattern_shadowed(&rest.rest.argument);
        }
        self.mark_var_declarations_shadowed(&arrow.body.statements);
        self.mark_lexical_declarations_shadowed(&arrow.body.statements);
        walk::walk_arrow_function_expression(self, arrow);
        self.leave_scope();
    }

    fn visit_catch_clause(&mut self, catch_clause: &oxc_ast::ast::CatchClause<'a>) {
        self.enter_scope(false);
        if let Some(param) = &catch_clause.param {
            self.mark_binding_pattern_shadowed(&param.pattern);
        }
        walk::walk_catch_clause(self, catch_clause);
        self.leave_scope();
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if class.r#type == ClassType::ClassDeclaration {
            if let Some(id) = class.id.as_ref() {
                self.shadow_name(id.name.to_string());
            }
            walk::walk_class(self, class);
            return;
        }
        self.enter_scope(false);
        if let Some(id) = class.id.as_ref() {
            self.shadow_name(id.name.to_string());
        }
        walk::walk_class(self, class);
        self.leave_scope();
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        self.enter_scope(false);
        self.mark_lexical_declarations_shadowed(&block.body);
        walk::walk_block_statement(self, block);
        self.leave_scope();
    }

    fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
        self.visit_expression(&switch.discriminant);
        self.enter_scope(false);
        self.mark_switch_lexical_declarations_shadowed(&switch.cases);
        self.visit_switch_cases(&switch.cases);
        self.leave_scope();
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.client_call_expr(&call.callee) {
            self.lines.push(crate::codebase::ts_source::line_number(
                self.source,
                call.span.start,
            ));
        }
        walk::walk_call_expression(self, call);
    }
}
