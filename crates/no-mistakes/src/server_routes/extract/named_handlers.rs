use super::{helpers::binding_name, ServerRouteVisitor};
use oxc_ast::ast::{ExportDefaultDeclarationKind, Expression, Statement, VariableDeclarator};
use std::collections::{BTreeSet, HashMap};

impl<'a> ServerRouteVisitor<'a> {
    pub(super) fn pre_collect_named_handlers(&mut self, program: &'a oxc_ast::ast::Program<'a>) {
        let mut handler_bodies: HashMap<String, &'a [Statement<'a>]> = HashMap::new();
        for statement in &program.body {
            self.collect_named_handler_bodies(statement, &mut handler_bodies);
        }
        self.compute_named_handler_query_params(&handler_bodies);
    }

    fn collect_named_handler_bodies(
        &mut self,
        statement: &'a Statement<'a>,
        handler_bodies: &mut HashMap<String, &'a [Statement<'a>]>,
    ) {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    if let Some(body) = &function.body {
                        handler_bodies.insert(id.name.to_string(), &body.statements);
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.collect_named_handler_from_declarator(declarator, handler_bodies);
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    match declaration {
                        oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                            if let Some(id) = &function.id {
                                if let Some(body) = &function.body {
                                    handler_bodies.insert(id.name.to_string(), &body.statements);
                                }
                            }
                        }
                        oxc_ast::ast::Declaration::VariableDeclaration(declaration) => {
                            for declarator in &declaration.declarations {
                                self.collect_named_handler_from_declarator(
                                    declarator,
                                    handler_bodies,
                                );
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
                            handler_bodies.insert(id.name.to_string(), &body.statements);
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
        handler_bodies: &mut HashMap<String, &'a [Statement<'a>]>,
    ) {
        let Some(name) = binding_name(&declarator.id) else {
            return;
        };
        let Some(init) = &declarator.init else {
            return;
        };
        if let Some(body) = match init {
            Expression::ArrowFunctionExpression(arrow) => Some(&*arrow.body),
            Expression::FunctionExpression(function) => {
                function.body.as_ref().map(oxc_allocator::Box::as_ref)
            }
            _ => None,
        } {
            handler_bodies.insert(name, &body.statements);
        }
    }

    fn compute_named_handler_query_params(
        &mut self,
        handler_bodies: &HashMap<String, &'a [Statement<'a>]>,
    ) {
        if handler_bodies.is_empty() {
            return;
        }
        for name in handler_bodies.keys() {
            self.named_handler_query_params
                .entry(name.clone())
                .or_default();
        }

        let mut remaining_budget = handler_bodies.len().saturating_add(1);
        while remaining_budget > 0 {
            remaining_budget -= 1;
            let prior = self.named_handler_query_params.clone();
            let mut next = prior.clone();
            let mut changed = false;

            for (name, body) in handler_bodies {
                let params: BTreeSet<String> = self.query_params_from_function_body(body, &prior);
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
