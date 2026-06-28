use super::{helpers, query_params, ServerRouteVisitor};
use oxc_ast::ast::{Expression, Program, Statement};
use std::collections::BTreeSet;

impl<'a> ServerRouteVisitor<'a> {
    pub(super) fn index_handler_query_params(&mut self, program: &Program<'a>) {
        for statement in &program.body {
            self.index_handler_query_params_from_statement(statement);
        }
    }

    fn index_handler_query_params_from_statement(&mut self, statement: &Statement<'a>) {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(name) = function.id.as_ref() {
                    let mut params = BTreeSet::new();
                    query_params::collect_query_params_from_optional_function_body(
                        function.body.as_ref(),
                        &mut params,
                    );
                    self.insert_handler_query_params(name.name.as_str(), params);
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.index_handler_expression(
                        helpers::binding_name(&declarator.id).as_deref(),
                        declarator.init.as_ref(),
                    );
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    self.index_handler_query_params_from_declaration(declaration);
                }
            }
            _ => {}
        }
    }

    fn index_handler_query_params_from_declaration(
        &mut self,
        declaration: &oxc_ast::ast::Declaration<'a>,
    ) {
        match declaration {
            oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                if let Some(name) = function.id.as_ref() {
                    let mut params = BTreeSet::new();
                    query_params::collect_query_params_from_optional_function_body(
                        function.body.as_ref(),
                        &mut params,
                    );
                    self.insert_handler_query_params(name.name.as_str(), params);
                }
            }
            oxc_ast::ast::Declaration::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.index_handler_expression(
                        helpers::binding_name(&declarator.id).as_deref(),
                        declarator.init.as_ref(),
                    );
                }
            }
            _ => {}
        }
    }

    fn index_handler_expression(&mut self, name: Option<&str>, init: Option<&Expression<'a>>) {
        let (Some(name), Some(init)) = (name, init) else {
            return;
        };
        let mut params = BTreeSet::new();
        self.collect_query_params_from_handler_expression(init, &mut params);
        self.insert_handler_query_params(name, params);
    }

    fn insert_handler_query_params(&mut self, name: &str, params: BTreeSet<String>) {
        if !params.is_empty() {
            self.handler_query_params
                .insert(name.to_string(), params.into_iter().collect());
        }
    }
}
