use super::{helpers::binding_name, ServerRouteVisitor};
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, Expression, FormalParameters, Statement, VariableDeclarator,
};
use std::collections::{BTreeSet, HashMap};

struct HandlerShape<'a> {
    params: &'a FormalParameters<'a>,
    body: &'a [Statement<'a>],
}

impl<'a> ServerRouteVisitor<'a> {
    pub(super) fn pre_collect_named_handlers(&mut self, program: &'a oxc_ast::ast::Program<'a>) {
        let mut handlers: HashMap<String, HandlerShape<'a>> = HashMap::new();
        for statement in &program.body {
            self.collect_named_handler_bodies(statement, &mut handlers);
        }
        self.compute_named_handler_query_params(&handlers);
    }

    fn collect_named_handler_bodies(
        &mut self,
        statement: &'a Statement<'a>,
        handlers: &mut HashMap<String, HandlerShape<'a>>,
    ) {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    if let Some(body) = &function.body {
                        handlers.insert(
                            id.name.to_string(),
                            HandlerShape {
                                params: &function.params,
                                body: &body.statements,
                            },
                        );
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.collect_named_handler_from_declarator(declarator, handlers);
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    match declaration {
                        oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                            if let Some(id) = &function.id {
                                if let Some(body) = &function.body {
                                    handlers.insert(
                                        id.name.to_string(),
                                        HandlerShape {
                                            params: &function.params,
                                            body: &body.statements,
                                        },
                                    );
                                }
                            }
                        }
                        oxc_ast::ast::Declaration::VariableDeclaration(declaration) => {
                            for declarator in &declaration.declarations {
                                self.collect_named_handler_from_declarator(declarator, handlers);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &export.declaration
                {
                    if let Some(id) = &function.id {
                        if let Some(body) = &function.body {
                            handlers.insert(
                                id.name.to_string(),
                                HandlerShape {
                                    params: &function.params,
                                    body: &body.statements,
                                },
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_named_handler_from_declarator(
        &mut self,
        declarator: &'a VariableDeclarator<'a>,
        handlers: &mut HashMap<String, HandlerShape<'a>>,
    ) {
        let Some(name) = binding_name(&declarator.id) else {
            return;
        };
        let Some(init) = &declarator.init else {
            return;
        };
        let handler = match init {
            Expression::ArrowFunctionExpression(arrow) => Some(HandlerShape {
                params: &arrow.params,
                body: &arrow.body.statements,
            }),
            Expression::FunctionExpression(function) => {
                function.body.as_ref().map(|body| HandlerShape {
                    params: &function.params,
                    body: &body.statements,
                })
            }
            _ => None,
        };
        if let Some(handler) = handler {
            handlers.insert(name, handler);
        }
    }

    fn compute_named_handler_query_params(&mut self, handlers: &HashMap<String, HandlerShape<'a>>) {
        if handlers.is_empty() {
            return;
        }
        for name in handlers.keys() {
            self.named_handler_query_params
                .entry(name.clone())
                .or_default();
        }

        let mut remaining_budget = handlers.len().saturating_add(1);
        while remaining_budget > 0 {
            remaining_budget -= 1;
            let prior = self.named_handler_query_params.clone();
            let mut next = prior.clone();
            let mut changed = false;

            for (name, handler) in handlers {
                let params: BTreeSet<String> =
                    self.query_params_from_function(handler.params, handler.body, &prior);
                if prior.get(name) != Some(&params) {
                    changed = true;
                }
                next.insert(name.clone(), params);
            }

            if !changed {
                break;
            }
            self.named_handler_query_params = next;
        }
    }
}
