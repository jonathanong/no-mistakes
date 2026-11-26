use super::{configs::project_name, literal_object, property, Collector, Origin};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, Function, FunctionBody,
    ObjectExpression, Program, ReturnStatement, Statement,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

impl<'ast> Collector<'_, 'ast> {
    pub(super) fn collect_playwright_factory(&mut self, program: &'ast Program<'ast>) {
        let start = self.locations.len();
        for statement in &program.body {
            match statement {
                Statement::ExportDeclaration(export) => match &export.declaration {
                    Declaration::FunctionDeclaration(function) => {
                        self.factory_function(function);
                    }
                    Declaration::VariableDeclaration(declaration)
                        if declaration.kind.is_const() =>
                    {
                        for binding in &declaration.declarations {
                            if let Some(init) = &binding.init {
                                self.factory_expression(init);
                            }
                        }
                    }
                    _ => {}
                },
                Statement::ExportDefaultDeclaration(export) => {
                    if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                        &export.declaration
                    {
                        self.factory_function(function);
                    } else if let Some(expression) = export.declaration.as_expression() {
                        self.factory_expression(expression);
                    }
                }
                _ => {}
            }
        }
        for location in &mut self.locations[start..] {
            location.origin = Origin::PlaywrightFactory;
        }
    }

    fn factory_expression(&mut self, expression: &'ast Expression<'ast>) {
        match unwrap_ts_wrappers(expression) {
            Expression::ArrowFunctionExpression(arrow) => {
                if let Some(expression) = arrow.get_expression() {
                    self.factory_return(expression);
                } else if let Some(body) = arrow.get_function_body() {
                    self.factory_body(body);
                }
            }
            Expression::FunctionExpression(function) => self.factory_function(function),
            _ => {}
        }
    }

    fn factory_function(&mut self, function: &'ast Function<'ast>) {
        if let Some(body) = &function.body {
            self.factory_body(body);
        }
    }

    fn factory_body(&mut self, body: &'ast FunctionBody<'ast>) {
        FactoryReturns(self).visit_function_body(body);
    }

    fn factory_return(&mut self, expression: &Expression<'ast>) {
        let Some(config) = literal_object(expression) else {
            return;
        };
        self.factory_retry(config, None);
        let Some(Expression::ArrayExpression(projects)) =
            property(config, "projects").map(|property| unwrap_ts_wrappers(&property.value))
        else {
            return;
        };
        for project in projects
            .elements
            .iter()
            .filter_map(|entry| literal_object(entry.as_expression()?))
        {
            self.factory_retry(project, project_name(project).as_deref());
        }
    }

    fn factory_retry(&mut self, config: &ObjectExpression<'_>, project: Option<&str>) {
        if let Some(retries) = property(config, "retries") {
            self.property_retry("Playwright retries", retries, project);
        }
    }
}

struct FactoryReturns<'a, 'scope, 'ast>(&'a mut Collector<'scope, 'ast>);

impl<'ast> Visit<'ast> for FactoryReturns<'_, '_, 'ast> {
    fn visit_return_statement(&mut self, statement: &ReturnStatement<'ast>) {
        if let Some(expression) = &statement.argument {
            self.0.factory_return(expression);
        }
    }

    fn visit_function(&mut self, _: &Function<'ast>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'ast>) {
    }
}
