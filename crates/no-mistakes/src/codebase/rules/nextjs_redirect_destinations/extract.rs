use crate::codebase::ts_source::{
    byte_offset_to_line, static_property_key_name, unwrap_ts_wrappers,
};
use oxc_ast::ast::{
    Expression, MethodDefinition, ObjectExpression, ObjectProperty, ObjectPropertyKind, Program,
    PropertyDefinition,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ExtractedDestinations {
    pub(super) body_found: bool,
    pub(super) destinations: Vec<ExtractedDestination>,
    pub(super) saw_destination_property: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExtractedDestination {
    pub(super) value: String,
    pub(super) line: usize,
}

pub(super) fn extract_named_destinations(
    path: &Path,
    source: &str,
    name: &str,
) -> ExtractedDestinations {
    crate::ast::with_program(path, source, |program, source| {
        extract_named_destinations_from_program(program, source, name)
    })
    .unwrap_or_default()
}

fn extract_named_destinations_from_program(
    program: &Program<'_>,
    source: &str,
    name: &str,
) -> ExtractedDestinations {
    let mut finder = BodyFinder {
        name,
        source,
        body_found: false,
        destinations: BTreeMap::new(),
        saw_destination_property: false,
    };
    finder.visit_program(program);
    ExtractedDestinations {
        body_found: finder.body_found,
        destinations: finder
            .destinations
            .into_iter()
            .map(|(value, line)| ExtractedDestination { value, line })
            .collect(),
        saw_destination_property: finder.saw_destination_property,
    }
}

struct BodyFinder<'a, 'n> {
    name: &'n str,
    source: &'a str,
    body_found: bool,
    destinations: BTreeMap<String, usize>,
    saw_destination_property: bool,
}

impl BodyFinder<'_, '_> {
    fn collect_from_expression(&mut self, expression: &Expression<'_>) {
        self.body_found = true;
        let mut collector = DestinationCollector {
            source: self.source,
            destinations: BTreeMap::new(),
            saw_destination_property: false,
        };
        collector.visit_expression(expression);
        self.destinations = collector.destinations;
        self.saw_destination_property = collector.saw_destination_property;
    }
}

impl<'a> Visit<'a> for BodyFinder<'a, '_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&property.key) == Some(self.name)
            && is_function_like(&property.value)
        {
            self.collect_from_expression(&property.value);
            return;
        }
        walk::walk_object_property(self, property);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&method.key) == Some(self.name) {
            if let Some(body) = method.value.body.as_deref() {
                self.body_found = true;
                let mut collector = DestinationCollector {
                    source: self.source,
                    destinations: BTreeMap::new(),
                    saw_destination_property: false,
                };
                collector.visit_function_body(body);
                self.destinations = collector.destinations;
                self.saw_destination_property = collector.saw_destination_property;
                return;
            }
        }
        walk::walk_method_definition(self, method);
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&property.key) == Some(self.name) {
            if let Some(value) = property.value.as_ref() {
                if is_function_like(value) {
                    self.collect_from_expression(value);
                    return;
                }
            }
        }
        walk::walk_property_definition(self, property);
    }
}

struct DestinationCollector<'a> {
    source: &'a str,
    destinations: BTreeMap<String, usize>,
    saw_destination_property: bool,
}

impl<'a> Visit<'a> for DestinationCollector<'a> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        inspect_destination_object(object, self.source, self);
        walk::walk_object_expression(self, object);
    }
}

fn inspect_destination_object(
    object: &ObjectExpression<'_>,
    source: &str,
    collector: &mut DestinationCollector<'_>,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if static_property_key_name(&property.key) != Some("destination") {
            continue;
        }
        collector.saw_destination_property = true;
        if let Some(value) = string_literal_value(&property.value) {
            let line = byte_offset_to_line(source, property.value.span().start as usize) as usize;
            collector.destinations.entry(value).or_insert(line);
        }
    }
}

fn is_function_like(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

fn string_literal_value(expression: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().to_string()),
        _ => None,
    }
}
