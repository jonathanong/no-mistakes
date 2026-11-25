use super::{
    bindings::Runner, literal_object, property, test_static_alternatives, Collector, Shape,
    SourceDemand,
};
use crate::ast::js_literal;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{AssignmentTarget, Expression, ObjectExpression, Program, Statement};
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

impl<'ast> Collector<'_, 'ast> {
    pub(super) fn collect_config(&mut self, program: &'ast Program<'ast>, demand: SourceDemand) {
        let Some(export) = program
            .body
            .iter()
            .rev()
            .find_map(|statement| match statement {
                Statement::ExportDefaultDeclaration(export) => export.declaration.as_expression(),
                Statement::ExpressionStatement(statement) => {
                    self.commonjs_export(&statement.expression)
                }
                _ => None,
            })
        else {
            return;
        };
        if demand.vitest_config {
            if let Some(config) = self.config_object(export, Runner::Vitest) {
                self.vitest_config(config);
            }
        }
        if demand.playwright_config {
            if let Some(config) = self.config_object(export, Runner::Playwright) {
                self.playwright_options(config, None);
                for project in self.projects(config, Runner::Playwright) {
                    let name = project_name(project);
                    self.playwright_options(project, name.as_deref());
                }
            }
        }
    }

    fn vitest_config(&mut self, root: &ObjectExpression<'_>) {
        let Some(test) = property(root, "test").and_then(|prop| literal_object(&prop.value)) else {
            return;
        };
        if let Some(retry) = property(test, "retry") {
            self.property_retry("Vitest test.retry", retry, project_name(test).as_deref());
        }
        for project in self.projects(test, Runner::Vitest) {
            if let Some(test) =
                property(project, "test").and_then(|prop| literal_object(&prop.value))
            {
                if let Some(retry) = property(test, "retry") {
                    self.property_retry("Vitest test.retry", retry, project_name(test).as_deref());
                }
            }
        }
    }

    fn playwright_options(&mut self, object: &ObjectExpression<'_>, project: Option<&str>) {
        if let Some(retries) = property(object, "retries") {
            self.property_retry("Playwright retries", retries, project);
        }
        if let Some(workers) = property(object, "workers").filter(|option| {
            test_static_alternatives(
                &option.value,
                &|value| matches!(value, Expression::NumericLiteral(number) if number.value == 1.0),
            )
        }) {
            self.record(
                Shape::Serial("Playwright workers: 1"),
                workers.key.span(),
                project,
            );
        }
        if let Some(parallel) = property(object, "fullyParallel").filter(|option| {
            test_static_alternatives(
                &option.value,
                &|value| matches!(value, Expression::BooleanLiteral(boolean) if !boolean.value),
            )
        }) {
            self.record(
                Shape::Serial("Playwright fullyParallel: false"),
                parallel.key.span(),
                project,
            );
        }
    }

    fn commonjs_export(
        &self,
        expression: &'ast Expression<'ast>,
    ) -> Option<&'ast Expression<'ast>> {
        let Expression::AssignmentExpression(assignment) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        let AssignmentTarget::StaticMemberExpression(member) = &assignment.left else {
            return None;
        };
        let Expression::Identifier(module) = unwrap_ts_wrappers(&member.object) else {
            return None;
        };
        (assignment.operator == AssignmentOperator::Assign
            && member.property.name == "exports"
            && module.name == "module"
            && self.bindings.symbol(module).is_none())
        .then_some(&assignment.right)
    }

    fn resolve_const(&self, expression: &'ast Expression<'ast>) -> &'ast Expression<'ast> {
        if let Expression::Identifier(identifier) = unwrap_ts_wrappers(expression) {
            return self
                .bindings
                .constant_initializer(identifier)
                .unwrap_or(expression);
        }
        expression
    }

    fn config_object(
        &self,
        expression: &'ast Expression<'ast>,
        runner: Runner,
    ) -> Option<&'ast ObjectExpression<'ast>> {
        let expression = self.resolve_const(expression);
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) {
            let (owner, members, _) = self.bindings.call(&call.callee)?;
            if owner == runner && matches!(members.as_slice(), ["defineConfig" | "defineProject"]) {
                return literal_object(
                    self.resolve_const(call.arguments.first()?.as_expression()?),
                );
            }
        }
        literal_object(expression)
    }

    fn projects(
        &self,
        object: &'ast ObjectExpression<'ast>,
        runner: Runner,
    ) -> Vec<&'ast ObjectExpression<'ast>> {
        let Some(projects) = property(object, "projects") else {
            return Vec::new();
        };
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(&projects.value) else {
            return Vec::new();
        };
        array
            .elements
            .iter()
            .filter_map(|element| self.config_object(element.as_expression()?, runner))
            .collect()
    }
}

pub(super) fn project_name<'a>(
    object: &'a ObjectExpression<'a>,
) -> Option<std::borrow::Cow<'a, str>> {
    match unwrap_ts_wrappers(&property(object, "name")?.value) {
        Expression::StringLiteral(name) => Some(js_literal(name)),
        _ => None,
    }
}
