use super::{Location, Origin, Shape, SourceDemand};
use crate::ast::{js_literal, property_key_name};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectProperty, Program};
use oxc_ast_visit::Visit;
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};

mod bindings;
mod calls;
mod playwright_bindings;
mod playwright_wrappers;
pub(crate) use playwright_bindings::fixture_initializers;
mod configs;
mod factories;

use bindings::Bindings;

pub(crate) fn collect<'a>(
    demand: SourceDemand,
    source: &str,
    program: &'a Program<'a>,
    semantic: &Semantic<'a>,
) -> Vec<Location> {
    let bindings = Bindings::new(
        program,
        semantic,
        demand.vitest_globals,
        demand.playwright_tests,
    );
    let mut collector = Collector {
        bindings,
        demand,
        line_starts: std::iter::once(0)
            .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
            .collect(),
        locations: Vec::new(),
    };
    if demand.vitest_config || demand.playwright_config {
        collector.collect_config(program, demand);
    }
    if demand.playwright_factory {
        collector.collect_playwright_factory(program);
    }
    if demand.standard_tests || demand.playwright_tests {
        collector.visit_program(program);
    }
    collector.locations
}

struct Collector<'scope, 'ast> {
    bindings: Bindings<'scope, 'ast>,
    demand: SourceDemand,
    line_starts: Vec<usize>,
    locations: Vec<Location>,
}

impl Collector<'_, '_> {
    fn record(&mut self, shape: Shape, span: Span, project: Option<&str>) {
        self.locations.push(self.location(shape, span, project));
    }

    fn location(&self, shape: Shape, span: Span, project: Option<&str>) -> Location {
        Location {
            shape,
            line: self
                .line_starts
                .partition_point(|offset| *offset <= span.start as usize),
            project: project.map(str::to_owned),
            origin: Origin::Settings,
            playwright_import: None,
            playwright_binding: None,
            playwright_wrappers: Vec::new(),
            resolved_playwright_exports: Vec::new(),
        }
    }

    fn property_retry(
        &mut self,
        shape: &'static str,
        property: &ObjectProperty<'_>,
        project: Option<&str>,
    ) {
        self.record(
            Shape::Retry(shape, retry_count(&property.value)),
            property.key.span(),
            project,
        );
    }
}

fn property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a ObjectProperty<'a>> {
    object
        .properties
        .iter()
        .rev()
        .filter_map(|prop| prop.as_property())
        .find(|prop| property_key_name(&prop.key).as_deref() == Some(name))
}

fn literal_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn retry_count(expression: &Expression<'_>) -> Option<u64> {
    match unwrap_ts_wrappers(expression) {
        Expression::NumericLiteral(number) => {
            let value = number.value;
            (value >= 0.0 && value < u64::MAX as f64 && value.fract() == 0.0)
                .then_some(value as u64)
        }
        Expression::UnaryExpression(argument)
            if argument.operator == oxc_syntax::operator::UnaryOperator::UnaryPlus =>
        {
            retry_count(&argument.argument)
        }
        Expression::ConditionalExpression(branch) => {
            Some(retry_count(&branch.consequent)?.max(retry_count(&branch.alternate)?))
        }
        _ => None,
    }
}

fn literal_string_is(expression: &Expression<'_>, expected: &str) -> bool {
    test_static_alternatives(
        expression,
        &|value| matches!(value, Expression::StringLiteral(text) if js_literal(text).as_ref() == expected),
    )
}

fn test_static_alternatives(
    expression: &Expression<'_>,
    predicate: &impl Fn(&Expression<'_>) -> bool,
) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(branch) => {
            test_static_alternatives(&branch.consequent, predicate)
                || test_static_alternatives(&branch.alternate, predicate)
        }
        leaf => predicate(leaf),
    }
}
