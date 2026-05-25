use crate::fetch::cache_opts::cache_wrapper_name;
use crate::fetch::visit_helpers::{enter_cache_wrapper, leave_cache_wrapper, try_extract_fetch};
use oxc_ast::ast::{CallExpression, Expression, FunctionType, ImportDeclarationSpecifier};
use oxc_ast_visit::{walk, Visit};

pub use crate::fetch::visitor_state::{FetchScope, FetchVisitor};

impl<'a> Visit<'a> for FetchVisitor<'a> {
    fn visit_binding_identifier(&mut self, ident: &oxc_ast::ast::BindingIdentifier<'a>) {
        if self.in_var_declaration {
            self.mark_identifier_shadowed_in_var_scope(ident.name.as_ref());
        } else {
            self.mark_identifier_shadowed(ident.name.as_ref());
        }
        if ident.name.as_ref() == "fetch" {
            self.mark_fetch_shadowed();
        }
        walk::walk_binding_identifier(self, ident);
    }

    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        if let Some(specifiers) = import.specifiers.as_ref() {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default_import) => {
                        self.mark_identifier_shadowed(default_import.local.name.as_ref());
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_import) => {
                        self.mark_identifier_shadowed(namespace_import.local.name.as_ref());
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                        self.mark_identifier_shadowed(import_specifier.local.name.as_ref());
                    }
                }
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let is_function_declaration = matches!(function.r#type, FunctionType::FunctionDeclaration);
        let is_ts_declare_function = matches!(function.r#type, FunctionType::TSDeclareFunction);
        if is_function_declaration || is_ts_declare_function {
            if let Some(id) = function.id.as_ref() {
                self.mark_identifier_shadowed_in_var_scope(id.name.as_ref());
            }
        }
        let name = function
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .or_else(|| {
                if matches!(function.r#type, FunctionType::FunctionExpression) {
                    self.pending_var_name.take()
                } else {
                    None
                }
            });
        self.function_name_stack.push(name);
        self.enter_fetch_scope(true);
        walk::walk_function(self, function, flags);
        self.leave_fetch_scope();
        self.function_name_stack.pop();
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let name = self.pending_var_name.take();
        self.function_name_stack.push(name);
        self.enter_fetch_scope(false);
        walk::walk_arrow_function_expression(self, arrow);
        self.leave_fetch_scope();
        self.function_name_stack.pop();
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id {
            if decl.init.as_ref().is_some_and(|init| {
                matches!(
                    init,
                    Expression::ArrowFunctionExpression(_)
                        | Expression::FunctionExpression(_)
                )
            }) {
                self.pending_var_name = Some(id.name.to_string());
            }
        }
        walk::walk_variable_declarator(self, decl);
        self.pending_var_name = None;
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.visit_expression(&statement.test);
        self.conditional_depth += 1;
        self.visit_statement(&statement.consequent);
        if let Some(alternate) = &statement.alternate {
            self.visit_statement(alternate);
        }
        self.conditional_depth -= 1;
    }

    fn visit_conditional_expression(
        &mut self,
        expr: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        self.visit_expression(&expr.test);
        self.conditional_depth += 1;
        self.visit_expression(&expr.consequent);
        self.visit_expression(&expr.alternate);
        self.conditional_depth -= 1;
    }

    fn visit_logical_expression(
        &mut self,
        expr: &oxc_ast::ast::LogicalExpression<'a>,
    ) {
        self.visit_expression(&expr.left);
        self.conditional_depth += 1;
        self.visit_expression(&expr.right);
        self.conditional_depth -= 1;
    }

    fn visit_try_statement(&mut self, stmt: &oxc_ast::ast::TryStatement<'a>) {
        if stmt.handler.is_some() {
            self.try_depth += 1;
            self.visit_block_statement(&stmt.block);
            self.try_depth -= 1;
        } else {
            self.visit_block_statement(&stmt.block);
        }
        if let Some(handler) = &stmt.handler {
            self.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &stmt.finalizer {
            self.visit_block_statement(finalizer);
        }
    }

    fn visit_catch_clause(&mut self, catch_clause: &oxc_ast::ast::CatchClause<'a>) {
        self.enter_fetch_scope(false);
        walk::walk_catch_clause(self, catch_clause);
        self.leave_fetch_scope();
    }

    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        let previous_in_var_declaration = self.in_var_declaration;
        self.in_var_declaration = declaration.kind == oxc_ast::ast::VariableDeclarationKind::Var;
        walk::walk_variable_declaration(self, declaration);
        self.in_var_declaration = previous_in_var_declaration;
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        self.enter_fetch_scope(false);
        walk::walk_block_statement(self, block);
        self.leave_fetch_scope();
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if is_promise_all_call(expr) {
            self.promise_all_depth += 1;
            walk::walk_call_expression(self, expr);
            self.promise_all_depth -= 1;
            return;
        }

        if cache_wrapper_name(expr).is_some() {
            let (prev_fn, prev_kind) = enter_cache_wrapper(expr, self);
            walk::walk_call_expression(self, expr);
            leave_cache_wrapper(self, prev_fn, prev_kind);
            return;
        }

        let in_scope = self
            .component_span
            .map(|s| expr.span.start >= s.start && expr.span.end <= s.end)
            .unwrap_or(true);

        if in_scope {
            if let Some(occurrence) = try_extract_fetch(expr, self) {
                self.fetches.push(occurrence);
            }
        }
        walk::walk_call_expression(self, expr);
    }
}

fn is_promise_all_call(expr: &CallExpression<'_>) -> bool {
    if let Expression::StaticMemberExpression(member) = &expr.callee {
        if let Expression::Identifier(obj) = &member.object {
            if obj.name.as_ref() == "Promise" {
                let method = member.property.name.as_ref();
                return method == "all" || method == "allSettled";
            }
        }
    }
    false
}

#[cfg(test)]
mod tests;
